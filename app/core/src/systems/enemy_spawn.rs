//! Enemy spawn system — timer-based, off-screen, difficulty-scaled.
//!
//! Each frame [`spawn_enemies`] checks [`EnemySpawner`] and, once the
//! effective spawn interval elapses, picks a random position just outside the
//! visible viewport and spawns either a [`EnemyType::Bat`] or
//! [`EnemyType::Skeleton`] (50 / 50).
//!
//! Difficulty scales with run time: every 60 seconds the
//! `difficulty_multiplier` increases by `0.1`, which shortens the spawn
//! interval and raises enemy HP.

use bevy::{prelude::*, state::state_scoped::DespawnOnExit};
use rand::RngExt;

use crate::{
    components::{CircleCollider, Enemy, EnemyAI},
    constants::{
        COLLIDER_BAT, COLLIDER_SKELETON, ENEMY_MAX_COUNT, ENEMY_SPAWN_BASE_INTERVAL, WINDOW_HEIGHT,
        WINDOW_WIDTH,
    },
    resources::{EnemySpawner, GameData},
    states::AppState,
    types::{AIType, EnemyType},
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Extra pixels beyond the half-viewport at which enemies appear.
///
/// Keeps enemies just outside the visible area so they "walk on-screen"
/// rather than popping into view.
const SPAWN_MARGIN: f32 = 60.0;

// ---------------------------------------------------------------------------
// Public helpers (pure functions — easy to unit-test)
// ---------------------------------------------------------------------------

/// Compute the difficulty multiplier from run elapsed time.
///
/// Grows by `0.1` per minute elapsed, starting at `1.0`.
/// e.g. 0 min → 1.0, 1 min → 1.1, 10 min → 2.0, 30 min → 4.0.
pub fn difficulty_from_elapsed(elapsed_secs: f32) -> f32 {
    let minutes = (elapsed_secs / 60.0).floor();
    1.0 + minutes * 0.1
}

/// Compute the effective spawn interval given the current difficulty.
///
/// Interval shrinks as difficulty grows: `BASE / difficulty`.
pub fn effective_spawn_interval(difficulty: f32) -> f32 {
    ENEMY_SPAWN_BASE_INTERVAL / difficulty.max(1.0)
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Spawns enemies off-screen at a timer-driven rate while in
/// [`AppState::Playing`].
///
/// Each frame this system:
/// 1. Returns early when [`EnemySpawner::active`] is `false`.
/// 2. Updates `difficulty_multiplier` from [`GameData::elapsed_time`].
/// 3. Throttles when the current enemy count reaches [`ENEMY_MAX_COUNT`].
/// 4. Accumulates delta time; spawns once the effective interval elapses.
/// 5. Picks a random off-screen edge position and a random enemy type.
pub fn spawn_enemies(
    mut commands: Commands,
    mut spawner: ResMut<EnemySpawner>,
    game_data: Res<GameData>,
    time: Res<Time>,
    camera_q: Query<&Transform, With<Camera2d>>,
    enemy_q: Query<(), With<Enemy>>,
) {
    if !spawner.active {
        return;
    }

    // Update difficulty and derived spawn interval from elapsed run time.
    spawner.difficulty_multiplier = difficulty_from_elapsed(game_data.elapsed_time);
    spawner.spawn_interval = effective_spawn_interval(spawner.difficulty_multiplier);

    // Throttle: do not exceed the enemy cap.
    if enemy_q.iter().count() >= ENEMY_MAX_COUNT {
        return;
    }

    spawner.spawn_timer += time.delta_secs();
    if spawner.spawn_timer < spawner.spawn_interval {
        return;
    }
    spawner.spawn_timer = 0.0;

    // Derive the camera-centred spawn position.
    let cam_pos = camera_q
        .single()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);
    let spawn_pos = random_off_screen_position(cam_pos);

    // 50 / 50 between Bat and Skeleton.
    let mut rng = rand::rng();
    let enemy_type = if rng.random_bool(0.5) {
        EnemyType::Bat
    } else {
        EnemyType::Skeleton
    };

    spawn_enemy(
        &mut commands,
        enemy_type,
        spawn_pos,
        spawner.difficulty_multiplier,
    );
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Choose a uniformly random position just outside one of the four viewport
/// edges, centred on `cam_pos`.
fn random_off_screen_position(cam_pos: Vec2) -> Vec2 {
    let mut rng = rand::rng();
    let half_w = WINDOW_WIDTH as f32 / 2.0 + SPAWN_MARGIN;
    let half_h = WINDOW_HEIGHT as f32 / 2.0 + SPAWN_MARGIN;

    match rng.random_range(0..4u8) {
        // Top edge
        0 => Vec2::new(
            cam_pos.x + rng.random_range(-half_w..half_w),
            cam_pos.y + half_h,
        ),
        // Bottom edge
        1 => Vec2::new(
            cam_pos.x + rng.random_range(-half_w..half_w),
            cam_pos.y - half_h,
        ),
        // Left edge
        2 => Vec2::new(
            cam_pos.x - half_w,
            cam_pos.y + rng.random_range(-half_h..half_h),
        ),
        // Right edge
        _ => Vec2::new(
            cam_pos.x + half_w,
            cam_pos.y + rng.random_range(-half_h..half_h),
        ),
    }
}

/// Placeholder color and collider radius for each spawn-eligible enemy type.
fn enemy_visuals(enemy_type: EnemyType) -> (Color, f32) {
    match enemy_type {
        EnemyType::Bat => (Color::srgb(0.5, 0.1, 0.8), COLLIDER_BAT),
        EnemyType::Skeleton => (Color::srgb(0.9, 0.9, 0.8), COLLIDER_SKELETON),
        // Fallback for future types added before they get explicit visuals.
        _ => (Color::srgb(0.7, 0.3, 0.3), 10.0),
    }
}

/// Spawn a single enemy entity at `position`.
///
/// Derives stats via [`Enemy::from_type`], attaches a placeholder
/// `Sprite` circle, and tags the entity with
/// [`DespawnOnExit(AppState::Playing)`] for automatic cleanup.
fn spawn_enemy(commands: &mut Commands, enemy_type: EnemyType, position: Vec2, difficulty: f32) {
    let (color, radius) = enemy_visuals(enemy_type);

    commands.spawn((
        Enemy::from_type(enemy_type, difficulty),
        EnemyAI {
            ai_type: AIType::ChasePlayer,
            attack_timer: 0.0,
            attack_range: 20.0,
        },
        CircleCollider { radius },
        Sprite {
            color,
            custom_size: Some(Vec2::splat(radius * 2.0)),
            ..default()
        },
        Transform::from_translation(position.extend(5.0)),
        DespawnOnExit(AppState::Playing),
    ));
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::resources::GameData;

    // -----------------------------------------------------------------------
    // Pure-function unit tests
    // -----------------------------------------------------------------------

    #[test]
    fn difficulty_starts_at_one() {
        assert!(
            (difficulty_from_elapsed(0.0) - 1.0).abs() < f32::EPSILON,
            "difficulty at t=0 should be 1.0"
        );
    }

    #[test]
    fn difficulty_increases_by_point_one_per_minute() {
        let one_min = difficulty_from_elapsed(60.0);
        let two_min = difficulty_from_elapsed(120.0);
        assert!(
            (one_min - 1.1).abs() < f32::EPSILON,
            "expected 1.1 at 1 min, got {one_min}"
        );
        assert!(
            (two_min - 1.2).abs() < f32::EPSILON,
            "expected 1.2 at 2 min, got {two_min}"
        );
    }

    #[test]
    fn difficulty_does_not_increase_within_a_minute() {
        // 0 s and 59 s should both yield the same multiplier.
        let at_zero = difficulty_from_elapsed(0.0);
        let at_59 = difficulty_from_elapsed(59.9);
        assert!(
            (at_zero - at_59).abs() < f32::EPSILON,
            "difficulty should not increase before a full minute elapses"
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
            (base - ENEMY_SPAWN_BASE_INTERVAL).abs() < f32::EPSILON,
            "interval at difficulty 1.0 should equal the base constant"
        );
    }

    #[test]
    fn effective_interval_clamps_difficulty_below_one() {
        let clamped = effective_spawn_interval(0.5);
        let base = effective_spawn_interval(1.0);
        assert!(
            (clamped - base).abs() < f32::EPSILON,
            "difficulty below 1.0 should be clamped to 1.0"
        );
    }

    // -----------------------------------------------------------------------
    // Integration tests (ECS App)
    // -----------------------------------------------------------------------

    fn build_playing_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<AppState>();
        app.insert_resource(GameData::default());
        app.insert_resource(EnemySpawner::default());
        app
    }

    /// After enough time passes, at least one enemy entity should be spawned.
    #[test]
    fn spawn_enemies_creates_enemy_after_interval() {
        use bevy::ecs::system::RunSystemOnce as _;
        use std::time::Duration;

        let mut app = build_playing_app();

        // No camera → cam_pos defaults to Vec2::ZERO (graceful fallback).
        // Advance time past the base spawn interval.
        app.world_mut().resource_mut::<EnemySpawner>().spawn_timer =
            ENEMY_SPAWN_BASE_INTERVAL + 0.1;

        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(1.0 / 60.0));

        app.world_mut()
            .run_system_once(spawn_enemies)
            .expect("spawn_enemies should run");

        let mut q = app.world_mut().query_filtered::<Entity, With<Enemy>>();
        let count = q.iter(app.world()).count();
        assert_eq!(count, 1, "expected exactly one enemy to be spawned");
    }

    /// When `active` is false, no enemy should be spawned.
    #[test]
    fn spawn_enemies_inactive_skips_spawn() {
        use bevy::ecs::system::RunSystemOnce as _;
        use std::time::Duration;

        let mut app = build_playing_app();

        app.world_mut().resource_mut::<EnemySpawner>().active = false;
        app.world_mut().resource_mut::<EnemySpawner>().spawn_timer =
            ENEMY_SPAWN_BASE_INTERVAL + 0.1;

        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(1.0 / 60.0));

        app.world_mut()
            .run_system_once(spawn_enemies)
            .expect("spawn_enemies should run");

        let mut q = app.world_mut().query_filtered::<Entity, With<Enemy>>();
        assert_eq!(
            q.iter(app.world()).count(),
            0,
            "inactive spawner must not spawn enemies"
        );
    }

    /// When the timer has not yet elapsed, no enemy is spawned.
    #[test]
    fn spawn_enemies_waits_for_interval() {
        use bevy::ecs::system::RunSystemOnce as _;
        use std::time::Duration;

        let mut app = build_playing_app();

        // spawn_timer starts at 0 → far below the interval.
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(1.0 / 60.0));

        app.world_mut()
            .run_system_once(spawn_enemies)
            .expect("spawn_enemies should run");

        let mut q = app.world_mut().query_filtered::<Entity, With<Enemy>>();
        assert_eq!(
            q.iter(app.world()).count(),
            0,
            "timer has not elapsed — no enemy should be spawned yet"
        );
    }

    /// Spawned enemy must carry `Enemy`, `EnemyAI`, and `CircleCollider`.
    #[test]
    fn spawned_enemy_has_required_components() {
        use bevy::ecs::system::RunSystemOnce as _;
        use std::time::Duration;

        let mut app = build_playing_app();

        app.world_mut().resource_mut::<EnemySpawner>().spawn_timer =
            ENEMY_SPAWN_BASE_INTERVAL + 0.1;

        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(1.0 / 60.0));

        app.world_mut()
            .run_system_once(spawn_enemies)
            .expect("spawn_enemies should run");

        let mut q = app.world_mut().query_filtered::<Entity, With<Enemy>>();
        let entity = q.single(app.world()).expect("one enemy should exist");

        let w = app.world();
        assert!(w.get::<Enemy>(entity).is_some(), "missing Enemy");
        assert!(w.get::<EnemyAI>(entity).is_some(), "missing EnemyAI");
        assert!(
            w.get::<CircleCollider>(entity).is_some(),
            "missing CircleCollider"
        );
        assert!(w.get::<Transform>(entity).is_some(), "missing Transform");
        assert!(w.get::<Sprite>(entity).is_some(), "missing Sprite");
    }

    /// Difficulty multiplier in `EnemySpawner` must be updated on each call.
    #[test]
    fn spawn_enemies_updates_difficulty_multiplier() {
        use bevy::ecs::system::RunSystemOnce as _;
        use std::time::Duration;

        let mut app = build_playing_app();

        // Simulate 2 minutes of elapsed run time.
        app.world_mut().resource_mut::<GameData>().elapsed_time = 120.0;

        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(1.0 / 60.0));

        app.world_mut()
            .run_system_once(spawn_enemies)
            .expect("spawn_enemies should run");

        let diff = app.world().resource::<EnemySpawner>().difficulty_multiplier;
        let expected = difficulty_from_elapsed(120.0);
        assert!(
            (diff - expected).abs() < f32::EPSILON,
            "difficulty should be {expected} at 2 min, got {diff}"
        );
    }
}
