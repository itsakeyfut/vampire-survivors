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
use systems::damage::apply_damage_to_enemies;
use systems::enemies::ai::move_enemies;
use systems::enemies::cull::cull_distant_enemies;
use systems::enemies::difficulty::update_difficulty;
use systems::enemies::spawn::spawn_enemies;
use systems::game_over::check_player_death;
use systems::game_timer::update_game_timer;
use systems::player::collision::{
    apply_damage_to_player, enemy_player_collision, tick_invincibility,
};
use systems::player::{despawn_game_session, player_movement, spawn_player};
use systems::projectiles::collision::projectile_enemy_collision;
use systems::projectiles::{despawn_expired_projectiles, move_projectiles};
use systems::spatial::update_spatial_grid;
use systems::weapons::bible::{fire_bible, orbit_bible, spawn_bible_visual};
use systems::weapons::cooldown::tick_weapon_cooldowns;
use systems::weapons::cross::{fire_cross, update_cross};
use systems::weapons::fire_wand::{
    despawn_expired_fireballs, despawn_explosion_effects, fire_fire_wand, fireball_enemy_collision,
    move_fireballs,
};
use systems::weapons::garlic::{fire_garlic, spawn_garlic_visual, update_garlic_visual};
use systems::weapons::knife::fire_knife;
use systems::weapons::magic_wand::fire_magic_wand;
use systems::weapons::thunder_ring::{despawn_thunder_effects, fire_thunder_ring};
use systems::weapons::whip::{despawn_whip_effects, fire_whip};
use systems::xp::apply::apply_selected_upgrade;
use systems::xp::attraction::{attract_gems_to_player, move_attracted_gems};
use systems::xp::choices::generate_level_up_choices;
use systems::xp::drop::spawn_xp_gems;
use systems::xp::level_up::check_level_up;

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
            // Player lifecycle
            // ---------------------------------------------------------------
            // spawn_player skips silently when a Player entity already exists,
            // so re-entering Playing from LevelUp / Paused is safe.
            // apply_selected_upgrade reads PendingUpgradeIndex; it is a no-op
            // when None (e.g. Title → Playing on game start).
            .add_systems(
                OnEnter(AppState::Playing),
                (spawn_player, apply_selected_upgrade).chain(),
            )
            // Clean up all gameplay entities when the run truly ends.
            // despawn_game_session removes every GameSessionEntity (player,
            // enemies, gems, projectiles, whip effects, …).
            .add_systems(OnEnter(AppState::GameOver), despawn_game_session)
            .add_systems(OnEnter(AppState::Victory), despawn_game_session)
            // Also on Title so a Paused → Title quit cleans up correctly.
            .add_systems(OnEnter(AppState::Title), despawn_game_session)
            // ---------------------------------------------------------------
            // LevelUp state — generate upgrade choices when overlay opens
            // ---------------------------------------------------------------
            // The player entity is still alive here (no DespawnOnExit on it),
            // so generate_level_up_choices can query WeaponInventory /
            // PassiveInventory to build a valid choice pool.
            .add_systems(OnEnter(AppState::LevelUp), generate_level_up_choices)
            // ---------------------------------------------------------------
            // Playing state — per-frame gameplay systems (part 1: movement,
            // weapons, collision, and damage)
            // ---------------------------------------------------------------
            .add_systems(
                Update,
                (
                    player_movement,
                    move_enemies.after(player_movement),
                    tick_weapon_cooldowns.after(player_movement),
                    update_spatial_grid.after(move_enemies),
                    fire_magic_wand.after(tick_weapon_cooldowns),
                    fire_knife.after(tick_weapon_cooldowns),
                    fire_bible.after(tick_weapon_cooldowns),
                    orbit_bible.after(fire_bible).after(update_spatial_grid),
                    fire_garlic
                        .after(tick_weapon_cooldowns)
                        .after(update_spatial_grid),
                    fire_thunder_ring.after(tick_weapon_cooldowns),
                    fire_cross.after(tick_weapon_cooldowns),
                    fire_fire_wand.after(tick_weapon_cooldowns),
                    fire_whip
                        .after(tick_weapon_cooldowns)
                        .after(update_spatial_grid),
                    move_projectiles,
                    update_cross.after(move_projectiles),
                    projectile_enemy_collision
                        .after(update_spatial_grid)
                        .after(move_projectiles)
                        .after(update_cross),
                    apply_damage_to_enemies
                        .after(fire_whip)
                        .after(fire_garlic)
                        .after(fire_magic_wand)
                        .after(fire_knife)
                        .after(orbit_bible)
                        .after(fire_thunder_ring)
                        .after(projectile_enemy_collision)
                        .after(fireball_enemy_collision),
                    tick_invincibility.before(enemy_player_collision),
                    enemy_player_collision.after(update_spatial_grid),
                    apply_damage_to_player.after(enemy_player_collision),
                )
                    .run_if(in_state(AppState::Playing)),
            )
            // Playing state — per-frame gameplay systems (part 2: XP, enemies,
            // timers, and death)
            .add_systems(
                Update,
                (
                    spawn_xp_gems.after(apply_damage_to_enemies),
                    attract_gems_to_player.after(player_movement),
                    move_attracted_gems.after(attract_gems_to_player),
                    update_game_timer,
                    update_difficulty.after(update_game_timer),
                    spawn_enemies.after(update_difficulty),
                    cull_distant_enemies
                        .after(move_enemies)
                        .after(spawn_enemies),
                    check_level_up
                        .after(move_attracted_gems)
                        .before(check_player_death),
                    check_player_death.after(apply_damage_to_player),
                )
                    .run_if(in_state(AppState::Playing)),
            )
            // Playing state — per-frame gameplay systems (part 2b: fireball
            // movement and collision; separate block to stay within the
            // 20-item system-tuple limit)
            .add_systems(
                Update,
                (
                    move_fireballs,
                    fireball_enemy_collision
                        .after(move_fireballs)
                        .after(update_spatial_grid),
                )
                    .run_if(in_state(AppState::Playing)),
            )
            // Playing state — per-frame gameplay systems (part 3: visuals and
            // entity cleanup; independent of damage/XP ordering)
            .add_systems(
                Update,
                (
                    spawn_bible_visual,
                    spawn_garlic_visual,
                    update_garlic_visual,
                    despawn_whip_effects,
                    despawn_thunder_effects,
                    despawn_expired_projectiles,
                    despawn_explosion_effects,
                    despawn_expired_fireballs,
                )
                    .run_if(in_state(AppState::Playing)),
            );
    }
}
