pub mod components;
pub mod constants;
pub mod resources;
pub mod states;
pub mod types;

use bevy::prelude::*;

use resources::{
    EnemySpawner, GameData, LevelUpChoices, MetaProgress, SelectedCharacter, SpatialGrid,
    TreasureSpawner,
};
use states::AppState;

/// Core game plugin. Registers states, inserts default resources, and will
/// add systems as subsequent phases are implemented.
pub struct GameCorePlugin;

impl Plugin for GameCorePlugin {
    fn build(&self, app: &mut App) {
        app
            // Game state machine
            .init_state::<AppState>()
            // Per-run resources (reset when a new run begins)
            .insert_resource(GameData::default())
            .insert_resource(EnemySpawner::default())
            .insert_resource(TreasureSpawner::default())
            .insert_resource(SpatialGrid::default())
            .insert_resource(LevelUpChoices::default())
            .insert_resource(SelectedCharacter::default())
            // Persistent meta-progression (loaded from save/meta.json)
            .insert_resource(MetaProgress::load());
    }
}
