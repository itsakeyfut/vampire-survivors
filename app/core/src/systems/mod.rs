//! Gameplay systems for the Vampire Survivors clone.
//!
//! Each sub-module owns one logical concern so that systems can be wired,
//! tested, and reasoned about in isolation.

pub mod collision;
pub mod damage;
pub mod enemies;
pub mod game_over;
pub mod game_timer;
pub mod kill_count;
pub mod player;
pub mod projectiles;
pub mod spatial;
pub mod victory;
pub mod weapons;
pub mod xp;
