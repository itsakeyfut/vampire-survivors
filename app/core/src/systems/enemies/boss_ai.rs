//! Boss AI systems — movement and phase-transition logic.
//!
//! | Phase     | HP threshold | Behavior                                     |
//! |-----------|--------------|----------------------------------------------|
//! | `Phase1`  | 100% – 60%   | Chase player at base speed (30 px/s)         |
//! | `Phase2`  | 60% – 30%    | Chase at 1.5× speed, 3 Mini Deaths summoned  |
//! | `Phase3`  | < 30%        | Implemented in future issue                  |
//!
//! ## System ordering
//!
//! All systems run each frame in [`AppState::Playing`]:
//!
//! - [`move_boss_phase1`] / [`move_boss_phase2`] run after `player_movement`
//!   so the boss always aims at the player's updated position.
//! - [`check_boss_phase_transition`] runs after the move systems; once it
//!   flips the phase, the corresponding move system stops and the next starts.

use bevy::prelude::*;

use crate::{
    components::{CircleCollider, Enemy, EnemyAI, GameSessionEntity, Player},
    config::{EnemyConfig, EnemyParams},
    types::{AIType, BossPhase, EnemyType},
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// HP fraction below which Phase1 transitions to Phase2.
const PHASE2_HP_THRESHOLD: f32 = 0.6;
/// HP fraction below which Phase2 transitions to Phase3.
const PHASE3_HP_THRESHOLD: f32 = 0.3;
/// Speed multiplier applied to the boss's base `move_speed` in Phase2.
const PHASE2_SPEED_MULTIPLIER: f32 = 1.5;
/// Number of Mini Deaths summoned when the boss enters Phase2.
const MINI_DEATH_SPAWN_COUNT: usize = 3;
/// Radial offset from the boss center when placing each Mini Death (pixels).
const MINI_DEATH_SPAWN_RADIUS: f32 = 80.0;
/// Collider radius for Mini Death entities (pixels).
const MINI_DEATH_COLLIDER: f32 = 20.0;

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

/// Moves Boss Death toward the player while in [`BossPhase::Phase2`].
///
/// Applies [`PHASE2_SPEED_MULTIPLIER`] (1.5×) to `enemy.move_speed`, raising
/// the effective speed from 30 to 45 px/s.  The system is a no-op when no
/// player entity exists or when the boss is in any phase other than Phase2.
pub fn move_boss_phase2(
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
        if *phase != BossPhase::Phase2 {
            continue;
        }
        let boss_pos = boss_tf.translation.truncate();
        let direction = (player_pos - boss_pos).normalize_or_zero();
        boss_tf.translation +=
            (direction * enemy.move_speed * PHASE2_SPEED_MULTIPLIER * dt).extend(0.0);
    }
}

/// Monitors boss HP and advances phase when thresholds are crossed.
///
/// - Phase1 → Phase2 at HP < 60%: increases speed and summons 3 Mini Deaths.
/// - Phase2 → Phase3 at HP < 30%: implemented in a future issue.
///
/// Runs every frame; each transition fires exactly once (guarded by the
/// current phase check).
pub fn check_boss_phase_transition(
    mut commands: Commands,
    mut boss_q: Query<(&Enemy, &mut BossPhase, &Transform)>,
    enemy_cfg: EnemyParams,
) {
    for (enemy, mut phase, boss_tf) in boss_q.iter_mut() {
        match *phase {
            BossPhase::Phase1 if enemy.current_hp < enemy.max_hp * PHASE2_HP_THRESHOLD => {
                *phase = BossPhase::Phase2;
                spawn_mini_deaths(
                    &mut commands,
                    boss_tf.translation.truncate(),
                    enemy_cfg.get(),
                );
            }
            BossPhase::Phase2 if enemy.current_hp < enemy.max_hp * PHASE3_HP_THRESHOLD => {
                *phase = BossPhase::Phase3;
                // Phase3 behavior implemented in a future issue.
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Spawns [`MINI_DEATH_SPAWN_COUNT`] Mini Death entities evenly distributed
/// around `boss_pos` at radius [`MINI_DEATH_SPAWN_RADIUS`].
///
/// Uses `Enemy::from_config()` when `cfg` is available so that hot-reloading
/// `enemy.ron` takes effect.  Falls back to compile-time constants otherwise.
fn spawn_mini_deaths(commands: &mut Commands, boss_pos: Vec2, cfg: Option<&EnemyConfig>) {
    let (enemy, collider_radius) = match cfg {
        Some(c) => {
            let stats = c.stats_for(EnemyType::MiniDeath);
            let collider = stats.collider_radius;
            (
                Enemy::from_config(EnemyType::MiniDeath, stats, 1.0),
                collider,
            )
        }
        None => (
            Enemy::from_type(EnemyType::MiniDeath, 1.0),
            MINI_DEATH_COLLIDER,
        ),
    };

    let angle_step = std::f32::consts::TAU / MINI_DEATH_SPAWN_COUNT as f32;
    for i in 0..MINI_DEATH_SPAWN_COUNT {
        let angle = i as f32 * angle_step;
        let offset = Vec2::new(angle.cos(), angle.sin()) * MINI_DEATH_SPAWN_RADIUS;
        let spawn_pos = boss_pos + offset;

        commands.spawn((
            GameSessionEntity,
            enemy.clone(),
            EnemyAI {
                ai_type: AIType::ChasePlayer,
                attack_timer: 0.0,
                attack_range: 0.0,
            },
            CircleCollider {
                radius: collider_radius,
            },
            // Dark purple placeholder sprite to distinguish Mini Deaths from the boss.
            Sprite {
                color: Color::srgb(0.7, 0.1, 0.7),
                custom_size: Some(Vec2::splat(collider_radius * 2.0)),
                ..default()
            },
            Transform::from_xyz(spawn_pos.x, spawn_pos.y, 5.0),
        ));
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
    // move_boss_phase2
    // -----------------------------------------------------------------------

    /// Phase2 boss moves toward the player faster than Phase1.
    #[test]
    fn phase2_boss_moves_faster_than_phase1() {
        let mut app1 = build_app();
        let boss1 = spawn_boss(&mut app1, 2000.0, 5000.0, Vec2::new(-10000.0, 0.0));
        spawn_player(&mut app1, Vec2::ZERO);

        let mut app2 = build_app();
        let boss2 = spawn_boss(&mut app2, 2000.0, 5000.0, Vec2::new(-10000.0, 0.0));
        app2.world_mut().entity_mut(boss2).insert(BossPhase::Phase2);
        spawn_player(&mut app2, Vec2::ZERO);

        let dt = 1.0_f32;
        advance(&mut app1, dt);
        advance(&mut app2, dt);

        app1.world_mut().run_system_once(move_boss_phase1).unwrap();
        app2.world_mut().run_system_once(move_boss_phase2).unwrap();

        let x1 = app1.world().get::<Transform>(boss1).unwrap().translation.x;
        let x2 = app2.world().get::<Transform>(boss2).unwrap().translation.x;
        assert!(
            x2 > x1,
            "Phase2 boss should move farther than Phase1 in the same time; x1={x1} x2={x2}"
        );
    }

    /// Phase2 boss applies the 1.5× speed multiplier.
    #[test]
    fn phase2_boss_applies_speed_multiplier() {
        let mut app = build_app();
        let boss = spawn_boss(&mut app, 2000.0, 5000.0, Vec2::new(-10000.0, 0.0));
        app.world_mut().entity_mut(boss).insert(BossPhase::Phase2);
        spawn_player(&mut app, Vec2::ZERO);

        let dt = 1.0_f32;
        advance(&mut app, dt);
        app.world_mut().run_system_once(move_boss_phase2).unwrap();

        let x = app.world().get::<Transform>(boss).unwrap().translation.x;
        let base_speed = Enemy::from_type(EnemyType::BossDeath, 1.0).move_speed;
        let expected = base_speed * PHASE2_SPEED_MULTIPLIER * dt;
        let moved = x - (-10000.0);
        assert!(
            (moved - expected).abs() < 0.1,
            "Phase2 speed should be {expected} px/s; moved {moved}"
        );
    }

    /// Phase1 boss is not moved by move_boss_phase2.
    #[test]
    fn phase1_boss_not_moved_by_phase2_system() {
        let mut app = build_app();
        let boss = spawn_boss(&mut app, 5000.0, 5000.0, Vec2::new(-500.0, 0.0));
        spawn_player(&mut app, Vec2::ZERO);

        advance(&mut app, 1.0 / 60.0);
        app.world_mut().run_system_once(move_boss_phase2).unwrap();

        let x = app.world().get::<Transform>(boss).unwrap().translation.x;
        assert_eq!(x, -500.0, "Phase1 boss must not be moved by Phase2 system");
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

    /// Three Mini Deaths are spawned when Phase1 → Phase2 transition fires.
    #[test]
    fn phase2_transition_spawns_three_mini_deaths() {
        let mut app = build_app();
        spawn_boss(&mut app, 2999.0, 5000.0, Vec2::ZERO);

        app.world_mut()
            .run_system_once(check_boss_phase_transition)
            .unwrap();
        app.world_mut().flush();

        let mut q = app
            .world_mut()
            .query_filtered::<&Enemy, With<GameSessionEntity>>();
        let mini_deaths: Vec<_> = q
            .iter(app.world())
            .filter(|e| e.enemy_type == EnemyType::MiniDeath)
            .collect();
        assert_eq!(
            mini_deaths.len(),
            MINI_DEATH_SPAWN_COUNT,
            "exactly {MINI_DEATH_SPAWN_COUNT} Mini Deaths should spawn at Phase2 transition"
        );
    }

    /// Mini Deaths spawn evenly distributed around the boss.
    #[test]
    fn mini_deaths_spawn_around_boss_position() {
        let boss_pos = Vec2::new(100.0, 200.0);
        let mut app = build_app();
        spawn_boss(&mut app, 2999.0, 5000.0, boss_pos);

        app.world_mut()
            .run_system_once(check_boss_phase_transition)
            .unwrap();
        app.world_mut().flush();

        let mut q = app
            .world_mut()
            .query_filtered::<(&Enemy, &Transform), With<GameSessionEntity>>();
        let positions: Vec<Vec2> = q
            .iter(app.world())
            .filter(|(e, _)| e.enemy_type == EnemyType::MiniDeath)
            .map(|(_, t)| t.translation.truncate())
            .collect();

        for pos in &positions {
            let dist = pos.distance(boss_pos);
            assert!(
                (dist - MINI_DEATH_SPAWN_RADIUS).abs() < 1.0,
                "Mini Death should spawn at radius {MINI_DEATH_SPAWN_RADIUS} from boss; got {dist}"
            );
        }
    }

    /// Mini Deaths are not spawned again on a second call (Phase2 is already set).
    #[test]
    fn mini_deaths_not_spawned_again_in_phase2() {
        let mut app = build_app();
        let boss = spawn_boss(&mut app, 2999.0, 5000.0, Vec2::ZERO);

        // First call: Phase1 → Phase2, spawns Mini Deaths.
        app.world_mut()
            .run_system_once(check_boss_phase_transition)
            .unwrap();
        app.world_mut().flush();

        // Second call: already Phase2, no additional spawns.
        // Lower HP further so Phase3 threshold isn't crossed yet.
        app.world_mut().get_mut::<Enemy>(boss).unwrap().current_hp = 1600.0; // still above 30% of 5000 = 1500
        app.world_mut()
            .run_system_once(check_boss_phase_transition)
            .unwrap();
        app.world_mut().flush();

        let mut q = app
            .world_mut()
            .query_filtered::<&Enemy, With<GameSessionEntity>>();
        let mini_deaths: Vec<_> = q
            .iter(app.world())
            .filter(|e| e.enemy_type == EnemyType::MiniDeath)
            .collect();
        assert_eq!(
            mini_deaths.len(),
            MINI_DEATH_SPAWN_COUNT,
            "Mini Deaths must only spawn once at Phase2 transition"
        );
    }

    /// Phase2 transitions to Phase3 below 30% HP.
    #[test]
    fn transitions_to_phase3_below_threshold() {
        let mut app = build_app();
        let boss = spawn_boss(&mut app, 1499.0, 5000.0, Vec2::ZERO); // 29.98%
        app.world_mut().entity_mut(boss).insert(BossPhase::Phase2);

        app.world_mut()
            .run_system_once(check_boss_phase_transition)
            .unwrap();

        assert_eq!(
            *app.world().get::<BossPhase>(boss).unwrap(),
            BossPhase::Phase3,
            "should transition to Phase3 below 30% threshold"
        );
    }

    /// Exactly at the 30% threshold stays Phase2 (< not <=).
    #[test]
    fn exact_phase3_threshold_stays_phase2() {
        let mut app = build_app();
        let boss = spawn_boss(&mut app, 1500.0, 5000.0, Vec2::ZERO); // exactly 30%
        app.world_mut().entity_mut(boss).insert(BossPhase::Phase2);

        app.world_mut()
            .run_system_once(check_boss_phase_transition)
            .unwrap();

        assert_eq!(
            *app.world().get::<BossPhase>(boss).unwrap(),
            BossPhase::Phase2,
            "exactly 30% HP should remain Phase2 (threshold is exclusive)"
        );
    }
}
