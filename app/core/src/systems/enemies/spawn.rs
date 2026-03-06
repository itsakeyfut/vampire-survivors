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
//! | Ghost    | 10 min    |
//! | Demon    | 15 min    |
//! | Medusa   | 20 min    |
//! | Dragon   | 25 min    |

use bevy::prelude::*;
use rand::RngExt;

use crate::{
    components::{CircleCollider, Enemy, EnemyAI, GameSessionEntity, PhaseThrough},
    config::{EnemyParams, EnemyStatsEntry, GameParams},
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
/// Collider radius for Ghost enemies (pixels).
const DEFAULT_COLLIDER_GHOST: f32 = 10.0;
/// Elapsed seconds before Zombie is added to the spawn table (5 minutes).
const DEFAULT_ZOMBIE_UNLOCK_SECS: f32 = 300.0;
/// Elapsed seconds before Ghost is added to the spawn table (10 minutes).
const DEFAULT_GHOST_UNLOCK_SECS: f32 = 600.0;
/// Collider radius for Demon enemies (pixels).
const DEFAULT_COLLIDER_DEMON: f32 = 14.0;
/// Elapsed seconds before Demon is added to the spawn table (15 minutes).
const DEFAULT_DEMON_UNLOCK_SECS: f32 = 900.0;
/// Collider radius for Medusa enemies (pixels).
const DEFAULT_COLLIDER_MEDUSA: f32 = 12.0;
/// Elapsed seconds before Medusa is added to the spawn table (20 minutes).
const DEFAULT_MEDUSA_UNLOCK_SECS: f32 = 1200.0;
/// Collider radius for Dragon enemies (pixels).
const DEFAULT_COLLIDER_DRAGON: f32 = 20.0;
/// Elapsed seconds before Dragon is added to the spawn table (25 minutes).
const DEFAULT_DRAGON_UNLOCK_SECS: f32 = 1500.0;
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
    let elapsed = game_data.elapsed_time;
    let zombie_unlock = enemy_cfg
        .get()
        .map(|c| c.zombie_unlock_secs)
        .unwrap_or(DEFAULT_ZOMBIE_UNLOCK_SECS)
        .max(0.0);
    let ghost_unlock = enemy_cfg
        .get()
        .map(|c| c.ghost_unlock_secs)
        .unwrap_or(DEFAULT_GHOST_UNLOCK_SECS)
        .max(0.0);
    let demon_unlock = enemy_cfg
        .get()
        .map(|c| c.demon_unlock_secs)
        .unwrap_or(DEFAULT_DEMON_UNLOCK_SECS)
        .max(0.0);
    let medusa_unlock = enemy_cfg
        .get()
        .map(|c| c.medusa_unlock_secs)
        .unwrap_or(DEFAULT_MEDUSA_UNLOCK_SECS)
        .max(0.0);
    let dragon_unlock = enemy_cfg
        .get()
        .map(|c| c.dragon_unlock_secs)
        .unwrap_or(DEFAULT_DRAGON_UNLOCK_SECS)
        .max(0.0);

    // Build the spawn table from independent unlock flags so each enemy type
    // respects only its own threshold, regardless of ordering in the config.
    let mut table: Vec<EnemyType> = vec![EnemyType::Bat, EnemyType::Skeleton];
    if elapsed >= zombie_unlock {
        table.push(EnemyType::Zombie);
    }
    if elapsed >= ghost_unlock {
        table.push(EnemyType::Ghost);
    }
    if elapsed >= demon_unlock {
        table.push(EnemyType::Demon);
    }
    if elapsed >= medusa_unlock {
        table.push(EnemyType::Medusa);
    }
    if elapsed >= dragon_unlock {
        table.push(EnemyType::Dragon);
    }

    let mut rng = rand::rng();
    let idx = rng.random_range(0..table.len() as u8) as usize;
    let enemy_type = table[idx];

    // Derive all enemy stats from config when available, falling back to constants.
    let cfg_stats = enemy_cfg.get().map(|c| c.stats_for(enemy_type).clone());
    let collider_radius = cfg_stats
        .as_ref()
        .map(|s| s.collider_radius)
        .unwrap_or_else(|| fallback_collider_radius(enemy_type));

    spawn_enemy(
        &mut commands,
        enemy_type,
        spawn_pos,
        spawner.difficulty_multiplier,
        collider_radius,
        cfg_stats.as_ref(),
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
        // Semi-transparent white/blue for Ghost.
        EnemyType::Ghost => Color::srgba(0.8, 0.9, 1.0, 0.55),
        // Deep red/fiery for Demon.
        EnemyType::Demon => Color::srgb(0.8, 0.1, 0.05),
        // Stone/snake gray for Medusa.
        EnemyType::Medusa => Color::srgb(0.6, 0.6, 0.5),
        // Deep orange-red for Dragon.
        EnemyType::Dragon => Color::srgb(0.9, 0.3, 0.0),
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
        EnemyType::Ghost => DEFAULT_COLLIDER_GHOST,
        EnemyType::Demon => DEFAULT_COLLIDER_DEMON,
        EnemyType::Medusa => DEFAULT_COLLIDER_MEDUSA,
        EnemyType::Dragon => DEFAULT_COLLIDER_DRAGON,
        _ => 10.0,
    }
}

/// Spawn a single enemy entity at `position`.
///
/// Uses `cfg_stats` to build the [`Enemy`] component when available so that
/// all stats (HP, speed, damage, XP, gold) reflect the loaded RON config.
/// Falls back to compile-time `DEFAULT_ENEMY_STATS_*` constants otherwise.
/// Ghost enemies additionally receive [`PhaseThrough`].
/// Medusa enemies use [`AIType::KeepDistance`] instead of `ChasePlayer`.
fn spawn_enemy(
    commands: &mut Commands,
    enemy_type: EnemyType,
    position: Vec2,
    difficulty: f32,
    collider_radius: f32,
    cfg_stats: Option<&EnemyStatsEntry>,
) {
    let color = enemy_color(enemy_type);
    let enemy_component = match cfg_stats {
        Some(stats) => Enemy::from_config(enemy_type, stats, difficulty),
        None => Enemy::from_type(enemy_type, difficulty),
    };
    let ai = if enemy_type == EnemyType::Medusa {
        EnemyAI {
            ai_type: AIType::KeepDistance,
            attack_timer: 0.0,
            attack_range: 250.0,
        }
    } else {
        EnemyAI {
            ai_type: AIType::ChasePlayer,
            attack_timer: 0.0,
            attack_range: 20.0,
        }
    };
    let mut entity = commands.spawn((
        enemy_component,
        ai,
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

    if enemy_type == EnemyType::Ghost {
        entity.insert(PhaseThrough);
    }
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

    /// Ghost stats match the issue spec (HP 25, speed 100, damage 10).
    #[test]
    fn ghost_has_correct_base_stats() {
        use crate::components::Enemy;
        let e = Enemy::from_type(EnemyType::Ghost, 1.0);
        assert_eq!(e.max_hp, 25.0, "Ghost base HP must be 25");
        assert_eq!(e.move_speed, 100.0, "Ghost speed must be 100");
        assert_eq!(e.damage, 10.0, "Ghost damage must be 10");
    }

    /// Before 10 min, Ghost must not appear in the spawn table.
    #[test]
    fn ghost_does_not_spawn_before_ten_minutes() {
        use bevy::ecs::system::RunSystemOnce as _;
        use std::time::Duration;

        let mut ghost_count = 0usize;
        for _ in 0..200 {
            let mut app = build_playing_app();
            // Set elapsed just below ghost unlock (9 min 59 s).
            app.world_mut().resource_mut::<GameData>().elapsed_time =
                DEFAULT_GHOST_UNLOCK_SECS - 1.0;
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
                if e.enemy_type == EnemyType::Ghost {
                    ghost_count += 1;
                }
            }
        }
        assert_eq!(
            ghost_count, 0,
            "Ghost must not spawn before 10 min, but spawned {ghost_count} times in 200 attempts"
        );
    }

    /// After 10 min, Ghost should appear in the spawn table.
    #[test]
    fn ghost_can_spawn_after_ten_minutes() {
        use bevy::ecs::system::RunSystemOnce as _;
        use std::time::Duration;

        let mut ghost_count = 0usize;
        for _ in 0..500 {
            let mut app = build_playing_app();
            app.world_mut().resource_mut::<GameData>().elapsed_time =
                DEFAULT_GHOST_UNLOCK_SECS + 1.0;
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
                if e.enemy_type == EnemyType::Ghost {
                    ghost_count += 1;
                }
            }
        }
        assert!(
            ghost_count > 0,
            "Ghost must appear after 10 min (0 spawns in 500 attempts)"
        );
    }

    /// Ghost spawned after 10 min must carry the PhaseThrough component.
    #[test]
    fn ghost_has_phase_through_component() {
        use bevy::ecs::system::RunSystemOnce as _;
        use std::time::Duration;

        // Run until a Ghost spawns.
        for _ in 0..500 {
            let mut app = build_playing_app();
            app.world_mut().resource_mut::<GameData>().elapsed_time =
                DEFAULT_GHOST_UNLOCK_SECS + 1.0;
            app.world_mut().resource_mut::<EnemySpawner>().spawn_timer =
                DEFAULT_ENEMY_SPAWN_BASE_INTERVAL + 0.1;
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(Duration::from_secs_f32(1.0 / 60.0));
            app.world_mut()
                .run_system_once(spawn_enemies)
                .expect("spawn_enemies should run");
            app.world_mut().flush();

            let mut q = app
                .world_mut()
                .query_filtered::<Entity, With<PhaseThrough>>();
            let phase_through_count = q.iter(app.world()).count();
            let mut eq = app.world_mut().query::<&Enemy>();
            let ghost_count = eq
                .iter(app.world())
                .filter(|e| e.enemy_type == EnemyType::Ghost)
                .count();

            if ghost_count > 0 {
                assert_eq!(
                    phase_through_count, ghost_count,
                    "every Ghost must carry PhaseThrough"
                );
                return;
            }
        }
        panic!("expected at least one Ghost spawn in 500 attempts to verify PhaseThrough");
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

    /// Demon stats match the issue spec (HP 80, speed 130, damage 15).
    #[test]
    fn demon_has_correct_base_stats() {
        use crate::components::Enemy;
        let e = Enemy::from_type(EnemyType::Demon, 1.0);
        assert_eq!(e.max_hp, 80.0, "Demon base HP must be 80");
        assert_eq!(e.move_speed, 130.0, "Demon speed must be 130");
        assert_eq!(e.damage, 15.0, "Demon damage must be 15");
    }

    /// Demon HP scales with the difficulty multiplier.
    #[test]
    fn demon_hp_scales_with_difficulty() {
        use crate::components::Enemy;
        let base = Enemy::from_type(EnemyType::Demon, 1.0);
        let hard = Enemy::from_type(EnemyType::Demon, 2.0);
        assert!(
            (hard.max_hp - base.max_hp * 2.0).abs() < 1e-4,
            "Demon HP should double at difficulty 2"
        );
    }

    /// Before 15 min, Demon must not appear in the spawn table.
    #[test]
    fn demon_does_not_spawn_before_fifteen_minutes() {
        use bevy::ecs::system::RunSystemOnce as _;
        use std::time::Duration;

        let mut demon_count = 0usize;
        for _ in 0..200 {
            let mut app = build_playing_app();
            app.world_mut().resource_mut::<GameData>().elapsed_time =
                DEFAULT_DEMON_UNLOCK_SECS - 1.0;
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
                if e.enemy_type == EnemyType::Demon {
                    demon_count += 1;
                }
            }
        }
        assert_eq!(
            demon_count, 0,
            "Demon must not spawn before 15 min, but spawned {demon_count} times in 200 attempts"
        );
    }

    /// After 15 min, Demon should appear in the spawn table.
    #[test]
    fn demon_can_spawn_after_fifteen_minutes() {
        use bevy::ecs::system::RunSystemOnce as _;
        use std::time::Duration;

        let mut demon_count = 0usize;
        for _ in 0..500 {
            let mut app = build_playing_app();
            app.world_mut().resource_mut::<GameData>().elapsed_time =
                DEFAULT_DEMON_UNLOCK_SECS + 1.0;
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
                if e.enemy_type == EnemyType::Demon {
                    demon_count += 1;
                }
            }
        }
        assert!(
            demon_count > 0,
            "Demon must appear after 15 min (0 spawns in 500 attempts)"
        );
    }

    /// Medusa stats match the issue spec (HP 60, speed 60, damage 12).
    #[test]
    fn medusa_has_correct_base_stats() {
        use crate::components::Enemy;
        let e = Enemy::from_type(EnemyType::Medusa, 1.0);
        assert_eq!(e.max_hp, 60.0, "Medusa base HP must be 60");
        assert_eq!(e.move_speed, 60.0, "Medusa speed must be 60");
        assert_eq!(e.damage, 12.0, "Medusa damage must be 12");
    }

    /// Before 20 min, Medusa must not appear in the spawn table.
    #[test]
    fn medusa_does_not_spawn_before_twenty_minutes() {
        use bevy::ecs::system::RunSystemOnce as _;
        use std::time::Duration;

        let mut medusa_count = 0usize;
        for _ in 0..200 {
            let mut app = build_playing_app();
            app.world_mut().resource_mut::<GameData>().elapsed_time =
                DEFAULT_MEDUSA_UNLOCK_SECS - 1.0;
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
                if e.enemy_type == EnemyType::Medusa {
                    medusa_count += 1;
                }
            }
        }
        assert_eq!(
            medusa_count, 0,
            "Medusa must not spawn before 20 min, but spawned {medusa_count} times in 200 attempts"
        );
    }

    /// After 20 min, Medusa should appear in the spawn table.
    #[test]
    fn medusa_can_spawn_after_twenty_minutes() {
        use bevy::ecs::system::RunSystemOnce as _;
        use std::time::Duration;

        let mut medusa_count = 0usize;
        for _ in 0..500 {
            let mut app = build_playing_app();
            app.world_mut().resource_mut::<GameData>().elapsed_time =
                DEFAULT_MEDUSA_UNLOCK_SECS + 1.0;
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
                if e.enemy_type == EnemyType::Medusa {
                    medusa_count += 1;
                }
            }
        }
        assert!(
            medusa_count > 0,
            "Medusa must appear after 20 min (0 spawns in 500 attempts)"
        );
    }

    /// Medusa spawned after 20 min must use KeepDistance AI.
    #[test]
    fn medusa_spawns_with_keep_distance_ai() {
        use bevy::ecs::system::RunSystemOnce as _;
        use std::time::Duration;

        for _ in 0..500 {
            let mut app = build_playing_app();
            app.world_mut().resource_mut::<GameData>().elapsed_time =
                DEFAULT_MEDUSA_UNLOCK_SECS + 1.0;
            app.world_mut().resource_mut::<EnemySpawner>().spawn_timer =
                DEFAULT_ENEMY_SPAWN_BASE_INTERVAL + 0.1;
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(Duration::from_secs_f32(1.0 / 60.0));
            app.world_mut()
                .run_system_once(spawn_enemies)
                .expect("spawn_enemies should run");
            app.world_mut().flush();

            let mut eq = app.world_mut().query::<(&Enemy, &EnemyAI)>();
            let medusa: Vec<_> = eq
                .iter(app.world())
                .filter(|(e, _)| e.enemy_type == EnemyType::Medusa)
                .collect();

            if !medusa.is_empty() {
                for (_, ai) in &medusa {
                    assert_eq!(
                        ai.ai_type,
                        AIType::KeepDistance,
                        "Medusa must use KeepDistance AI"
                    );
                }
                return;
            }
        }
        panic!("expected at least one Medusa spawn in 500 attempts to verify KeepDistance AI");
    }

    /// Dragon stats match the issue spec (HP 150, speed 90, damage 25).
    #[test]
    fn dragon_has_correct_base_stats() {
        use crate::components::Enemy;
        let e = Enemy::from_type(EnemyType::Dragon, 1.0);
        assert_eq!(e.max_hp, 150.0, "Dragon base HP must be 150");
        assert_eq!(e.move_speed, 90.0, "Dragon speed must be 90");
        assert_eq!(e.damage, 25.0, "Dragon damage must be 25");
    }

    /// Before 25 min, Dragon must not appear in the spawn table.
    #[test]
    fn dragon_does_not_spawn_before_twenty_five_minutes() {
        use bevy::ecs::system::RunSystemOnce as _;
        use std::time::Duration;

        let mut dragon_count = 0usize;
        for _ in 0..200 {
            let mut app = build_playing_app();
            app.world_mut().resource_mut::<GameData>().elapsed_time =
                DEFAULT_DRAGON_UNLOCK_SECS - 1.0;
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
                if e.enemy_type == EnemyType::Dragon {
                    dragon_count += 1;
                }
            }
        }
        assert_eq!(
            dragon_count, 0,
            "Dragon must not spawn before 25 min, but spawned {dragon_count} times in 200 attempts"
        );
    }

    /// After 25 min, Dragon should appear in the spawn table.
    #[test]
    fn dragon_can_spawn_after_twenty_five_minutes() {
        use bevy::ecs::system::RunSystemOnce as _;
        use std::time::Duration;

        let mut dragon_count = 0usize;
        for _ in 0..500 {
            let mut app = build_playing_app();
            app.world_mut().resource_mut::<GameData>().elapsed_time =
                DEFAULT_DRAGON_UNLOCK_SECS + 1.0;
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
                if e.enemy_type == EnemyType::Dragon {
                    dragon_count += 1;
                }
            }
        }
        assert!(
            dragon_count > 0,
            "Dragon must appear after 25 min (0 spawns in 500 attempts)"
        );
    }

    /// Dragon spawned after 25 min must use ChasePlayer AI.
    #[test]
    fn dragon_spawns_with_chase_player_ai() {
        use bevy::ecs::system::RunSystemOnce as _;
        use std::time::Duration;

        for _ in 0..500 {
            let mut app = build_playing_app();
            app.world_mut().resource_mut::<GameData>().elapsed_time =
                DEFAULT_DRAGON_UNLOCK_SECS + 1.0;
            app.world_mut().resource_mut::<EnemySpawner>().spawn_timer =
                DEFAULT_ENEMY_SPAWN_BASE_INTERVAL + 0.1;
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(Duration::from_secs_f32(1.0 / 60.0));
            app.world_mut()
                .run_system_once(spawn_enemies)
                .expect("spawn_enemies should run");
            app.world_mut().flush();

            let mut eq = app.world_mut().query::<(&Enemy, &EnemyAI)>();
            let dragons: Vec<_> = eq
                .iter(app.world())
                .filter(|(e, _)| e.enemy_type == EnemyType::Dragon)
                .collect();

            if !dragons.is_empty() {
                for (_, ai) in &dragons {
                    assert_eq!(
                        ai.ai_type,
                        AIType::ChasePlayer,
                        "Dragon must use ChasePlayer AI"
                    );
                }
                return;
            }
        }
        panic!("expected at least one Dragon spawn in 500 attempts to verify ChasePlayer AI");
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
