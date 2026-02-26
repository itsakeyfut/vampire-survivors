//! Whip weapon — fan-shaped instant-hit melee attack.
//!
//! The Whip swings in a wide arc to the player's left or right, alternating
//! each activation.  Every enemy inside the fan receives a
//! [`DamageEnemyEvent`]; the [`apply_damage`](super::damage::apply_damage)
//! system then reduces their HP.
//!
//! ## Fan-shaped hitbox
//!
//! An enemy at relative position `rel = enemy_pos − player_pos` is inside the
//! fan when **all three** conditions hold:
//!
//! ```text
//! rel.x * direction > 0          (correct horizontal side)
//! rel.length()      < range      (within reach)
//! rel.y.abs()       < range * 0.6 (within vertical spread ≈ ±34°)
//! ```
//!
//! where `direction = +1.0` (right) or `−1.0` (left).

use bevy::{prelude::*, state::state_scoped::DespawnOnExit};

use crate::{
    components::{Enemy, Player, PlayerStats, PlayerWhipSide},
    events::{DamageEnemyEvent, WeaponFiredEvent},
    resources::SpatialGrid,
    states::AppState,
    types::{WeaponType, WhipSide},
};

// ---------------------------------------------------------------------------
// Fallback constants (used when RON config is not yet loaded)
// ---------------------------------------------------------------------------

/// Reach of the Whip in pixels (before area_multiplier is applied).
const DEFAULT_WHIP_RANGE: f32 = 160.0;
/// Base damage of the Whip at level 1.
const DEFAULT_WHIP_BASE_DAMAGE: f32 = 20.0;
/// Additional damage per weapon level above 1.
const DEFAULT_WHIP_DAMAGE_PER_LEVEL: f32 = 10.0;
/// How long the swing visual stays on screen (seconds).
const DEFAULT_WHIP_EFFECT_DURATION: f32 = 0.15;

// ---------------------------------------------------------------------------
// Whip swing effect component
// ---------------------------------------------------------------------------

/// Marks a short-lived swing sprite spawned when the Whip activates.
///
/// [`despawn_whip_effects`] removes these entities when `remaining` reaches
/// zero.
#[derive(Component, Debug)]
pub struct WhipSwingEffect {
    /// Seconds remaining before this entity is despawned.
    pub remaining: f32,
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

/// Computes raw Whip damage for a given level (before player multiplier).
fn whip_damage_for_level(level: u8) -> f32 {
    DEFAULT_WHIP_BASE_DAMAGE + DEFAULT_WHIP_DAMAGE_PER_LEVEL * (level.clamp(1, 8) as f32 - 1.0)
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Activates the Whip when a [`WeaponFiredEvent`] arrives for
/// [`WeaponType::Whip`] or [`WeaponType::BloodyTear`].
///
/// - Uses [`SpatialGrid`] to find candidate enemies within `range` before
///   performing the exact fan-shaped hitbox check.
/// - Emits one [`DamageEnemyEvent`] per hit enemy.
/// - Spawns a [`WhipSwingEffect`] sprite as visual feedback.
/// - Flips [`PlayerWhipSide`] so the next swing is on the opposite side.
///
/// Must run after [`super::spatial::update_spatial_grid`] so the grid
/// reflects the current frame's enemy positions.
pub fn fire_whip(
    mut fired_events: MessageReader<WeaponFiredEvent>,
    mut damage_events: MessageWriter<DamageEnemyEvent>,
    mut player_q: Query<(&Transform, &PlayerStats, &mut PlayerWhipSide), With<Player>>,
    enemy_q: Query<&Transform, With<Enemy>>,
    spatial_grid: Res<SpatialGrid>,
    mut commands: Commands,
) {
    for event in fired_events.read() {
        if event.weapon_type != WeaponType::Whip && event.weapon_type != WeaponType::BloodyTear {
            continue;
        }

        let Ok((player_tf, stats, mut whip_side)) = player_q.get_mut(event.player) else {
            continue;
        };

        let player_pos = player_tf.translation.truncate();
        let range = DEFAULT_WHIP_RANGE * stats.area_multiplier;
        let damage = whip_damage_for_level(event.level) * stats.damage_multiplier;
        let direction = if whip_side.0 == WhipSide::Right {
            1.0_f32
        } else {
            -1.0_f32
        };

        // Use SpatialGrid to narrow candidates, then apply exact fan check.
        for enemy_entity in spatial_grid.get_nearby(player_pos, range) {
            let Ok(enemy_tf) = enemy_q.get(enemy_entity) else {
                continue;
            };
            let rel = enemy_tf.translation.truncate() - player_pos;
            if rel.x * direction > 0.0 && rel.length() < range && rel.y.abs() < range * 0.6 {
                damage_events.write(DamageEnemyEvent {
                    entity: enemy_entity,
                    damage,
                    weapon_type: event.weapon_type,
                });
            }
        }

        // Spawn a short-lived colored rectangle as visual feedback.
        let effect_x = player_pos.x + direction * range * 0.5;
        commands.spawn((
            WhipSwingEffect {
                remaining: DEFAULT_WHIP_EFFECT_DURATION,
            },
            Sprite {
                color: Color::srgba(0.9, 0.2, 0.3, 0.6),
                custom_size: Some(Vec2::new(range, range * 0.6)),
                ..default()
            },
            Transform::from_xyz(effect_x, player_pos.y, 4.0),
            DespawnOnExit(AppState::Playing),
        ));

        // Alternate side for next activation.
        whip_side.0 = whip_side.0.flip();
    }
}

/// Ticks down [`WhipSwingEffect`] lifetimes and despawns expired entities.
pub fn despawn_whip_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut effect_q: Query<(Entity, &mut WhipSwingEffect)>,
) {
    let delta = time.delta_secs();
    for (entity, mut effect) in effect_q.iter_mut() {
        effect.remaining -= delta;
        if effect.remaining <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::ecs::system::RunSystemOnce as _;

    use super::*;
    use crate::{
        components::WeaponInventory,
        events::WeaponFiredEvent,
        resources::SpatialGrid,
        types::{WeaponState, WeaponType, WhipSide},
    };

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<WeaponFiredEvent>();
        app.add_message::<DamageEnemyEvent>();
        app.insert_resource(SpatialGrid::default());
        app
    }

    fn advance(app: &mut App, secs: f32) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(secs));
    }

    /// Spawns a player with a Whip weapon (timer=0 so it fires immediately)
    /// and the given whip side.
    fn spawn_player(app: &mut App, side: WhipSide) -> Entity {
        let mut weapon = WeaponState::new(WeaponType::Whip);
        weapon.cooldown_timer = 0.0;
        app.world_mut()
            .spawn((
                Player,
                PlayerStats::default(),
                WeaponInventory {
                    weapons: vec![weapon],
                },
                PlayerWhipSide(side),
                Transform::from_xyz(0.0, 0.0, 10.0),
            ))
            .id()
    }

    fn spawn_enemy(app: &mut App, pos: Vec2) -> Entity {
        app.world_mut()
            .spawn((
                Enemy::from_type(crate::types::EnemyType::Bat, 1.0),
                Transform::from_xyz(pos.x, pos.y, 1.0),
            ))
            .id()
    }

    /// Fires WeaponFiredEvent via tick_weapon_cooldowns, then runs fire_whip.
    ///
    /// Populates the [`SpatialGrid`] between the two steps so that
    /// `fire_whip` can use `get_nearby` for enemy candidate selection.
    fn tick_and_fire(app: &mut App) {
        use crate::systems::{
            spatial::update_spatial_grid, weapon_cooldown::tick_weapon_cooldowns,
        };
        advance(app, 1.0 / 60.0);
        app.world_mut()
            .run_system_once(tick_weapon_cooldowns)
            .expect("tick_weapon_cooldowns should run");
        app.world_mut()
            .run_system_once(update_spatial_grid)
            .expect("update_spatial_grid should run");
        app.world_mut()
            .run_system_once(fire_whip)
            .expect("fire_whip should run");
    }

    fn damage_events(app: &App) -> Vec<DamageEnemyEvent> {
        let messages = app.world().resource::<Messages<DamageEnemyEvent>>();
        let mut cursor = messages.get_cursor();
        cursor.read(messages).cloned().collect()
    }

    // -----------------------------------------------------------------------
    // Tests
    // -----------------------------------------------------------------------

    /// Enemy on the correct (right) side, within range — must be hit.
    #[test]
    fn enemy_in_whip_range_takes_damage() {
        let mut app = build_app();
        spawn_player(&mut app, WhipSide::Right);
        spawn_enemy(&mut app, Vec2::new(100.0, 0.0)); // to the right, within 160 px

        tick_and_fire(&mut app);

        let events = damage_events(&app);
        assert_eq!(events.len(), 1, "one enemy should have been hit");
        assert!(events[0].damage > 0.0);
        assert_eq!(events[0].weapon_type, WeaponType::Whip);
    }

    /// Enemy on the wrong (left) side when swinging right — must not be hit.
    #[test]
    fn enemy_on_wrong_side_not_hit() {
        let mut app = build_app();
        spawn_player(&mut app, WhipSide::Right); // swings right
        spawn_enemy(&mut app, Vec2::new(-100.0, 0.0)); // to the left

        tick_and_fire(&mut app);

        assert_eq!(
            damage_events(&app).len(),
            0,
            "enemy on wrong side should not be hit"
        );
    }

    /// Enemy beyond the range limit — must not be hit.
    #[test]
    fn enemy_beyond_range_not_hit() {
        let mut app = build_app();
        spawn_player(&mut app, WhipSide::Right);
        spawn_enemy(&mut app, Vec2::new(200.0, 0.0)); // 200 px > DEFAULT_WHIP_RANGE (160)

        tick_and_fire(&mut app);

        assert_eq!(
            damage_events(&app).len(),
            0,
            "enemy beyond range should not be hit"
        );
    }

    /// Enemy outside the vertical spread (rel.y.abs() >= range * 0.6) — not hit.
    #[test]
    fn enemy_outside_vertical_spread_not_hit() {
        let mut app = build_app();
        spawn_player(&mut app, WhipSide::Right);
        // range * 0.6 = 160 * 0.6 = 96; place enemy at y = 110
        spawn_enemy(&mut app, Vec2::new(50.0, 110.0));

        tick_and_fire(&mut app);

        assert_eq!(
            damage_events(&app).len(),
            0,
            "enemy outside vertical spread should not be hit"
        );
    }

    /// Multiple enemies: only those inside the fan are hit.
    #[test]
    fn only_enemies_in_fan_are_hit() {
        let mut app = build_app();
        spawn_player(&mut app, WhipSide::Right);
        spawn_enemy(&mut app, Vec2::new(80.0, 0.0)); // inside fan
        spawn_enemy(&mut app, Vec2::new(-80.0, 0.0)); // wrong side
        spawn_enemy(&mut app, Vec2::new(250.0, 0.0)); // out of range

        tick_and_fire(&mut app);

        assert_eq!(
            damage_events(&app).len(),
            1,
            "only the enemy inside the fan should be hit"
        );
    }

    /// WhipSide flips from Right to Left after each activation.
    #[test]
    fn whip_side_flips_after_activation() {
        let mut app = build_app();
        let player = spawn_player(&mut app, WhipSide::Right);

        tick_and_fire(&mut app);

        let side = app
            .world()
            .entity(player)
            .get::<PlayerWhipSide>()
            .expect("PlayerWhipSide should exist")
            .0;
        assert_eq!(
            side,
            WhipSide::Left,
            "side should flip to Left after Right swing"
        );
    }

    /// Whip fires on the left side when PlayerWhipSide is Left.
    #[test]
    fn whip_hits_left_when_side_is_left() {
        let mut app = build_app();
        spawn_player(&mut app, WhipSide::Left);
        spawn_enemy(&mut app, Vec2::new(-100.0, 0.0)); // to the left

        tick_and_fire(&mut app);

        assert_eq!(
            damage_events(&app).len(),
            1,
            "enemy on left should be hit when WhipSide is Left"
        );
    }

    /// No enemies → no damage events.
    #[test]
    fn no_enemies_no_damage_events() {
        let mut app = build_app();
        spawn_player(&mut app, WhipSide::Right);

        tick_and_fire(&mut app);

        assert_eq!(damage_events(&app).len(), 0);
    }

    /// Damage scales with weapon level: level 2 deals more than level 1.
    #[test]
    fn damage_scales_with_level() {
        assert!(
            whip_damage_for_level(2) > whip_damage_for_level(1),
            "level 2 should deal more damage than level 1"
        );
        assert!(
            whip_damage_for_level(8) > whip_damage_for_level(1),
            "level 8 should deal the most damage"
        );
    }

    /// WhipSwingEffect despawns after its lifetime expires.
    #[test]
    fn whip_effect_despawns_after_lifetime() {
        let mut app = build_app();
        spawn_player(&mut app, WhipSide::Right);

        tick_and_fire(&mut app);

        // Effect should exist immediately after firing.
        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<WhipSwingEffect>>();
        let count_before = q.iter(app.world()).count();
        assert_eq!(count_before, 1, "swing effect should be spawned");

        // Advance past the effect duration and run the despawn system once.
        // Using run_system_once instead of app.update() avoids the TimePlugin
        // in MinimalPlugins resetting the manually-advanced delta in First.
        advance(&mut app, DEFAULT_WHIP_EFFECT_DURATION + 0.01);
        app.world_mut()
            .run_system_once(despawn_whip_effects)
            .expect("despawn_whip_effects should run");

        let mut q2 = app
            .world_mut()
            .query_filtered::<Entity, With<WhipSwingEffect>>();
        assert_eq!(
            q2.iter(app.world()).count(),
            0,
            "swing effect should be despawned after lifetime"
        );
    }
}
