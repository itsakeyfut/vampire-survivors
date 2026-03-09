pub mod apply;
pub mod attraction;
pub mod choices;
pub mod drop;
pub mod evolution;
pub mod level_up;
pub mod treasure;

use bevy::prelude::*;

use crate::states::AppState;

pub struct XpPlugin;

impl Plugin for XpPlugin {
    fn build(&self, app: &mut App) {
        use crate::systems::damage::apply_damage_to_enemies;
        use crate::systems::player::player_movement;
        use crate::systems::player::spawn_player;
        use crate::systems::xp::apply::{apply_selected_upgrade, recalculate_player_stats};
        use crate::systems::xp::attraction::{attract_gems_to_player, move_attracted_gems};
        use crate::systems::xp::choices::generate_level_up_choices;
        use crate::systems::xp::drop::{drop_treasure_on_mini_boss_death, spawn_xp_gems};
        use crate::systems::xp::level_up::check_level_up;
        use crate::systems::xp::treasure::{apply_evolution, open_treasure_chests};
        app.add_observer(apply_evolution)
            .add_systems(
                OnEnter(AppState::Playing),
                apply_selected_upgrade.after(spawn_player),
            )
            .add_systems(OnEnter(AppState::LevelUp), generate_level_up_choices)
            .add_systems(
                Update,
                (
                    recalculate_player_stats,
                    spawn_xp_gems.after(apply_damage_to_enemies),
                    drop_treasure_on_mini_boss_death.after(apply_damage_to_enemies),
                    attract_gems_to_player.after(player_movement),
                    move_attracted_gems.after(attract_gems_to_player),
                    check_level_up.after(move_attracted_gems),
                    open_treasure_chests,
                )
                    .run_if(in_state(AppState::Playing)),
            );
    }
}
