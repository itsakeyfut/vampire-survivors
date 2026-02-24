pub mod components;
pub mod constants;
pub mod game;
pub mod player;
pub mod resources;
pub mod states;
pub mod systems;
pub mod types;

use bevy::prelude::*;

use game::update_game_timer;
use player::{player_movement, spawn_player};
use resources::{
    EnemySpawner, GameData, LevelUpChoices, MetaProgress, SelectedCharacter, SpatialGrid,
    TreasureSpawner,
};
use states::AppState;
use systems::difficulty::update_difficulty;
use systems::enemy_ai::move_enemies;
use systems::enemy_cull::cull_distant_enemies;
use systems::enemy_spawn::spawn_enemies;

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
            .insert_resource(SelectedCharacter::default())
            // ---------------------------------------------------------------
            // Persistent meta-progression (loaded from save/meta.json)
            // ---------------------------------------------------------------
            .insert_resource(MetaProgress::load())
            // ---------------------------------------------------------------
            // Playing state — player lifecycle
            // ---------------------------------------------------------------
            // StateScoped entities (camera + player) are despawned automatically
            // on OnExit(Playing), so no explicit cleanup system is needed.
            .add_systems(OnEnter(AppState::Playing), spawn_player)
            // ---------------------------------------------------------------
            // Playing state — per-frame gameplay systems
            // ---------------------------------------------------------------
            .add_systems(
                Update,
                (
                    player_movement,
                    update_game_timer,
                    update_difficulty.after(update_game_timer),
                    spawn_enemies.after(update_difficulty),
                    move_enemies.after(player_movement),
                    cull_distant_enemies
                        .after(move_enemies)
                        .after(spawn_enemies),
                )
                    .run_if(in_state(AppState::Playing)),
            );
    }
}
