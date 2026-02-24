//! Difficulty scaling system and helper functions.
//!
//! [`update_difficulty`] runs every frame while in [`AppState::Playing`] and
//! keeps [`EnemySpawner::difficulty_multiplier`] and
//! [`EnemySpawner::spawn_interval`] in sync with the elapsed run time.
//!
//! ## Scaling formula
//!
//! ```text
//! difficulty_multiplier = 1.0 + floor(elapsed_secs / 60) × 0.1
//! spawn_interval        = ENEMY_SPAWN_BASE_INTERVAL / difficulty_multiplier
//! ```
//!
//! Examples:
//! |  Time  | Multiplier | Spawn interval (base 0.5 s) |
//! |--------|------------|----------------------------|
//! |  0 min |    1.0×    |  0.500 s                   |
//! |  1 min |    1.1×    |  0.455 s                   |
//! | 10 min |    2.0×    |  0.250 s                   |
//! | 30 min |    4.0×    |  0.125 s                   |

use bevy::prelude::*;

use crate::{
    config::EnemyParams,
    constants::{DIFFICULTY_MAX, ENEMY_SPAWN_BASE_INTERVAL},
    resources::{EnemySpawner, GameData},
};

// ---------------------------------------------------------------------------
// Public helpers (pure — easy to unit-test)
// ---------------------------------------------------------------------------

/// Compute the difficulty multiplier from run elapsed time.
///
/// Grows by `0.1` per minute elapsed, starting at `1.0`, and is capped at
/// [`DIFFICULTY_MAX`] to prevent sub-frame spawn intervals at extreme runtimes.
pub fn difficulty_from_elapsed(elapsed_secs: f32) -> f32 {
    let minutes = (elapsed_secs / 60.0).floor();
    (1.0 + minutes * 0.1).min(DIFFICULTY_MAX)
}

/// Compute the effective spawn interval given the current difficulty.
///
/// Interval shrinks as difficulty grows: `BASE / difficulty`.
/// Difficulty values below `1.0` are clamped to `1.0` so the interval
/// never exceeds the base value.
pub fn effective_spawn_interval(difficulty: f32) -> f32 {
    ENEMY_SPAWN_BASE_INTERVAL / difficulty.max(1.0)
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Updates [`EnemySpawner`] difficulty state from the run's elapsed time.
///
/// Runs every frame in [`AppState::Playing`], after [`crate::systems::game_timer::update_game_timer`]
/// so it always sees the current frame's elapsed time.
///
/// `spawn_base_interval` and `difficulty_max` are read from [`EnemyParams`]
/// when config is loaded; otherwise the constants from `constants.rs` are used.
///
/// Updates:
/// - `difficulty_multiplier` — grows by 0.1 per minute, capped at `difficulty_max`
/// - `spawn_interval`        — derived as `base_interval / difficulty_multiplier`
pub fn update_difficulty(
    game_data: Res<GameData>,
    mut spawner: ResMut<EnemySpawner>,
    enemy_cfg: EnemyParams,
) {
    let base_interval = enemy_cfg
        .get()
        .map(|c| c.spawn_base_interval)
        .unwrap_or(ENEMY_SPAWN_BASE_INTERVAL);
    let diff_max = enemy_cfg
        .get()
        .map(|c| c.difficulty_max)
        .unwrap_or(DIFFICULTY_MAX);

    spawner.difficulty_multiplier = difficulty_from_elapsed(game_data.elapsed_time).min(diff_max);
    spawner.spawn_interval = base_interval / spawner.difficulty_multiplier.max(1.0);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce as _;

    use super::*;
    use crate::resources::GameData;

    // Tolerance for floating-point comparisons in this module.
    // Wider than f32::EPSILON (~1.19e-7) to be robust against normal
    // binary-float rounding (0.1 is not exactly representable in f32).
    const EPS: f32 = 1e-6;

    // -----------------------------------------------------------------------
    // Pure-function unit tests
    // -----------------------------------------------------------------------

    #[test]
    fn difficulty_starts_at_one() {
        assert!(
            (difficulty_from_elapsed(0.0) - 1.0).abs() < EPS,
            "difficulty at t=0 should be 1.0"
        );
    }

    #[test]
    fn difficulty_increases_by_point_one_per_minute() {
        let one_min = difficulty_from_elapsed(60.0);
        let two_min = difficulty_from_elapsed(120.0);
        assert!(
            (one_min - 1.1).abs() < EPS,
            "expected 1.1 at 1 min, got {one_min}"
        );
        assert!(
            (two_min - 1.2).abs() < EPS,
            "expected 1.2 at 2 min, got {two_min}"
        );
    }

    #[test]
    fn difficulty_does_not_increase_within_a_minute() {
        let at_zero = difficulty_from_elapsed(0.0);
        let at_59 = difficulty_from_elapsed(59.9);
        assert!(
            (at_zero - at_59).abs() < EPS,
            "difficulty should not increase before a full minute elapses"
        );
    }

    #[test]
    fn difficulty_at_thirty_minutes_is_four() {
        let at_30min = difficulty_from_elapsed(30.0 * 60.0);
        assert!(
            (at_30min - 4.0).abs() < EPS,
            "expected 4.0 at 30 min, got {at_30min}"
        );
    }

    #[test]
    fn difficulty_is_capped_at_difficulty_max() {
        // At 900 min the uncapped formula would give 91.0; the cap must apply.
        let way_later = difficulty_from_elapsed(900.0 * 60.0);
        assert!(
            way_later <= DIFFICULTY_MAX + EPS,
            "difficulty should not exceed DIFFICULTY_MAX ({DIFFICULTY_MAX}), got {way_later}"
        );
    }

    #[test]
    fn effective_interval_shrinks_with_difficulty() {
        let base = effective_spawn_interval(1.0);
        let harder = effective_spawn_interval(2.0);
        assert!(
            harder < base,
            "interval at difficulty 2 ({harder}) should be less than at 1 ({base})"
        );
        assert!(
            (base - ENEMY_SPAWN_BASE_INTERVAL).abs() < EPS,
            "interval at difficulty 1.0 should equal the base constant"
        );
    }

    #[test]
    fn effective_interval_clamps_difficulty_below_one() {
        let clamped = effective_spawn_interval(0.5);
        let base = effective_spawn_interval(1.0);
        assert!(
            (clamped - base).abs() < EPS,
            "difficulty below 1.0 should be clamped to 1.0"
        );
    }

    // -----------------------------------------------------------------------
    // Integration tests (ECS App)
    // -----------------------------------------------------------------------

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(GameData::default());
        app.insert_resource(EnemySpawner::default());
        app
    }

    /// `update_difficulty` must write the correct multiplier into `EnemySpawner`.
    #[test]
    fn update_difficulty_sets_correct_multiplier() {
        let mut app = build_app();
        app.world_mut().resource_mut::<GameData>().elapsed_time = 120.0; // 2 minutes

        app.world_mut()
            .run_system_once(update_difficulty)
            .expect("update_difficulty should run");

        let diff = app.world().resource::<EnemySpawner>().difficulty_multiplier;
        let expected = difficulty_from_elapsed(120.0);
        assert!(
            (diff - expected).abs() < EPS,
            "expected {expected} at 2 min, got {diff}"
        );
    }

    /// `update_difficulty` must also update `spawn_interval` consistently.
    #[test]
    fn update_difficulty_sets_spawn_interval() {
        let mut app = build_app();
        app.world_mut().resource_mut::<GameData>().elapsed_time = 60.0; // 1 minute

        app.world_mut()
            .run_system_once(update_difficulty)
            .expect("update_difficulty should run");

        // At 60 s → difficulty = 1.1 → interval = BASE / 1.1.
        // Computed independently of the system under test.
        let expected_interval = ENEMY_SPAWN_BASE_INTERVAL / 1.1_f32;
        let actual = app.world().resource::<EnemySpawner>().spawn_interval;
        assert!(
            (actual - expected_interval).abs() < EPS,
            "spawn_interval should be {expected_interval}, got {actual}"
        );
    }

    /// At t=0 the difficulty multiplier should be exactly 1.0.
    #[test]
    fn update_difficulty_at_zero_elapsed_is_one() {
        let mut app = build_app();
        // elapsed_time defaults to 0.0

        app.world_mut()
            .run_system_once(update_difficulty)
            .expect("update_difficulty should run");

        let diff = app.world().resource::<EnemySpawner>().difficulty_multiplier;
        assert!(
            (diff - 1.0).abs() < EPS,
            "difficulty at t=0 should be 1.0, got {diff}"
        );
    }

    /// Spawn interval must be shorter at high difficulty than at the start.
    #[test]
    fn update_difficulty_spawn_interval_decreases_over_time() {
        let mut app = build_app();

        // Run at t=0
        app.world_mut()
            .run_system_once(update_difficulty)
            .expect("run at t=0");
        let interval_start = app.world().resource::<EnemySpawner>().spawn_interval;

        // Run at t=10 min
        app.world_mut().resource_mut::<GameData>().elapsed_time = 600.0;
        app.world_mut()
            .run_system_once(update_difficulty)
            .expect("run at t=10 min");
        let interval_10min = app.world().resource::<EnemySpawner>().spawn_interval;

        assert!(
            interval_10min < interval_start,
            "spawn interval at 10 min ({interval_10min}) should be shorter than at start ({interval_start})"
        );
    }
}
