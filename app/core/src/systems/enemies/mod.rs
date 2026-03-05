pub mod ai;
pub mod cull;
pub mod difficulty;
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
        use crate::systems::enemies::medusa::{
            medusa_projectile_player_collision, move_medusa_projectiles, tick_medusa_attack,
        };
        use crate::systems::enemies::spawn::spawn_enemies;
        use crate::systems::game_timer::update_game_timer;
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
                tick_medusa_attack.after(move_enemies),
                move_medusa_projectiles,
                medusa_projectile_player_collision.after(move_medusa_projectiles),
            )
                .run_if(in_state(AppState::Playing)),
        );
    }
}
