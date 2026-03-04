pub mod components;
pub mod config;
pub mod events;
pub mod resources;
pub mod states;
pub mod systems;
pub mod types;

use bevy::prelude::*;

use events::{
    DamageEnemyEvent, EnemyDiedEvent, GameOverEvent, LevelUpEvent, PlayerDamagedEvent,
    WeaponFiredEvent,
};
use resources::{
    EnemySpawner, GameData, LevelUpChoices, MetaProgress, PendingUpgradeIndex, SelectedCharacter,
    SpatialGrid, TreasureSpawner,
};
use states::AppState;
use systems::{
    enemies::EnemiesPlugin, game_over::GameOverPlugin, game_timer::TimerPlugin,
    player::PlayerPlugin, projectiles::ProjectilesPlugin, spatial::SpatialPlugin,
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
            .add_message::<LevelUpEvent>()
            // ---------------------------------------------------------------
            // Sub-plugins (each owns its domain's systems)
            // ---------------------------------------------------------------
            .add_plugins((
                TimerPlugin,
                SpatialPlugin,
                PlayerPlugin,
                EnemiesPlugin,
                WeaponsPlugin,
                ProjectilesPlugin,
                XpPlugin,
                GameOverPlugin,
            ));
    }
}
