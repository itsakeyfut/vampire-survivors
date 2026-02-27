pub mod components;
pub mod config;
pub mod events;
pub mod resources;
pub mod states;
pub mod systems;
pub mod types;

use bevy::prelude::*;

use events::{DamageEnemyEvent, EnemyDiedEvent, PlayerDamagedEvent, WeaponFiredEvent};
use resources::{
    EnemySpawner, GameData, LevelUpChoices, MetaProgress, SelectedCharacter, SpatialGrid,
    TreasureSpawner,
};
use states::AppState;
use systems::damage::apply_damage_to_enemies;
use systems::difficulty::update_difficulty;
use systems::enemy_ai::move_enemies;
use systems::enemy_cull::cull_distant_enemies;
use systems::enemy_spawn::spawn_enemies;
use systems::game_timer::update_game_timer;
use systems::player::{player_movement, spawn_player};
use systems::player_collision::{
    apply_damage_to_player, enemy_player_collision, tick_invincibility,
};
use systems::projectile::{despawn_expired_projectiles, move_projectiles};
use systems::projectile_collision::projectile_enemy_collision;
use systems::spatial::update_spatial_grid;
use systems::weapon_cooldown::tick_weapon_cooldowns;
use systems::weapon_magic_wand::fire_magic_wand;
use systems::weapon_whip::{despawn_whip_effects, fire_whip};

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
            // Events
            // ---------------------------------------------------------------
            .add_message::<WeaponFiredEvent>()
            .add_message::<DamageEnemyEvent>()
            .add_message::<EnemyDiedEvent>()
            .add_message::<PlayerDamagedEvent>()
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
                    tick_weapon_cooldowns.after(player_movement),
                    update_spatial_grid.after(move_enemies),
                    fire_magic_wand.after(tick_weapon_cooldowns),
                    fire_whip
                        .after(tick_weapon_cooldowns)
                        .after(update_spatial_grid),
                    projectile_enemy_collision
                        .after(update_spatial_grid)
                        .after(move_projectiles),
                    apply_damage_to_enemies
                        .after(fire_whip)
                        .after(fire_magic_wand)
                        .after(projectile_enemy_collision),
                    despawn_whip_effects,
                    move_projectiles,
                    despawn_expired_projectiles,
                    update_game_timer,
                    update_difficulty.after(update_game_timer),
                    spawn_enemies.after(update_difficulty),
                    tick_invincibility.before(enemy_player_collision),
                    enemy_player_collision.after(update_spatial_grid),
                    apply_damage_to_player.after(enemy_player_collision),
                    move_enemies.after(player_movement),
                    cull_distant_enemies
                        .after(move_enemies)
                        .after(spawn_enemies),
                )
                    .run_if(in_state(AppState::Playing)),
            );
    }
}
