pub mod components;
pub mod config;
pub mod events;
pub mod resources;
pub mod states;
pub mod systems;
pub mod types;

use bevy::prelude::*;

use events::{
    BossSpawnedEvent, DamageEnemyEvent, EnemyDiedEvent, GameOverEvent, LevelUpEvent,
    PlayerDamagedEvent, VictoryEvent, WeaponFiredEvent,
};
use resources::{
    EnemySpawner, GameData, LevelUpChoices, MetaProgress, PendingUpgradeIndex, SelectedCharacter,
    SpatialGrid, TreasureSpawner,
};

/// Resets all per-run resources to their defaults at the start of each run.
///
/// Registered on [`OnTransition`] from [`AppState::CharacterSelect`] to
/// [`AppState::Playing`] so it fires **only** when a new run begins —
/// not when returning from [`AppState::LevelUp`] or [`AppState::Paused`],
/// which would wipe out level progress and pending upgrade choices.
///
/// [`MetaProgress`] and [`SelectedCharacter`] are intentionally excluded
/// because they persist across runs.
fn reset_per_run_resources(
    mut game_data: ResMut<GameData>,
    mut enemy_spawner: ResMut<EnemySpawner>,
    mut treasure_spawner: ResMut<TreasureSpawner>,
    mut level_up_choices: ResMut<LevelUpChoices>,
    mut pending_upgrade: ResMut<PendingUpgradeIndex>,
) {
    *game_data = GameData::default();
    *enemy_spawner = EnemySpawner::default();
    *treasure_spawner = TreasureSpawner::default();
    *level_up_choices = LevelUpChoices::default();
    *pending_upgrade = PendingUpgradeIndex::default();
}
use states::AppState;
use systems::{
    damage::apply_damage_to_enemies, enemies::EnemiesPlugin, game_over::GameOverPlugin,
    game_timer::TimerPlugin, kill_count::track_kill_count, player::PlayerPlugin,
    projectiles::ProjectilesPlugin, spatial::SpatialPlugin, victory::VictoryPlugin,
    weapons::WeaponsPlugin, xp::XpPlugin,
};

/// Core game plugin. Registers states, inserts default resources, and wires up
/// all gameplay systems.
pub struct GameCorePlugin;

impl Plugin for GameCorePlugin {
    fn build(&self, app: &mut App) {
        app
            // ---------------------------------------------------------------
            // State machine
            // ---------------------------------------------------------------
            .init_state::<AppState>()
            // ---------------------------------------------------------------
            // Per-run resources  (reset when a new run begins)
            // ---------------------------------------------------------------
            .insert_resource(GameData::default())
            .insert_resource(EnemySpawner::default())
            .insert_resource(TreasureSpawner::default())
            .insert_resource(SpatialGrid::default())
            .insert_resource(LevelUpChoices::default())
            .insert_resource(PendingUpgradeIndex::default())
            .insert_resource(SelectedCharacter::default())
            // ---------------------------------------------------------------
            // Persistent meta-progression (loaded from save/meta.json)
            // ---------------------------------------------------------------
            .insert_resource(MetaProgress::load())
            // ---------------------------------------------------------------
            // Events
            // ---------------------------------------------------------------
            .add_message::<WeaponFiredEvent>()
            .add_message::<DamageEnemyEvent>()
            .add_message::<EnemyDiedEvent>()
            .add_message::<PlayerDamagedEvent>()
            .add_message::<GameOverEvent>()
            .add_message::<VictoryEvent>()
            .add_message::<LevelUpEvent>()
            .add_message::<BossSpawnedEvent>()
            // ---------------------------------------------------------------
            // Per-run reset: fires only when a brand-new run begins.
            // Covers both entry paths — Title → Playing (when CharacterSelect
            // is skipped) and CharacterSelect → Playing (after character
            // selection).  LevelUp → Playing and Paused → Playing returns are
            // intentionally excluded so level progress and pending upgrade
            // choices are preserved.
            // ---------------------------------------------------------------
            .add_systems(
                OnTransition {
                    exited: AppState::Title,
                    entered: AppState::Playing,
                },
                reset_per_run_resources,
            )
            .add_systems(
                OnTransition {
                    exited: AppState::CharacterSelect,
                    entered: AppState::Playing,
                },
                reset_per_run_resources,
            )
            // ---------------------------------------------------------------
            // Sub-plugins (each owns its domain's systems)
            // ---------------------------------------------------------------
            .add_systems(
                Update,
                track_kill_count
                    .after(apply_damage_to_enemies)
                    .run_if(in_state(AppState::Playing)),
            )
            .add_plugins((
                TimerPlugin,
                SpatialPlugin,
                PlayerPlugin,
                EnemiesPlugin,
                WeaponsPlugin,
                ProjectilesPlugin,
                XpPlugin,
                GameOverPlugin,
                VictoryPlugin,
            ));
    }
}
