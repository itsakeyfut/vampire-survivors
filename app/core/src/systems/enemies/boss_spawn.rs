//! Boss spawn system — triggers Boss Death at the 30-minute mark.
//!
//! [`check_boss_spawn`] runs every frame during [`AppState::Playing`].
//! When `GameData.elapsed_time` first reaches `boss_spawn_time` (default
//! 1800 s from `game.ron`) it:
//!
//! 1. Sets `GameData.boss_spawned = true` to prevent re-entry.
//! 2. Sets `EnemySpawner.active = false` to stop normal enemy spawning.
//! 3. Emits a [`BossSpawnedEvent`] for UI and other listeners.
//! 4. Spawns the Boss Death entity just off-screen to the right of the player.

use bevy::prelude::*;

use crate::{
    components::{CircleCollider, Enemy, EnemyAI, GameSessionEntity, Player},
    config::GameParams,
    events::BossSpawnedEvent,
    resources::{EnemySpawner, GameData},
    types::{AIType, EnemyType},
};

// ---------------------------------------------------------------------------
// Fallback constants (used when RON config is not yet loaded)
// ---------------------------------------------------------------------------

/// Boss spawn time in seconds (30 minutes).
const DEFAULT_BOSS_SPAWN_TIME: f32 = 1800.0;
/// Collider radius for Boss Death in pixels.
const DEFAULT_BOSS_COLLIDER: f32 = 40.0;
/// Horizontal offset from the player when spawning the boss (pixels).
const BOSS_SPAWN_OFFSET_X: f32 = 700.0;

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Checks whether the 30-minute mark has been reached and spawns Boss Death.
///
/// This is a no-op once `game_data.boss_spawned` is `true`, so it is safe to
/// keep registered in `Update` without any per-frame cost after the trigger.
pub fn check_boss_spawn(
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
    mut enemy_spawner: ResMut<EnemySpawner>,
    mut boss_events: MessageWriter<BossSpawnedEvent>,
    game_cfg: GameParams,
    player_q: Query<&Transform, With<Player>>,
) {
    // Already spawned — nothing to do.
    if game_data.boss_spawned {
        return;
    }

    let spawn_time = game_cfg
        .get()
        .map(|c| c.boss_spawn_time)
        .unwrap_or(DEFAULT_BOSS_SPAWN_TIME);

    if game_data.elapsed_time < spawn_time {
        return;
    }

    // Mark boss spawned and stop normal enemy spawning.
    game_data.boss_spawned = true;
    enemy_spawner.active = false;
    boss_events.write(BossSpawnedEvent);

    // Spawn just off-screen to the right of the player; fall back to origin.
    let offset = Vec2::new(BOSS_SPAWN_OFFSET_X, 0.0);
    let spawn_pos = player_q
        .single()
        .map(|t| t.translation.truncate() + offset)
        .unwrap_or(offset);

    let difficulty = enemy_spawner.difficulty_multiplier.max(1.0);
    let enemy = Enemy::from_type(EnemyType::BossDeath, difficulty);

    commands.spawn((
        GameSessionEntity,
        enemy,
        EnemyAI {
            ai_type: AIType::BossMultiPhase,
            attack_timer: 0.0,
            attack_range: 300.0,
        },
        CircleCollider {
            radius: DEFAULT_BOSS_COLLIDER,
        },
        Sprite {
            color: Color::srgb(0.3, 0.0, 0.5),
            custom_size: Some(Vec2::splat(DEFAULT_BOSS_COLLIDER * 2.0)),
            ..default()
        },
        Transform::from_xyz(spawn_pos.x, spawn_pos.y, 5.0),
    ));
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::ecs::system::RunSystemOnce as _;
    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::resources::GameData;
    use crate::states::AppState;

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<AppState>();
        app.insert_resource(GameData::default());
        app.insert_resource(EnemySpawner::default());
        app.add_message::<BossSpawnedEvent>();
        app
    }

    fn boss_events(app: &App) -> Vec<BossSpawnedEvent> {
        let messages = app.world().resource::<Messages<BossSpawnedEvent>>();
        let mut cursor = messages.get_cursor();
        cursor.read(messages).cloned().collect()
    }

    /// Before the time threshold is reached, nothing happens.
    #[test]
    fn no_spawn_before_time_threshold() {
        let mut app = build_app();
        app.world_mut().resource_mut::<GameData>().elapsed_time = DEFAULT_BOSS_SPAWN_TIME - 1.0;

        app.world_mut()
            .run_system_once(check_boss_spawn)
            .expect("check_boss_spawn should run");

        assert!(boss_events(&app).is_empty(), "no event before threshold");
        assert!(
            !app.world().resource::<GameData>().boss_spawned,
            "boss_spawned should still be false"
        );
        assert!(
            app.world().resource::<EnemySpawner>().active,
            "spawner should still be active"
        );
    }

    /// When elapsed_time reaches the threshold the boss event fires.
    #[test]
    fn spawn_fires_event_at_threshold() {
        let mut app = build_app();
        app.world_mut().resource_mut::<GameData>().elapsed_time = DEFAULT_BOSS_SPAWN_TIME;

        app.world_mut()
            .run_system_once(check_boss_spawn)
            .expect("check_boss_spawn should run");

        assert_eq!(boss_events(&app).len(), 1, "exactly one BossSpawnedEvent");
    }

    /// After the boss spawns, `boss_spawned` is true and the spawner is inactive.
    #[test]
    fn spawn_sets_boss_spawned_flag_and_disables_spawner() {
        let mut app = build_app();
        app.world_mut().resource_mut::<GameData>().elapsed_time = DEFAULT_BOSS_SPAWN_TIME;

        app.world_mut()
            .run_system_once(check_boss_spawn)
            .expect("check_boss_spawn should run");

        assert!(
            app.world().resource::<GameData>().boss_spawned,
            "boss_spawned must be set to true"
        );
        assert!(
            !app.world().resource::<EnemySpawner>().active,
            "spawner must be deactivated"
        );
    }

    /// Calling the system a second time is a no-op (no double-spawn).
    #[test]
    fn no_double_spawn_when_already_spawned() {
        let mut app = build_app();
        {
            let mut gd = app.world_mut().resource_mut::<GameData>();
            gd.elapsed_time = DEFAULT_BOSS_SPAWN_TIME + 10.0;
            gd.boss_spawned = true;
        }

        app.world_mut()
            .run_system_once(check_boss_spawn)
            .expect("check_boss_spawn should run");

        assert!(
            boss_events(&app).is_empty(),
            "no event when already spawned"
        );
    }

    /// A Boss Death entity is spawned in the world.
    #[test]
    fn boss_entity_is_spawned() {
        let mut app = build_app();
        app.world_mut().resource_mut::<GameData>().elapsed_time = DEFAULT_BOSS_SPAWN_TIME;

        app.world_mut()
            .run_system_once(check_boss_spawn)
            .expect("check_boss_spawn should run");
        app.world_mut().flush();

        let mut q = app
            .world_mut()
            .query_filtered::<&Enemy, With<GameSessionEntity>>();
        let enemies: Vec<_> = q.iter(app.world()).collect();
        assert_eq!(enemies.len(), 1, "exactly one boss entity expected");
        assert_eq!(
            enemies[0].enemy_type,
            EnemyType::BossDeath,
            "spawned entity must be BossDeath"
        );
    }

    /// Elapsed time exactly at threshold triggers the spawn (boundary check).
    #[test]
    fn spawn_triggers_exactly_at_threshold() {
        let mut app = build_app();
        app.world_mut().resource_mut::<GameData>().elapsed_time = DEFAULT_BOSS_SPAWN_TIME;

        app.world_mut()
            .run_system_once(check_boss_spawn)
            .expect("check_boss_spawn should run");

        assert_eq!(boss_events(&app).len(), 1, "event at exact threshold");
    }

    /// Advance time via run_system_once to confirm the system is time-driven.
    #[test]
    fn timer_advance_triggers_boss_spawn() {
        let mut app = build_app();

        // Advance to just below threshold — nothing fires.
        app.world_mut().resource_mut::<GameData>().elapsed_time = DEFAULT_BOSS_SPAWN_TIME - 0.01;
        app.world_mut()
            .run_system_once(check_boss_spawn)
            .expect("run 1");
        assert!(boss_events(&app).is_empty());

        // Cross the threshold — boss spawns.
        app.world_mut().resource_mut::<GameData>().elapsed_time = DEFAULT_BOSS_SPAWN_TIME;
        app.world_mut()
            .run_system_once(check_boss_spawn)
            .expect("run 2");
        assert_eq!(boss_events(&app).len(), 1);
    }
}
