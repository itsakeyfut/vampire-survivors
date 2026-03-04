//! Enemy AI movement system — ChasePlayer behavior.
//!
//! [`move_enemies`] runs every frame while in [`AppState::Playing`] and
//! translates each enemy entity toward the player using frame-rate-independent
//! speed scaled by [`Enemy::move_speed`].

use bevy::prelude::*;

use crate::{
    components::{Enemy, EnemyAI, Player},
    types::AIType,
};

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Moves every [`Enemy`] toward the player each frame.
///
/// - Reads the single [`Player`] transform to get the target position.
/// - For each enemy with [`AIType::ChasePlayer`], computes the normalised
///   direction and advances the transform by `move_speed × Δt` pixels.
/// - `normalize_or_zero` prevents NaN when an enemy is exactly on the player.
/// - Enemies without a player to chase remain stationary.
pub fn move_enemies(
    time: Res<Time>,
    player_q: Query<&Transform, With<Player>>,
    mut enemy_q: Query<(&Enemy, &EnemyAI, &mut Transform), Without<Player>>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();

    for (enemy, ai, mut tf) in enemy_q.iter_mut() {
        if ai.ai_type != AIType::ChasePlayer {
            continue;
        }
        let enemy_pos = tf.translation.truncate();
        let direction = (player_pos - enemy_pos).normalize_or_zero();
        tf.translation += (direction * enemy.move_speed * time.delta_secs()).extend(0.0);
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
        components::{EnemyAI, Player},
        types::{AIType, EnemyType},
    };

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app
    }

    fn advance_and_run(app: &mut App) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(1.0 / 60.0));
        app.world_mut()
            .run_system_once(move_enemies)
            .expect("move_enemies should run");
    }

    fn spawn_player_at(app: &mut App, pos: Vec2) {
        app.world_mut()
            .spawn((Player, Transform::from_translation(pos.extend(0.0))));
    }

    fn spawn_chase_enemy_at(app: &mut App, pos: Vec2) -> Entity {
        app.world_mut()
            .spawn((
                Enemy::from_type(EnemyType::Bat, 1.0),
                EnemyAI {
                    ai_type: AIType::ChasePlayer,
                    attack_timer: 0.0,
                    attack_range: 20.0,
                },
                Transform::from_translation(pos.extend(5.0)),
            ))
            .id()
    }

    // -----------------------------------------------------------------------

    /// Enemy must move closer to the player after one frame.
    #[test]
    fn enemy_moves_toward_player() {
        let mut app = build_app();
        spawn_player_at(&mut app, Vec2::new(100.0, 0.0));
        let enemy = spawn_chase_enemy_at(&mut app, Vec2::ZERO);

        advance_and_run(&mut app);

        let x = app.world().get::<Transform>(enemy).unwrap().translation.x;
        assert!(
            x > 0.0,
            "enemy should have moved toward player (x > 0), got {x}"
        );
    }

    /// Each of multiple enemies must move toward the same player.
    #[test]
    fn multiple_enemies_all_move_toward_player() {
        let mut app = build_app();
        spawn_player_at(&mut app, Vec2::new(200.0, 0.0));

        let e1 = spawn_chase_enemy_at(&mut app, Vec2::new(-100.0, 0.0));
        let e2 = spawn_chase_enemy_at(&mut app, Vec2::new(0.0, -100.0));
        let e3 = spawn_chase_enemy_at(&mut app, Vec2::new(0.0, 100.0));

        advance_and_run(&mut app);

        let w = app.world();
        let x1 = w.get::<Transform>(e1).unwrap().translation.x;
        let x2 = w.get::<Transform>(e2).unwrap().translation.x;
        let y3 = w.get::<Transform>(e3).unwrap().translation.y;

        assert!(x1 > -100.0, "e1 should move rightward, got x={x1}");
        assert!(x2 > 0.0, "e2 should move rightward, got x={x2}");
        assert!(y3 < 100.0, "e3 should move downward, got y={y3}");
    }

    /// An enemy exactly on the player must not produce NaN / must not panic.
    #[test]
    fn enemy_at_player_position_no_nan() {
        let mut app = build_app();
        spawn_player_at(&mut app, Vec2::ZERO);
        let enemy = spawn_chase_enemy_at(&mut app, Vec2::ZERO);

        advance_and_run(&mut app); // should not panic

        let t = app.world().get::<Transform>(enemy).unwrap().translation;
        assert!(
            t.x.is_finite() && t.y.is_finite(),
            "translation must be finite"
        );
    }

    /// Without a player entity, enemies must stay in place.
    #[test]
    fn no_player_enemies_stay_still() {
        let mut app = build_app();
        let enemy = spawn_chase_enemy_at(&mut app, Vec2::new(50.0, 50.0));

        advance_and_run(&mut app);

        let t = app.world().get::<Transform>(enemy).unwrap().translation;
        assert_eq!(t.x, 50.0, "enemy x should not change without a player");
        assert_eq!(t.y, 50.0, "enemy y should not change without a player");
    }

    /// Enemy movement distance must equal `move_speed × delta_secs`.
    #[test]
    fn enemy_movement_is_speed_times_delta() {
        let mut app = build_app();
        // Place player directly to the right so movement is purely horizontal.
        spawn_player_at(&mut app, Vec2::new(1000.0, 0.0));
        let enemy = spawn_chase_enemy_at(&mut app, Vec2::ZERO);

        let dt = 1.0_f32 / 60.0;
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(dt));
        app.world_mut()
            .run_system_once(move_enemies)
            .expect("move_enemies should run");

        let speed = app.world().get::<Enemy>(enemy).unwrap().move_speed;
        let x = app.world().get::<Transform>(enemy).unwrap().translation.x;
        let expected = speed * dt;
        assert!(
            (x - expected).abs() < 1e-4,
            "expected x ≈ {expected}, got {x}"
        );
    }

    /// Enemies with non-ChasePlayer AI must not move.
    #[test]
    fn non_chase_enemy_stays_still() {
        let mut app = build_app();
        spawn_player_at(&mut app, Vec2::new(100.0, 0.0));

        let enemy = app
            .world_mut()
            .spawn((
                Enemy::from_type(EnemyType::Medusa, 1.0),
                EnemyAI {
                    ai_type: AIType::KeepDistance,
                    attack_timer: 0.0,
                    attack_range: 150.0,
                },
                Transform::from_xyz(0.0, 0.0, 5.0),
            ))
            .id();

        advance_and_run(&mut app);

        let t = app.world().get::<Transform>(enemy).unwrap().translation;
        assert_eq!(t.x, 0.0, "KeepDistance enemy must not chase");
    }
}
