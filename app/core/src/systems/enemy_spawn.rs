//! Enemy spawn system — timer-based, off-screen.
//!
//! Each frame [`spawn_enemies`] reads the current [`EnemySpawner`] state
//! (set by [`super::difficulty::update_difficulty`]) and, once the effective
//! spawn interval elapses, picks a random position just outside the visible
//! viewport and spawns either a [`EnemyType::Bat`] or [`EnemyType::Skeleton`]
//! (50 / 50).

use bevy::{prelude::*, state::state_scoped::DespawnOnExit};
use rand::RngExt;

use crate::{
    components::{CircleCollider, Enemy, EnemyAI},
    config::EnemyParams,
    constants::{COLLIDER_BAT, COLLIDER_SKELETON, ENEMY_MAX_COUNT, WINDOW_HEIGHT, WINDOW_WIDTH},
    resources::EnemySpawner,
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
// System
// ---------------------------------------------------------------------------

/// Spawns enemies off-screen at a timer-driven rate while in
/// [`AppState::Playing`].
///
/// Reads `difficulty_multiplier` and `spawn_interval` from [`EnemySpawner`],
/// which are kept up-to-date by
/// [`super::difficulty::update_difficulty`] (runs earlier in the same frame).
///
/// Each frame this system:
/// 1. Returns early when [`EnemySpawner::active`] is `false`.
/// 2. Throttles when the current enemy count reaches [`ENEMY_MAX_COUNT`].
/// 3. Accumulates delta time; spawns once the effective interval elapses.
/// 4. Picks a random off-screen edge position and a random enemy type.
pub fn spawn_enemies(
    mut commands: Commands,
    mut spawner: ResMut<EnemySpawner>,
    time: Res<Time>,
    camera_q: Query<&Transform, With<Camera2d>>,
    enemy_q: Query<(), With<Enemy>>,
    enemy_cfg: EnemyParams,
) {
    if !spawner.active {
        return;
    }

    // Throttle: do not exceed the enemy cap (from config or constant fallback).
    let max_count = enemy_cfg
        .get()
        .map(|c| c.max_count)
        .unwrap_or(ENEMY_MAX_COUNT);
    if enemy_q.iter().count() >= max_count {
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

    // Derive collider radius from config, falling back to constants.
    let collider_radius = enemy_cfg
        .get()
        .map(|c| c.stats_for(enemy_type).collider_radius)
        .unwrap_or_else(|| fallback_collider_radius(enemy_type));

    spawn_enemy(
        &mut commands,
        enemy_type,
        spawn_pos,
        spawner.difficulty_multiplier,
        collider_radius,
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

/// Placeholder colour for each spawn-eligible enemy type.
fn enemy_color(enemy_type: EnemyType) -> Color {
    match enemy_type {
        EnemyType::Bat => Color::srgb(0.5, 0.1, 0.8),
        EnemyType::Skeleton => Color::srgb(0.9, 0.9, 0.8),
        // Fallback for future types added before they get explicit visuals.
        _ => Color::srgb(0.7, 0.3, 0.3),
    }
}

/// Fallback collider radius from constants when config is not loaded.
fn fallback_collider_radius(enemy_type: EnemyType) -> f32 {
    match enemy_type {
        EnemyType::Bat => COLLIDER_BAT,
        EnemyType::Skeleton => COLLIDER_SKELETON,
        _ => 10.0,
    }
}

/// Spawn a single enemy entity at `position`.
///
/// Derives stats via [`Enemy::from_type`], attaches a placeholder
/// `Sprite` circle, and tags the entity with
/// [`DespawnOnExit(AppState::Playing)`] for automatic cleanup.
fn spawn_enemy(
    commands: &mut Commands,
    enemy_type: EnemyType,
    position: Vec2,
    difficulty: f32,
    collider_radius: f32,
) {
    let color = enemy_color(enemy_type);

    commands.spawn((
        Enemy::from_type(enemy_type, difficulty),
        EnemyAI {
            ai_type: AIType::ChasePlayer,
            attack_timer: 0.0,
            attack_range: 20.0,
        },
        CircleCollider {
            radius: collider_radius,
        },
        Sprite {
            color,
            custom_size: Some(Vec2::splat(collider_radius * 2.0)),
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
    use crate::constants::ENEMY_SPAWN_BASE_INTERVAL;

    // -----------------------------------------------------------------------
    // Integration tests (ECS App)
    // -----------------------------------------------------------------------

    fn build_playing_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<AppState>();
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
}
