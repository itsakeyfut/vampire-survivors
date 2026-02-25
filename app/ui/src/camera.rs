use bevy::prelude::*;
use vs_core::components::Player;
use vs_core::config::GameParams;

// ---------------------------------------------------------------------------
// Fallback constants (used when RON config is not yet loaded)
// ---------------------------------------------------------------------------

/// Camera follow speed; higher = tighter/faster follow.
const DEFAULT_CAMERA_LERP_SPEED: f32 = 10.0;

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

/// Spawns the single orthographic [`Camera2d`] used to render the game world.
///
/// Registered at [`Startup`] so the camera exists before any state transition,
/// making it available for title / menu rendering as well as gameplay.
/// One world unit equals one logical pixel (scale = 1.0).
pub fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Transform::from_xyz(0.0, 0.0, 999.9)));
}

// ---------------------------------------------------------------------------
// Follow
// ---------------------------------------------------------------------------

/// Smoothly moves the camera toward the player's position each frame.
///
/// Uses exponential lerp (`current.lerp(target, speed × Δt)`) so the camera
/// closes the gap quickly when far away and decelerates as it catches up.
/// Speed is read from [`GameParams`], falling back to [`DEFAULT_CAMERA_LERP_SPEED`].
///
/// Only runs while in [`AppState::Playing`] (registered by [`GameUIPlugin`]).
pub fn camera_follow_player(
    time: Res<Time>,
    player_q: Query<&Transform, With<Player>>,
    mut camera_q: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    game_cfg: GameParams,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let Ok(mut camera_tf) = camera_q.single_mut() else {
        return;
    };

    let lerp_speed = game_cfg
        .get()
        .map(|c| c.camera_lerp_speed)
        .unwrap_or(DEFAULT_CAMERA_LERP_SPEED);

    let target = player_tf.translation.truncate();
    let current = camera_tf.translation.truncate();
    let lerped = current.lerp(target, lerp_speed * time.delta_secs());

    camera_tf.translation.x = lerped.x;
    camera_tf.translation.y = lerped.y;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce as _;

    use super::*;

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app
    }

    /// `setup_camera` must spawn exactly one `Camera2d`.
    #[test]
    fn setup_camera_spawns_one_camera() {
        let mut app = build_app();
        app.add_systems(Startup, setup_camera);
        app.update();

        let mut q = app.world_mut().query_filtered::<Entity, With<Camera2d>>();
        assert_eq!(q.iter(app.world()).count(), 1);
    }

    /// Camera `z` must be 999.9 so all game sprites (z < 999) are visible.
    #[test]
    fn setup_camera_z_is_999() {
        let mut app = build_app();
        app.add_systems(Startup, setup_camera);
        app.update();

        let mut q = app.world_mut().query::<(&Camera2d, &Transform)>();
        let (_, tf) = q.single(app.world()).expect("camera should exist");
        assert_eq!(tf.translation.z, 999.9);
    }

    /// `camera_follow_player` must move the camera toward the player's position.
    #[test]
    fn camera_follow_moves_toward_player() {
        use std::time::Duration;

        let mut app = build_app();

        // Camera at origin, player at (200, 0).
        app.world_mut()
            .spawn((Camera2d, Transform::from_xyz(0.0, 0.0, 999.9)));
        app.world_mut()
            .spawn((Player, Transform::from_xyz(200.0, 0.0, 10.0)));

        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(1.0 / 60.0));
        app.world_mut()
            .run_system_once(camera_follow_player)
            .expect("system should run");

        let mut q = app
            .world_mut()
            .query_filtered::<&Transform, With<Camera2d>>();
        let camera_x = q
            .single(app.world())
            .expect("camera should exist")
            .translation
            .x;

        assert!(
            camera_x > 0.0,
            "camera should have moved toward player, got x = {camera_x}"
        );
        assert!(
            camera_x < 200.0,
            "camera should not overshoot, got x = {camera_x}"
        );
    }

    /// When there is no player, the camera must stay still (no panic).
    #[test]
    fn camera_follow_no_player_is_noop() {
        use std::time::Duration;

        let mut app = build_app();
        app.world_mut()
            .spawn((Camera2d, Transform::from_xyz(0.0, 0.0, 999.9)));

        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(1.0 / 60.0));
        app.world_mut()
            .run_system_once(camera_follow_player)
            .expect("system should not panic without a player");

        let mut q = app
            .world_mut()
            .query_filtered::<&Transform, With<Camera2d>>();
        let tf = q.single(app.world()).expect("camera should exist");
        assert_eq!(tf.translation.x, 0.0);
        assert_eq!(tf.translation.y, 0.0);
    }
}
