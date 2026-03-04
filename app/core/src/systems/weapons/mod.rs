pub mod bible;
pub mod cooldown;
pub mod cross;
pub mod fire_wand;
pub mod garlic;
pub mod knife;
pub mod magic_wand;
pub mod thunder_ring;
pub mod whip;

use bevy::prelude::*;

use crate::states::AppState;

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        use crate::systems::damage::apply_damage_to_enemies;
        use crate::systems::player::player_movement;
        use crate::systems::projectiles::collision::projectile_enemy_collision;
        use crate::systems::spatial::update_spatial_grid;
        use crate::systems::weapons::bible::{fire_bible, orbit_bible, spawn_bible_visual};
        use crate::systems::weapons::cooldown::tick_weapon_cooldowns;
        use crate::systems::weapons::cross::fire_cross;
        use crate::systems::weapons::fire_wand::{
            despawn_expired_fireballs, despawn_explosion_effects, fire_fire_wand,
            fireball_enemy_collision, move_fireballs,
        };
        use crate::systems::weapons::garlic::{
            fire_garlic, spawn_garlic_visual, update_garlic_visual,
        };
        use crate::systems::weapons::knife::fire_knife;
        use crate::systems::weapons::magic_wand::fire_magic_wand;
        use crate::systems::weapons::thunder_ring::{despawn_thunder_effects, fire_thunder_ring};
        use crate::systems::weapons::whip::{despawn_whip_effects, fire_whip};
        app.add_systems(
            Update,
            (
                tick_weapon_cooldowns.after(player_movement),
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
                move_fireballs,
                fireball_enemy_collision
                    .after(move_fireballs)
                    .after(update_spatial_grid),
                spawn_bible_visual,
                spawn_garlic_visual,
                update_garlic_visual,
                despawn_whip_effects,
                despawn_thunder_effects,
                despawn_explosion_effects,
                despawn_expired_fireballs,
            )
                .run_if(in_state(AppState::Playing)),
        )
        .add_systems(
            Update,
            apply_damage_to_enemies
                .after(fire_whip)
                .after(fire_garlic)
                .after(fire_magic_wand)
                .after(fire_knife)
                .after(orbit_bible)
                .after(fire_thunder_ring)
                .after(projectile_enemy_collision)
                .after(fireball_enemy_collision)
                .run_if(in_state(AppState::Playing)),
        );
    }
}
