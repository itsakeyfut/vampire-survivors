//! Gameplay systems for the Vampire Survivors clone.
//!
//! Each sub-module owns one logical concern so that systems can be wired,
//! tested, and reasoned about in isolation.

pub mod collision;
pub mod damage;
pub mod difficulty;
pub mod enemy_ai;
pub mod enemy_cull;
pub mod enemy_spawn;
pub mod game_over;
pub mod game_timer;
pub mod gem_attraction;
pub mod gem_drop;
pub mod level_up;
pub mod level_up_choices;
pub mod player;
pub mod player_collision;
pub mod projectile;
pub mod projectile_collision;
pub mod spatial;
pub mod weapon_cooldown;
pub mod weapon_magic_wand;
pub mod weapon_whip;
