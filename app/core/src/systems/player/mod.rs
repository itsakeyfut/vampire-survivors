pub mod collision;

use bevy::prelude::*;

use crate::states::AppState;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        use crate::systems::player::collision::{
            apply_damage_to_player, enemy_player_collision, tick_invincibility,
        };
        use crate::systems::spatial::update_spatial_grid;
        app.add_systems(OnEnter(AppState::Playing), spawn_player)
            .add_systems(OnEnter(AppState::GameOver), despawn_game_session)
            .add_systems(OnEnter(AppState::Victory), despawn_game_session)
            .add_systems(OnEnter(AppState::Title), despawn_game_session)
            .add_systems(
                Update,
                (
                    player_movement,
                    regen_hp.after(apply_damage_to_player),
                    tick_invincibility.before(enemy_player_collision),
                    enemy_player_collision.after(update_spatial_grid),
                    apply_damage_to_player.after(enemy_player_collision),
                )
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

use crate::{
    components::{
        BasePlayerStats, CircleCollider, GameSessionEntity, PassiveInventory, Player,
        PlayerFacingDirection, PlayerStats, PlayerWhipSide, WeaponInventory,
    },
    config::{CharacterParams, PlayerParams},
    resources::SelectedCharacter,
    types::{WeaponState, WhipSide},
};

// ---------------------------------------------------------------------------
// Fallback constants (used when RON config is not yet loaded)
// ---------------------------------------------------------------------------

/// Player collider radius in pixels.
const DEFAULT_COLLIDER_PLAYER: f32 = 12.0;

// ---------------------------------------------------------------------------
// Spawn
// ---------------------------------------------------------------------------

/// Spawns the player entity when entering [`AppState::Playing`].
///
/// The player persists through [`AppState::LevelUp`] and [`AppState::Paused`]
/// so that gameplay systems (including upgrade choice generation) can still
/// access the player's inventory while the game is paused.  Cleanup happens
/// via [`despawn_game_session`] when the run truly ends.
///
/// Returns early when a [`Player`] entity already exists — this prevents
/// double-spawning when the state re-enters `Playing` after a `LevelUp` or
/// `Paused` round-trip.
///
/// Character-specific stats (`max_hp`, `move_speed`, `damage_multiplier`,
/// `cooldown_reduction`, `starting_weapon`) are read from [`CharacterParams`].
/// Non-character stats (pickup radius, gem speeds, projectile modifiers, …)
/// come from [`PlayerParams`].  Both fall back to hardcoded defaults while
/// their RON assets are still loading.
pub fn spawn_player(
    mut commands: Commands,
    player_cfg: PlayerParams,
    char_params: CharacterParams,
    selected_character: Res<SelectedCharacter>,
    existing_player: Query<Entity, With<Player>>,
) {
    // Player persists through LevelUp / Paused; only spawn once per run.
    if !existing_player.is_empty() {
        return;
    }

    // Character-specific stats (from character.ron or hardcoded fallback).
    let char_type = selected_character.0;
    let char_stats = char_params.stats_for(char_type);

    // Non-character stats and collider radius (from player.ron or fallback).
    let (stats, collider_radius) = if let Some(cfg) = player_cfg.get() {
        let stats = PlayerStats {
            // Character-specific overrides:
            max_hp: char_stats.max_hp,
            current_hp: char_stats.max_hp,
            move_speed: char_stats.move_speed,
            damage_multiplier: char_stats.damage_multiplier,
            cooldown_reduction: char_stats.cooldown_reduction,
            // Non-character values from player.ron:
            projectile_speed_mult: cfg.base_projectile_speed,
            duration_multiplier: cfg.base_duration_mult,
            area_multiplier: cfg.base_area_mult,
            extra_projectiles: 0,
            luck: cfg.base_luck,
            hp_regen: cfg.base_hp_regen,
            pickup_radius: cfg.pickup_radius,
            gem_attraction_speed: cfg.gem_attraction_speed,
            gem_absorption_radius: cfg.gem_absorption_radius,
        };
        (stats, cfg.collider_radius)
    } else {
        // Full fallback: character-specific values + component defaults for the rest.
        let stats = PlayerStats {
            max_hp: char_stats.max_hp,
            current_hp: char_stats.max_hp,
            move_speed: char_stats.move_speed,
            damage_multiplier: char_stats.damage_multiplier,
            cooldown_reduction: char_stats.cooldown_reduction,
            ..PlayerStats::default()
        };
        (stats, DEFAULT_COLLIDER_PLAYER)
    };

    // Player entity: cyan circle sprite + all required ECS components.
    // GameSessionEntity (not DespawnOnExit) — player persists through LevelUp
    // / Paused; despawn_game_session handles cleanup when the run ends.
    commands.spawn((
        GameSessionEntity,
        Player,
        BasePlayerStats::from(&stats),
        stats,
        Sprite {
            color: Color::srgb(0.2, 0.8, 1.0),
            custom_size: Some(Vec2::splat(collider_radius * 2.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 10.0),
        CircleCollider {
            radius: collider_radius,
        },
        WeaponInventory {
            weapons: vec![WeaponState::new(char_stats.starting_weapon)],
        },
        PassiveInventory::default(),
        // Whip starts on the right side; flips each swing.
        PlayerWhipSide(WhipSide::Right),
        // Knife reads this to know which direction to fire.
        PlayerFacingDirection::default(),
    ));
}

/// Despawns all [`GameSessionEntity`] entities when the run ends.
///
/// Registered on [`OnEnter(AppState::GameOver)`],
/// [`OnEnter(AppState::Victory)`], and [`OnEnter(AppState::Title)`] so
/// every gameplay entity (player, enemies, projectiles, XP gems, whip effects,
/// …) is removed regardless of which path ends the run.
/// This is a no-op when no session entities exist (e.g., on initial startup).
pub fn despawn_game_session(
    mut commands: Commands,
    session_q: Query<Entity, With<GameSessionEntity>>,
) {
    for entity in session_q.iter() {
        commands.entity(entity).despawn();
    }
}

// ---------------------------------------------------------------------------
// Movement
// ---------------------------------------------------------------------------

/// Moves the player based on WASD / arrow-key input.
///
/// - Input from all four cardinal directions is summed then normalised so that
///   diagonal movement is not faster than axis-aligned movement.
/// - Movement is frame-rate independent: distance = speed × Δt.
/// - [`PlayerFacingDirection`] is updated whenever the player moves, so that
///   directional weapons (e.g. Knife) always have a valid aim vector.
pub fn player_movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &PlayerStats, &mut PlayerFacingDirection), With<Player>>,
) {
    let Ok((mut transform, stats, mut facing)) = query.single_mut() else {
        return;
    };

    let mut direction = Vec2::ZERO;

    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    if direction != Vec2::ZERO {
        let normalized = direction.normalize();
        facing.0 = normalized;
        let delta = normalized * stats.move_speed * time.delta_secs();
        transform.translation += delta.extend(0.0);
    }
}

// ---------------------------------------------------------------------------
// HP regeneration
// ---------------------------------------------------------------------------

/// Recovers the player's HP each frame at a rate of `hp_regen` points per
/// second, clamped to `max_hp`.
///
/// The system is a no-op when `hp_regen` is zero or negative (the default),
/// so it has no cost for characters that haven't acquired Pummarola.
pub fn regen_hp(time: Res<Time>, mut query: Query<&mut PlayerStats, With<Player>>) {
    let Ok(mut stats) = query.single_mut() else {
        return;
    };
    if stats.current_hp <= 0.0 || stats.hp_regen <= 0.0 {
        return;
    }
    stats.current_hp = (stats.current_hp + stats.hp_regen * time.delta_secs()).min(stats.max_hp);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce as _;

    use super::*;
    use crate::states::AppState;
    use crate::types::{CharacterType, WeaponType};

    // -----------------------------------------------------------------------
    // Unit tests (pure logic, no ECS App)
    // -----------------------------------------------------------------------

    /// Diagonal input must normalise to a unit vector so movement speed is
    /// consistent in all 8 directions.
    #[test]
    fn diagonal_direction_normalises_to_unit_length() {
        let raw = Vec2::new(1.0, 1.0);
        let normalised = raw.normalize();
        let len = normalised.length();
        assert!((len - 1.0).abs() < 1e-6, "expected unit length, got {len}");
    }

    /// [`PlayerStats::default`] must match the `DEFAULT_*` fallback values so
    /// the two sources of truth stay in sync.
    #[test]
    fn player_stats_default_matches_movement_constants() {
        use crate::components::player::{DEFAULT_PLAYER_BASE_HP, DEFAULT_PLAYER_BASE_SPEED};
        let stats = PlayerStats::default();
        assert_eq!(stats.move_speed, DEFAULT_PLAYER_BASE_SPEED);
        assert_eq!(stats.max_hp, DEFAULT_PLAYER_BASE_HP);
    }

    // -----------------------------------------------------------------------
    // Integration tests (ECS App with state setup)
    // -----------------------------------------------------------------------

    fn build_playing_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
        app.init_state::<AppState>();
        app.insert_resource(SelectedCharacter::default());
        app
    }

    /// After `spawn_player` runs, exactly one entity with [`Player`] exists.
    #[test]
    fn spawn_player_creates_player_entity() {
        let mut app = build_playing_app();
        app.add_systems(Update, spawn_player);
        app.update();

        // query_filtered requires &mut World (Bevy 0.17); split the borrow.
        let mut q = app.world_mut().query_filtered::<Entity, With<Player>>();
        let count = q.iter(app.world()).count();
        assert_eq!(count, 1, "expected exactly one Player entity after spawn");
    }

    /// The spawned player entity must carry all components required by
    /// downstream gameplay systems.
    #[test]
    fn spawn_player_entity_has_required_components() {
        let mut app = build_playing_app();
        app.add_systems(Update, spawn_player);
        app.update();

        let mut q = app.world_mut().query_filtered::<Entity, With<Player>>();
        let entity = q.single(app.world()).expect("player entity should exist");

        let w = app.world();
        assert!(
            w.get::<PlayerStats>(entity).is_some(),
            "missing PlayerStats"
        );
        assert!(w.get::<Transform>(entity).is_some(), "missing Transform");
        assert!(w.get::<Sprite>(entity).is_some(), "missing Sprite");
        assert!(
            w.get::<CircleCollider>(entity).is_some(),
            "missing CircleCollider"
        );
        assert!(
            w.get::<WeaponInventory>(entity).is_some(),
            "missing WeaponInventory"
        );
        assert!(
            w.get::<PassiveInventory>(entity).is_some(),
            "missing PassiveInventory"
        );
    }

    /// The placeholder sprite must be 24×24 px.
    #[test]
    fn spawn_player_sprite_is_24px_circle() {
        let mut app = build_playing_app();
        app.add_systems(Update, spawn_player);
        app.update();

        let mut q = app.world_mut().query_filtered::<Entity, With<Player>>();
        let entity = q.single(app.world()).expect("player entity should exist");

        let sprite = app.world().get::<Sprite>(entity).expect("Sprite missing");
        assert_eq!(sprite.custom_size, Some(Vec2::splat(24.0)));
    }

    /// `player_movement` must move the player rightward when ArrowRight is pressed.
    ///
    /// Uses `run_system_once` to bypass the `First`-schedule `TimePlugin`, then
    /// manually advances `Time` so `delta_secs()` returns a non-zero value.
    #[test]
    fn player_movement_moves_right_on_arrow_right() {
        use std::time::Duration;

        let mut app = build_playing_app();

        // Spawn a player entity directly (bypassing spawn_player).
        app.world_mut().spawn((
            Player,
            PlayerStats::default(),
            PlayerFacingDirection::default(),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        // Insert the input resource with ArrowRight pressed.
        let mut input = ButtonInput::<KeyCode>::default();
        input.press(KeyCode::ArrowRight);
        app.insert_resource(input);

        // Advance Time manually BEFORE running the system so that delta_secs() > 0.
        // We bypass app.update() to avoid the TimePlugin resetting the delta.
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(1.0 / 60.0));
        app.world_mut()
            .run_system_once(player_movement)
            .expect("player_movement system should run");

        let mut q = app.world_mut().query_filtered::<Entity, With<Player>>();
        let entity = q.single(app.world()).expect("player should exist");

        let x = app
            .world()
            .get::<Transform>(entity)
            .expect("Transform missing")
            .translation
            .x;
        assert!(x > 0.0, "player should have moved right, got x = {x}");
    }

    /// Default character spawns with exactly one Whip in the weapon inventory.
    #[test]
    fn default_character_starts_with_whip() {
        let mut app = build_playing_app();
        app.add_systems(Update, spawn_player);
        app.update();

        let mut q = app.world_mut().query_filtered::<Entity, With<Player>>();
        let entity = q.single(app.world()).expect("player entity should exist");

        let inv = app
            .world()
            .get::<WeaponInventory>(entity)
            .expect("WeaponInventory missing");
        assert_eq!(
            inv.weapons.len(),
            1,
            "should have exactly one starting weapon"
        );
        assert_eq!(
            inv.weapons[0].weapon_type,
            WeaponType::Whip,
            "DefaultCharacter should start with Whip"
        );
    }

    /// Magician spawns with MagicWand and character-specific stats (lower HP, cooldown reduction).
    #[test]
    fn magician_spawns_with_magic_wand_and_correct_stats() {
        let mut app = build_playing_app();
        app.insert_resource(SelectedCharacter(CharacterType::Magician));
        app.add_systems(Update, spawn_player);
        app.update();

        let mut q = app.world_mut().query_filtered::<Entity, With<Player>>();
        let entity = q.single(app.world()).expect("player entity should exist");

        let inv = app.world().get::<WeaponInventory>(entity).unwrap();
        assert_eq!(inv.weapons[0].weapon_type, WeaponType::MagicWand);

        let stats = app.world().get::<PlayerStats>(entity).unwrap();
        assert_eq!(stats.max_hp, 80.0, "Magician max_hp must be 80");
        assert!(
            stats.cooldown_reduction > 0.0,
            "Magician must have positive cooldown_reduction"
        );
    }

    /// Thief spawns with Knife and higher move speed than the default character.
    #[test]
    fn thief_spawns_with_knife_and_higher_speed() {
        let mut app = build_playing_app();
        app.insert_resource(SelectedCharacter(CharacterType::Thief));
        app.add_systems(Update, spawn_player);
        app.update();

        let mut q = app.world_mut().query_filtered::<Entity, With<Player>>();
        let entity = q.single(app.world()).expect("player entity should exist");

        let inv = app.world().get::<WeaponInventory>(entity).unwrap();
        assert_eq!(inv.weapons[0].weapon_type, WeaponType::Knife);

        let stats = app.world().get::<PlayerStats>(entity).unwrap();
        assert_eq!(stats.max_hp, 90.0, "Thief max_hp must be 90");
        assert!(
            stats.move_speed > 200.0,
            "Thief move_speed must exceed 200 (DefaultCharacter base); got {}",
            stats.move_speed
        );
    }

    /// Knight spawns with Whip and higher HP than the default character.
    #[test]
    fn knight_spawns_with_whip_and_higher_hp() {
        let mut app = build_playing_app();
        app.insert_resource(SelectedCharacter(CharacterType::Knight));
        app.add_systems(Update, spawn_player);
        app.update();

        let mut q = app.world_mut().query_filtered::<Entity, With<Player>>();
        let entity = q.single(app.world()).expect("player entity should exist");

        let inv = app.world().get::<WeaponInventory>(entity).unwrap();
        assert_eq!(inv.weapons[0].weapon_type, WeaponType::Whip);

        let stats = app.world().get::<PlayerStats>(entity).unwrap();
        assert!(
            stats.max_hp > 100.0,
            "Knight max_hp must exceed 100 (DefaultCharacter base); got {}",
            stats.max_hp
        );
    }

    /// `current_hp` at spawn must equal `max_hp` for all characters.
    #[test]
    fn spawn_player_current_hp_equals_max_hp() {
        for char_type in [
            CharacterType::DefaultCharacter,
            CharacterType::Magician,
            CharacterType::Thief,
            CharacterType::Knight,
        ] {
            let mut app = build_playing_app();
            app.insert_resource(SelectedCharacter(char_type));
            app.add_systems(Update, spawn_player);
            app.update();

            let mut q = app.world_mut().query_filtered::<Entity, With<Player>>();
            let entity = q.single(app.world()).expect("player entity should exist");
            let stats = app.world().get::<PlayerStats>(entity).unwrap();
            assert_eq!(
                stats.current_hp, stats.max_hp,
                "{char_type:?} current_hp must equal max_hp at spawn"
            );
        }
    }

    /// Each character's starting weapon is read from `CharacterParams` (fallback
    /// values) so the `WeaponInventory` always contains the correct weapon type.
    #[test]
    fn starting_weapon_matches_character_params_fallback() {
        use crate::types::get_character_stats;
        assert_eq!(
            get_character_stats(CharacterType::DefaultCharacter).starting_weapon,
            WeaponType::Whip
        );
        assert_eq!(
            get_character_stats(CharacterType::Magician).starting_weapon,
            WeaponType::MagicWand
        );
        assert_eq!(
            get_character_stats(CharacterType::Thief).starting_weapon,
            WeaponType::Knife
        );
        assert_eq!(
            get_character_stats(CharacterType::Knight).starting_weapon,
            WeaponType::Whip
        );
    }

    /// No movement should occur when no keys are pressed.
    #[test]
    fn player_movement_no_input_stays_still() {
        use std::time::Duration;

        let mut app = build_playing_app();

        app.world_mut().spawn((
            Player,
            PlayerStats::default(),
            PlayerFacingDirection::default(),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        app.insert_resource(ButtonInput::<KeyCode>::default());

        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(1.0 / 60.0));
        app.world_mut()
            .run_system_once(player_movement)
            .expect("player_movement system should run");

        let mut q = app.world_mut().query_filtered::<Entity, With<Player>>();
        let entity = q.single(app.world()).expect("player should exist");

        let translation = app
            .world()
            .get::<Transform>(entity)
            .expect("Transform missing")
            .translation;
        assert_eq!(translation, Vec3::ZERO);
    }

    /// `regen_hp` recovers HP each frame at the configured rate, clamped to max_hp.
    #[test]
    fn regen_hp_recovers_health_over_time() {
        use std::time::Duration;

        let mut app = build_playing_app();

        let mut stats = PlayerStats::default();
        stats.current_hp = 50.0;
        stats.max_hp = 100.0;
        stats.hp_regen = 10.0; // 10 HP/s

        app.world_mut().spawn((Player, stats));

        // Advance 1 second and run the system.
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs(1));
        app.world_mut()
            .run_system_once(regen_hp)
            .expect("regen_hp should run");

        let mut q = app.world_mut().query_filtered::<Entity, With<Player>>();
        let entity = q.single(app.world()).expect("player should exist");
        let hp = app.world().get::<PlayerStats>(entity).unwrap().current_hp;
        assert!((hp - 60.0).abs() < 0.01, "expected 60.0 HP, got {hp}");
    }

    /// `regen_hp` must not exceed max_hp.
    #[test]
    fn regen_hp_clamps_to_max_hp() {
        use std::time::Duration;

        let mut app = build_playing_app();

        let mut stats = PlayerStats::default();
        stats.current_hp = 99.0;
        stats.max_hp = 100.0;
        stats.hp_regen = 10.0;

        app.world_mut().spawn((Player, stats));

        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs(1));
        app.world_mut()
            .run_system_once(regen_hp)
            .expect("regen_hp should run");

        let mut q = app.world_mut().query_filtered::<Entity, With<Player>>();
        let entity = q.single(app.world()).expect("player should exist");
        let hp = app.world().get::<PlayerStats>(entity).unwrap().current_hp;
        assert_eq!(hp, 100.0, "HP must not exceed max_hp");
    }

    /// `regen_hp` is a no-op when hp_regen is zero.
    #[test]
    fn regen_hp_zero_rate_does_nothing() {
        use std::time::Duration;

        let mut app = build_playing_app();

        let mut stats = PlayerStats::default();
        stats.current_hp = 50.0;
        stats.max_hp = 100.0;
        stats.hp_regen = 0.0;

        app.world_mut().spawn((Player, stats));

        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs(1));
        app.world_mut()
            .run_system_once(regen_hp)
            .expect("regen_hp should run");

        let mut q = app.world_mut().query_filtered::<Entity, With<Player>>();
        let entity = q.single(app.world()).expect("player should exist");
        let hp = app.world().get::<PlayerStats>(entity).unwrap().current_hp;
        assert_eq!(hp, 50.0, "HP should not change when hp_regen is 0");
    }

    /// `regen_hp` is a no-op when hp_regen is negative.
    #[test]
    fn regen_hp_negative_rate_does_nothing() {
        use std::time::Duration;

        let mut app = build_playing_app();

        let mut stats = PlayerStats::default();
        stats.current_hp = 50.0;
        stats.max_hp = 100.0;
        stats.hp_regen = -5.0;

        app.world_mut().spawn((Player, stats));

        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs(1));
        app.world_mut()
            .run_system_once(regen_hp)
            .expect("regen_hp should run");

        let mut q = app.world_mut().query_filtered::<Entity, With<Player>>();
        let entity = q.single(app.world()).expect("player should exist");
        let hp = app.world().get::<PlayerStats>(entity).unwrap().current_hp;
        assert_eq!(hp, 50.0, "HP should not change when hp_regen is negative");
    }

    /// `regen_hp` must not heal a dead player (current_hp <= 0).
    #[test]
    fn regen_hp_does_not_heal_dead_player() {
        use std::time::Duration;

        let mut app = build_playing_app();

        let mut stats = PlayerStats::default();
        stats.current_hp = 0.0;
        stats.max_hp = 100.0;
        stats.hp_regen = 10.0;

        app.world_mut().spawn((Player, stats));

        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs(1));
        app.world_mut()
            .run_system_once(regen_hp)
            .expect("regen_hp should run");

        let mut q = app.world_mut().query_filtered::<Entity, With<Player>>();
        let entity = q.single(app.world()).expect("player should exist");
        let hp = app.world().get::<PlayerStats>(entity).unwrap().current_hp;
        assert_eq!(hp, 0.0, "dead player must not be healed by regen");
    }
}
