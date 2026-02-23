use bevy::{prelude::*, state::state_scoped::DespawnOnExit};

use crate::{
    components::{CircleCollider, PassiveInventory, Player, PlayerStats, WeaponInventory},
    states::AppState,
};

// ---------------------------------------------------------------------------
// Spawn
// ---------------------------------------------------------------------------

/// Spawns the player entity and a 2D camera when entering [`AppState::Playing`].
///
/// Both entities carry [`DespawnOnExit`] so they are automatically despawned
/// when the game leaves the `Playing` state — no explicit cleanup system is
/// needed.
///
/// The camera is positioned at the origin for now; issue #8 (task 2.3) will
/// make it smoothly follow the player.
pub fn spawn_player(mut commands: Commands) {
    // Static 2D camera — will be enhanced to follow the player in issue #8.
    commands.spawn((DespawnOnExit(AppState::Playing), Camera2d));

    // Player entity: cyan circle sprite + all required ECS components.
    commands.spawn((
        DespawnOnExit(AppState::Playing),
        Player,
        PlayerStats::default(),
        Sprite {
            color: Color::srgb(0.2, 0.8, 1.0),
            custom_size: Some(Vec2::splat(24.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 10.0),
        CircleCollider { radius: 12.0 },
        WeaponInventory::default(),
        PassiveInventory::default(),
    ));
}

// ---------------------------------------------------------------------------
// Movement
// ---------------------------------------------------------------------------

/// Moves the player based on WASD / arrow-key input.
///
/// - Input from all four cardinal directions is summed then normalised so that
///   diagonal movement is not faster than axis-aligned movement.
/// - Movement is frame-rate independent: distance = speed × Δt.
pub fn player_movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &PlayerStats), With<Player>>,
) {
    let Ok((mut transform, stats)) = query.single_mut() else {
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
        let delta = direction.normalize() * stats.move_speed * time.delta_secs();
        transform.translation += delta.extend(0.0);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce as _;

    use super::*;
    use crate::constants::{PLAYER_BASE_HP, PLAYER_BASE_SPEED};

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

    /// [`PlayerStats::default`] must match the movement-related constants so
    /// the two sources of truth stay in sync.
    #[test]
    fn player_stats_default_matches_movement_constants() {
        let stats = PlayerStats::default();
        assert_eq!(stats.move_speed, PLAYER_BASE_SPEED);
        assert_eq!(stats.max_hp, PLAYER_BASE_HP);
    }

    // -----------------------------------------------------------------------
    // Integration tests (ECS App with state setup)
    // -----------------------------------------------------------------------

    fn build_playing_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
        app.init_state::<AppState>();
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

    /// No movement should occur when no keys are pressed.
    #[test]
    fn player_movement_no_input_stays_still() {
        use std::time::Duration;

        let mut app = build_playing_app();

        app.world_mut().spawn((
            Player,
            PlayerStats::default(),
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
}
