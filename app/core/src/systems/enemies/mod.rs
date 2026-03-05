pub mod ai;
pub mod cull;
pub mod difficulty;
pub mod dragon;
pub mod medusa;
pub mod spawn;

use bevy::prelude::*;

use crate::states::AppState;

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        use crate::systems::enemies::ai::move_enemies;
        use crate::systems::enemies::cull::cull_distant_enemies;
        use crate::systems::enemies::difficulty::update_difficulty;
        use crate::systems::enemies::dragon::{
            dragon_fireball_player_collision, move_dragon_fireballs, tick_dragon_attack,
        };
        use crate::systems::enemies::medusa::{
            medusa_projectile_player_collision, move_medusa_projectiles, tick_medusa_attack,
        };
        use crate::systems::enemies::spawn::spawn_enemies;
        use crate::systems::game_timer::update_game_timer;
        use crate::systems::player::collision::enemy_player_collision;
        use crate::systems::player::player_movement;
        app.add_systems(
            Update,
            (
                move_enemies.after(player_movement),
                update_difficulty.after(update_game_timer),
                spawn_enemies.after(update_difficulty),
                cull_distant_enemies
                    .after(move_enemies)
                    .after(spawn_enemies),
                tick_dragon_attack.after(move_enemies),
                move_dragon_fireballs,
                dragon_fireball_player_collision
                    .after(move_dragon_fireballs)
                    .after(enemy_player_collision),
                tick_medusa_attack.after(move_enemies),
                move_medusa_projectiles,
                // Run after enemy_player_collision so that if melee damage fires
                // first this system is ordered after it.  Note: both systems use
                // deferred Commands, so the InvincibilityTimer inserted by one is
                // not yet visible to the other within the same frame — same-frame
                // double-hit remains theoretically possible but is ordered
                // deterministically.
                medusa_projectile_player_collision
                    .after(move_medusa_projectiles)
                    .after(enemy_player_collision),
            )
                .run_if(in_state(AppState::Playing)),
        );
    }
}
