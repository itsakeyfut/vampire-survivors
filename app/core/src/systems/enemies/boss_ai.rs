//! Boss AI systems — movement and phase-transition logic.
//!
//! | Phase     | HP threshold | Behavior                  |
//! |-----------|--------------|---------------------------|
//! | `Phase1`  | 100% – 60%   | Chase player at speed 30  |
//! | `Phase2`  | < 60%        | Implemented in future issue |
//!
//! ## System ordering
//!
//! Both systems run each frame in [`AppState::Playing`]:
//!
//! - [`move_boss_phase1`] runs after `player_movement` so the boss always
//!   aims at the player's updated position.
//! - [`check_boss_phase_transition`] runs after [`move_boss_phase1`]; once it
//!   flips the phase the move system automatically stops chasing.

use bevy::prelude::*;

use crate::{
    components::{Enemy, Player},
    types::BossPhase,
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// HP fraction below which Phase1 transitions to Phase2.
const PHASE2_HP_THRESHOLD: f32 = 0.6;

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Moves Boss Death toward the player while in [`BossPhase::Phase1`].
///
/// Uses `enemy.move_speed` (30 px/s by default from `enemy.ron`) so the boss
/// always respects its configured speed.  The system is a no-op when no player
/// entity exists or when the boss is in any phase other than Phase1.
pub fn move_boss_phase1(
    time: Res<Time>,
    player_q: Query<&Transform, With<Player>>,
    mut boss_q: Query<(&Enemy, &mut Transform, &BossPhase), Without<Player>>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    let dt = time.delta_secs();

    for (enemy, mut boss_tf, phase) in boss_q.iter_mut() {
        if *phase != BossPhase::Phase1 {
            continue;
        }
        let boss_pos = boss_tf.translation.truncate();
        let direction = (player_pos - boss_pos).normalize_or_zero();
        boss_tf.translation += (direction * enemy.move_speed * dt).extend(0.0);
    }
}

/// Transitions Boss Death from Phase1 to Phase2 when HP drops below 60%.
///
/// Runs every frame; is a cheap no-op once the phase has already advanced.
/// Phase2 behavior (increased speed, additional attacks) is implemented
/// in a future issue.
pub fn check_boss_phase_transition(mut boss_q: Query<(&Enemy, &mut BossPhase)>) {
    for (enemy, mut phase) in boss_q.iter_mut() {
        if *phase == BossPhase::Phase1 && enemy.current_hp < enemy.max_hp * PHASE2_HP_THRESHOLD {
            *phase = BossPhase::Phase2;
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
        components::{Enemy, Player},
        types::{BossPhase, EnemyType},
    };

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app
    }

    fn advance(app: &mut App, secs: f32) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(secs));
    }

    fn spawn_boss(app: &mut App, hp: f32, max_hp: f32, pos: Vec2) -> Entity {
        let mut enemy = Enemy::from_type(EnemyType::BossDeath, 1.0);
        enemy.current_hp = hp;
        enemy.max_hp = max_hp;
        app.world_mut()
            .spawn((
                enemy,
                BossPhase::Phase1,
                Transform::from_xyz(pos.x, pos.y, 0.0),
            ))
            .id()
    }

    fn spawn_player(app: &mut App, pos: Vec2) -> Entity {
        app.world_mut()
            .spawn((Player, Transform::from_xyz(pos.x, pos.y, 0.0)))
            .id()
    }

    // -----------------------------------------------------------------------
    // move_boss_phase1
    // -----------------------------------------------------------------------

    /// Phase1 boss moves toward the player each frame.
    #[test]
    fn phase1_boss_moves_toward_player() {
        let mut app = build_app();
        // Boss at (-500, 0), player at (0, 0) — boss should move right (+x).
        let boss = spawn_boss(&mut app, 5000.0, 5000.0, Vec2::new(-500.0, 0.0));
        spawn_player(&mut app, Vec2::ZERO);

        advance(&mut app, 1.0 / 60.0);
        app.world_mut().run_system_once(move_boss_phase1).unwrap();

        let x = app.world().get::<Transform>(boss).unwrap().translation.x;
        assert!(x > -500.0, "boss should have moved toward player; x = {x}");
    }

    /// Phase1 boss uses its configured move_speed.
    #[test]
    fn phase1_boss_respects_move_speed() {
        let mut app = build_app();
        // Boss far to the left so direction is nearly pure +x.
        let boss = spawn_boss(&mut app, 5000.0, 5000.0, Vec2::new(-10000.0, 0.0));
        spawn_player(&mut app, Vec2::ZERO);

        let dt = 1.0_f32;
        advance(&mut app, dt);
        app.world_mut().run_system_once(move_boss_phase1).unwrap();

        let x = app.world().get::<Transform>(boss).unwrap().translation.x;
        let expected_speed = Enemy::from_type(EnemyType::BossDeath, 1.0).move_speed;
        let moved = x - (-10000.0);
        assert!(
            (moved - expected_speed * dt).abs() < 0.1,
            "boss should move at move_speed ({expected_speed} px/s); moved {moved}"
        );
    }

    /// Phase2 boss is not moved by move_boss_phase1.
    #[test]
    fn phase2_boss_not_moved_by_phase1_system() {
        let mut app = build_app();
        let boss = spawn_boss(&mut app, 1000.0, 5000.0, Vec2::new(-500.0, 0.0));
        // Switch to Phase2 manually.
        app.world_mut().entity_mut(boss).insert(BossPhase::Phase2);
        spawn_player(&mut app, Vec2::ZERO);

        advance(&mut app, 1.0 / 60.0);
        app.world_mut().run_system_once(move_boss_phase1).unwrap();

        let x = app.world().get::<Transform>(boss).unwrap().translation.x;
        assert_eq!(x, -500.0, "Phase2 boss must not be moved by Phase1 system");
    }

    /// No player → boss stays stationary.
    #[test]
    fn no_player_boss_stays_still() {
        let mut app = build_app();
        let boss = spawn_boss(&mut app, 5000.0, 5000.0, Vec2::new(-200.0, 0.0));

        advance(&mut app, 1.0 / 60.0);
        app.world_mut().run_system_once(move_boss_phase1).unwrap();

        let x = app.world().get::<Transform>(boss).unwrap().translation.x;
        assert_eq!(x, -200.0, "boss should stay still without a player");
    }

    // -----------------------------------------------------------------------
    // check_boss_phase_transition
    // -----------------------------------------------------------------------

    /// Phase1 stays Phase1 while HP is above 60%.
    #[test]
    fn phase1_stays_while_hp_above_threshold() {
        let mut app = build_app();
        let boss = spawn_boss(&mut app, 3001.0, 5000.0, Vec2::ZERO); // 60.02%

        app.world_mut()
            .run_system_once(check_boss_phase_transition)
            .unwrap();

        assert_eq!(
            *app.world().get::<BossPhase>(boss).unwrap(),
            BossPhase::Phase1,
            "should remain Phase1 above threshold"
        );
    }

    /// Phase1 transitions to Phase2 when HP drops below 60%.
    #[test]
    fn transitions_to_phase2_below_threshold() {
        let mut app = build_app();
        let boss = spawn_boss(&mut app, 2999.0, 5000.0, Vec2::ZERO); // 59.98%

        app.world_mut()
            .run_system_once(check_boss_phase_transition)
            .unwrap();

        assert_eq!(
            *app.world().get::<BossPhase>(boss).unwrap(),
            BossPhase::Phase2,
            "should transition to Phase2 below threshold"
        );
    }

    /// Phase2 is not reverted to Phase1 (one-way transition).
    #[test]
    fn phase2_is_not_reverted() {
        let mut app = build_app();
        let boss = spawn_boss(&mut app, 2999.0, 5000.0, Vec2::ZERO);
        app.world_mut().entity_mut(boss).insert(BossPhase::Phase2);

        app.world_mut()
            .run_system_once(check_boss_phase_transition)
            .unwrap();

        assert_eq!(
            *app.world().get::<BossPhase>(boss).unwrap(),
            BossPhase::Phase2,
            "Phase2 must not revert to Phase1"
        );
    }

    /// Exactly at the 60% threshold stays Phase1 (< not <=).
    #[test]
    fn exact_threshold_stays_phase1() {
        let mut app = build_app();
        let boss = spawn_boss(&mut app, 3000.0, 5000.0, Vec2::ZERO); // exactly 60%

        app.world_mut()
            .run_system_once(check_boss_phase_transition)
            .unwrap();

        assert_eq!(
            *app.world().get::<BossPhase>(boss).unwrap(),
            BossPhase::Phase1,
            "exactly 60% HP should remain Phase1 (threshold is exclusive)"
        );
    }
}
