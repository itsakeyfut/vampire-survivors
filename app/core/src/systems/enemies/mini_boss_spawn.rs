//! Mini-boss spawn system — timer-based, every 3 minutes.
//!
//! [`spawn_mini_boss`] runs every frame during [`AppState::Playing`].
//! When [`TreasureSpawner::spawn_timer`] first reaches `MINI_BOSS_INTERVAL`
//! (180 seconds) it:
//!
//! 1. Resets the timer to 0.
//! 2. Spawns a [`EnemyType::MiniBoss`] entity just off-screen using the
//!    same random edge logic as the normal enemy spawner.
//!
//! Spawning is skipped once the final boss has appeared (`boss_spawned`
//! flag) so the player is not harassed by mini-bosses during the climax.

use bevy::prelude::*;

use crate::{
    config::{EnemyParams, GameParams},
    resources::{GameData, TreasureSpawner},
    systems::enemies::spawn::{DEFAULT_COLLIDER_MINI_BOSS, spawn_enemy},
    types::EnemyType,
};

// ---------------------------------------------------------------------------
// Fallback constants (used before game.ron / enemy.ron finish loading)
// ---------------------------------------------------------------------------

/// Seconds between each mini-boss spawn.
const MINI_BOSS_INTERVAL: f32 = 180.0;

/// Fallback viewport half-width when `game.ron` is not yet loaded (pixels).
const DEFAULT_HALF_W: f32 = 640.0;
/// Fallback viewport half-height when `game.ron` is not yet loaded (pixels).
const DEFAULT_HALF_H: f32 = 360.0;
/// Extra off-screen margin so the mini-boss spawns out of view (pixels).
const DEFAULT_SPAWN_MARGIN: f32 = 60.0;

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Ticks [`TreasureSpawner`] and spawns a [`EnemyType::MiniBoss`] every
/// [`MINI_BOSS_INTERVAL`] seconds.
///
/// Reads the camera position to compute a random off-screen spawn location
/// (same four-edge strategy used by the normal enemy spawner).  Skips
/// spawning if the final boss has already appeared.
pub fn spawn_mini_boss(
    mut commands: Commands,
    mut treasure_spawner: ResMut<TreasureSpawner>,
    time: Res<Time>,
    camera_q: Query<&Transform, With<Camera2d>>,
    game_data: Res<GameData>,
    game_cfg: GameParams,
    enemy_cfg: EnemyParams,
) {
    // Stop mini-boss spawning once the final boss fight begins.
    if game_data.boss_spawned {
        return;
    }

    treasure_spawner.spawn_timer += time.delta_secs();
    if treasure_spawner.spawn_timer < MINI_BOSS_INTERVAL {
        return;
    }
    treasure_spawner.spawn_timer = 0.0;

    // Compute half-viewport + margin from config, fallback to constants.
    let (win_w, win_h) = game_cfg
        .get()
        .map(|c| (c.window_width as f32, c.window_height as f32))
        .unwrap_or((DEFAULT_HALF_W * 2.0, DEFAULT_HALF_H * 2.0));
    let margin = enemy_cfg
        .get()
        .map(|c| c.spawn_margin)
        .unwrap_or(DEFAULT_SPAWN_MARGIN)
        .max(0.0);
    let half_w = win_w / 2.0 + margin;
    let half_h = win_h / 2.0 + margin;

    let cam_pos = camera_q
        .single()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);
    let spawn_pos = random_off_screen_position(cam_pos, half_w, half_h);

    // Collider radius: prefer RON config, fall back to constant.
    let radius = enemy_cfg
        .get()
        .map(|c| c.mini_boss.collider_radius)
        .unwrap_or(DEFAULT_COLLIDER_MINI_BOSS);

    // Mini-boss HP is not scaled by difficulty (fixed challenge per spawn).
    spawn_enemy(
        &mut commands,
        EnemyType::MiniBoss,
        spawn_pos,
        1.0,
        radius,
        enemy_cfg.get().map(|c| &c.mini_boss),
    );
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn random_off_screen_position(cam_pos: Vec2, half_w: f32, half_h: f32) -> Vec2 {
    use rand::RngExt;
    let mut rng = rand::rng();
    match rng.random_range(0..4u8) {
        0 => Vec2::new(
            cam_pos.x + rng.random_range(-half_w..half_w),
            cam_pos.y + half_h,
        ),
        1 => Vec2::new(
            cam_pos.x + rng.random_range(-half_w..half_w),
            cam_pos.y - half_h,
        ),
        2 => Vec2::new(
            cam_pos.x - half_w,
            cam_pos.y + rng.random_range(-half_h..half_h),
        ),
        _ => Vec2::new(
            cam_pos.x + half_w,
            cam_pos.y + rng.random_range(-half_h..half_h),
        ),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce as _;
    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::{
        components::Enemy,
        resources::{GameData, TreasureSpawner},
        states::AppState,
        types::EnemyType,
    };

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<AppState>();
        app.insert_resource(GameData::default());
        app.insert_resource(TreasureSpawner::default());
        app
    }

    fn mini_boss_count(app: &mut App) -> usize {
        let mut q = app.world_mut().query_filtered::<&Enemy, ()>();
        q.iter(app.world())
            .filter(|e| e.enemy_type == EnemyType::MiniBoss)
            .count()
    }

    /// Timer below threshold — no mini-boss spawned.
    #[test]
    fn no_spawn_before_interval() {
        let mut app = build_app();
        app.world_mut()
            .resource_mut::<TreasureSpawner>()
            .spawn_timer = MINI_BOSS_INTERVAL - 1.0;

        app.world_mut()
            .run_system_once(spawn_mini_boss)
            .expect("spawn_mini_boss should run");
        app.world_mut().flush();

        assert_eq!(mini_boss_count(&mut app), 0, "no spawn before threshold");
    }

    /// Timer at exactly the interval — mini-boss spawns and timer resets.
    #[test]
    fn spawn_at_interval() {
        let mut app = build_app();
        app.world_mut()
            .resource_mut::<TreasureSpawner>()
            .spawn_timer = MINI_BOSS_INTERVAL;

        app.world_mut()
            .run_system_once(spawn_mini_boss)
            .expect("spawn_mini_boss should run");
        app.world_mut().flush();

        assert_eq!(mini_boss_count(&mut app), 1, "one mini-boss should spawn");
        assert_eq!(
            app.world().resource::<TreasureSpawner>().spawn_timer,
            0.0,
            "timer must reset after spawn"
        );
    }

    /// When boss_spawned is true, no mini-boss spawns even if timer is ready.
    #[test]
    fn no_spawn_when_boss_already_spawned() {
        let mut app = build_app();
        {
            let mut gd = app.world_mut().resource_mut::<GameData>();
            gd.boss_spawned = true;
        }
        app.world_mut()
            .resource_mut::<TreasureSpawner>()
            .spawn_timer = MINI_BOSS_INTERVAL;

        app.world_mut()
            .run_system_once(spawn_mini_boss)
            .expect("spawn_mini_boss should run");
        app.world_mut().flush();

        assert_eq!(
            mini_boss_count(&mut app),
            0,
            "no spawn during final boss fight"
        );
    }

    /// Spawned mini-boss entity carries EnemyType::MiniBoss.
    #[test]
    fn spawned_entity_is_mini_boss_type() {
        let mut app = build_app();
        app.world_mut()
            .resource_mut::<TreasureSpawner>()
            .spawn_timer = MINI_BOSS_INTERVAL;

        app.world_mut()
            .run_system_once(spawn_mini_boss)
            .expect("spawn_mini_boss should run");
        app.world_mut().flush();

        let mut q = app.world_mut().query::<&Enemy>();
        let enemy = q.single(app.world()).expect("one enemy entity expected");
        assert_eq!(
            enemy.enemy_type,
            EnemyType::MiniBoss,
            "spawned enemy must be MiniBoss"
        );
    }
}
