//! Enemy spawn system — timer-based, off-screen.
//!
//! Each frame [`spawn_enemies`] reads the current [`EnemySpawner`] state
//! (set by [`super::difficulty::update_difficulty`]) and, once the effective
//! spawn interval elapses, picks a random position just outside the visible
//! viewport and spawns an enemy chosen from the active spawn table.
//!
//! ## Spawn table (time-gated)
//!
//! | Enemy    | Unlocks at |
//! |----------|-----------|
//! | Bat      | 0 min     |
//! | Skeleton | 0 min     |
//! | Zombie   | 5 min     |

use bevy::prelude::*;
use rand::RngExt;

use crate::{
    components::{CircleCollider, Enemy, EnemyAI, GameSessionEntity},
    config::{EnemyParams, GameParams},
    resources::{EnemySpawner, GameData},
    types::{AIType, EnemyType},
};

// ---------------------------------------------------------------------------
// Fallback constants (used when RON config is not yet loaded)
// ---------------------------------------------------------------------------

/// Maximum simultaneous enemies before spawning is throttled.
const DEFAULT_ENEMY_MAX_COUNT: usize = 500;
/// Collider radius for Bat enemies (pixels).
const DEFAULT_COLLIDER_BAT: f32 = 8.0;
/// Collider radius for Skeleton enemies (pixels).
const DEFAULT_COLLIDER_SKELETON: f32 = 12.0;
/// Collider radius for Zombie enemies (pixels).
const DEFAULT_COLLIDER_ZOMBIE: f32 = 14.0;
/// Elapsed seconds before Zombie is added to the spawn table (5 minutes).
const DEFAULT_ZOMBIE_UNLOCK_SECS: f32 = 300.0;
/// Window width used to compute off-screen spawn bounds (pixels).
const DEFAULT_WINDOW_WIDTH: u32 = 1280;
/// Window height used to compute off-screen spawn bounds (pixels).
const DEFAULT_WINDOW_HEIGHT: u32 = 720;
/// Extra pixels beyond the half-viewport edge at which enemies appear.
const DEFAULT_SPAWN_MARGIN: f32 = 60.0;
/// Base enemy spawn interval in seconds (mirrors EnemyConfig default).
/// Only used in tests to set `spawn_timer` past the threshold.
#[cfg(test)]
const DEFAULT_ENEMY_SPAWN_BASE_INTERVAL: f32 = 0.5;

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
#[allow(clippy::too_many_arguments)]
pub fn spawn_enemies(
    mut commands: Commands,
    mut spawner: ResMut<EnemySpawner>,
    time: Res<Time>,
    camera_q: Query<&Transform, With<Camera2d>>,
    enemy_q: Query<(), With<Enemy>>,
    enemy_cfg: EnemyParams,
    game_cfg: GameParams,
    game_data: Res<GameData>,
) {
    if !spawner.active {
        return;
    }

    // Throttle: do not exceed the enemy cap (from config or constant fallback).
    let max_count = enemy_cfg
        .get()
        .map(|c| c.max_count)
        .unwrap_or(DEFAULT_ENEMY_MAX_COUNT);
    if enemy_q.iter().count() >= max_count {
        return;
    }

    spawner.spawn_timer += time.delta_secs();
    if spawner.spawn_timer < spawner.spawn_interval {
        return;
    }
    spawner.spawn_timer = 0.0;

    // Compute half-viewport dimensions with spawn margin from config.
    let (win_w, win_h) = game_cfg
        .get()
        .map(|c| (c.window_width as f32, c.window_height as f32))
        .unwrap_or((DEFAULT_WINDOW_WIDTH as f32, DEFAULT_WINDOW_HEIGHT as f32));
    let spawn_margin = enemy_cfg
        .get()
        .map(|c| c.spawn_margin)
        .unwrap_or(DEFAULT_SPAWN_MARGIN)
        .max(0.0);
    let half_w = win_w / 2.0 + spawn_margin;
    let half_h = win_h / 2.0 + spawn_margin;

    // Derive the camera-centred spawn position.
    let cam_pos = camera_q
        .single()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);
    let spawn_pos = random_off_screen_position(cam_pos, half_w, half_h);

    // Build the active spawn table based on elapsed time.
    let zombie_unlock = enemy_cfg
        .get()
        .map(|c| c.zombie_unlock_secs)
        .unwrap_or(DEFAULT_ZOMBIE_UNLOCK_SECS);
    let zombie_unlocked = game_data.elapsed_time >= zombie_unlock;

    let mut rng = rand::rng();
    let enemy_type = if zombie_unlocked {
        // Equal weight among Bat, Skeleton, Zombie.
        match rng.random_range(0..3u8) {
            0 => EnemyType::Bat,
            1 => EnemyType::Skeleton,
            _ => EnemyType::Zombie,
        }
    } else {
        // Only Bat and Skeleton before 5 min.
        if rng.random_bool(0.5) {
            EnemyType::Bat
        } else {
            EnemyType::Skeleton
        }
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
///
/// `half_w` and `half_h` are the half-extents of the spawn boundary (already
/// incorporating the window size and spawn margin).
fn random_off_screen_position(cam_pos: Vec2, half_w: f32, half_h: f32) -> Vec2 {
    let mut rng = rand::rng();

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
        EnemyType::Zombie => Color::srgb(0.35, 0.55, 0.25),
        // Fallback for future types added before they get explicit visuals.
        _ => Color::srgb(0.7, 0.3, 0.3),
    }
}

/// Fallback collider radius when config is not loaded.
fn fallback_collider_radius(enemy_type: EnemyType) -> f32 {
    match enemy_type {
        EnemyType::Bat => DEFAULT_COLLIDER_BAT,
        EnemyType::Skeleton => DEFAULT_COLLIDER_SKELETON,
        EnemyType::Zombie => DEFAULT_COLLIDER_ZOMBIE,
        _ => 10.0,
    }
}

/// Spawn a single enemy entity at `position`.
///
/// Derives stats via [`Enemy::from_type`], attaches a placeholder
/// `Sprite` circle, and tags the entity with [`GameSessionEntity`] for
/// end-of-run cleanup.
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
        GameSessionEntity,
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
    use crate::states::AppState;

    // -----------------------------------------------------------------------
    // Integration tests (ECS App)
    // -----------------------------------------------------------------------

    fn build_playing_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<AppState>();
        app.insert_resource(EnemySpawner::default());
        app.insert_resource(GameData::default());
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
            DEFAULT_ENEMY_SPAWN_BASE_INTERVAL + 0.1;

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
            DEFAULT_ENEMY_SPAWN_BASE_INTERVAL + 0.1;

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

    /// Zombie stats match the issue spec (HP 60, speed 60, damage 12).
    #[test]
    fn zombie_has_correct_base_stats() {
        use crate::components::Enemy;
        let e = Enemy::from_type(EnemyType::Zombie, 1.0);
        assert_eq!(e.max_hp, 60.0, "Zombie base HP must be 60");
        assert_eq!(e.move_speed, 60.0, "Zombie speed must be 60");
        assert_eq!(e.damage, 12.0, "Zombie damage must be 12");
    }

    /// Zombie HP scales with the difficulty multiplier.
    #[test]
    fn zombie_hp_scales_with_difficulty() {
        use crate::components::Enemy;
        let base = Enemy::from_type(EnemyType::Zombie, 1.0);
        let hard = Enemy::from_type(EnemyType::Zombie, 2.0);
        assert!(
            (hard.max_hp - base.max_hp * 2.0).abs() < 1e-4,
            "Zombie HP should double at difficulty 2"
        );
    }

    /// Before 5 min, only Bat and Skeleton should spawn (never Zombie).
    #[test]
    fn zombie_does_not_spawn_before_five_minutes() {
        use bevy::ecs::system::RunSystemOnce as _;
        use std::time::Duration;

        // Run 200 spawn attempts to get a statistically reliable sample.
        let mut zombie_count = 0usize;
        for _ in 0..200 {
            let mut app = build_playing_app();
            // elapsed_time stays at 0 (before zombie unlock).
            app.world_mut().resource_mut::<EnemySpawner>().spawn_timer =
                DEFAULT_ENEMY_SPAWN_BASE_INTERVAL + 0.1;
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(Duration::from_secs_f32(1.0 / 60.0));
            app.world_mut()
                .run_system_once(spawn_enemies)
                .expect("spawn_enemies should run");

            let mut q = app.world_mut().query::<&Enemy>();
            for e in q.iter(app.world()) {
                if e.enemy_type == EnemyType::Zombie {
                    zombie_count += 1;
                }
            }
        }
        assert_eq!(
            zombie_count, 0,
            "Zombie must not spawn before 5 min, but spawned {zombie_count} times in 200 attempts"
        );
    }

    /// After 5 min, Zombie should appear in the spawn table.
    #[test]
    fn zombie_can_spawn_after_five_minutes() {
        use bevy::ecs::system::RunSystemOnce as _;
        use std::time::Duration;

        let mut zombie_count = 0usize;
        for _ in 0..500 {
            let mut app = build_playing_app();
            // Set elapsed time to 5 min + 1 s.
            app.world_mut().resource_mut::<GameData>().elapsed_time =
                DEFAULT_ZOMBIE_UNLOCK_SECS + 1.0;
            app.world_mut().resource_mut::<EnemySpawner>().spawn_timer =
                DEFAULT_ENEMY_SPAWN_BASE_INTERVAL + 0.1;
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(Duration::from_secs_f32(1.0 / 60.0));
            app.world_mut()
                .run_system_once(spawn_enemies)
                .expect("spawn_enemies should run");

            let mut q = app.world_mut().query::<&Enemy>();
            for e in q.iter(app.world()) {
                if e.enemy_type == EnemyType::Zombie {
                    zombie_count += 1;
                }
            }
        }
        assert!(
            zombie_count > 0,
            "Zombie must appear after 5 min (0 spawns in 500 attempts)"
        );
    }

    /// Spawned enemy must carry `Enemy`, `EnemyAI`, and `CircleCollider`.
    #[test]
    fn spawned_enemy_has_required_components() {
        use bevy::ecs::system::RunSystemOnce as _;
        use std::time::Duration;

        let mut app = build_playing_app();

        app.world_mut().resource_mut::<EnemySpawner>().spawn_timer =
            DEFAULT_ENEMY_SPAWN_BASE_INTERVAL + 0.1;

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
