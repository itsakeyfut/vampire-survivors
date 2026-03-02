pub mod collectible;
pub mod enemy;
pub mod physics;
pub mod player;
pub mod weapon;

pub use collectible::*;
pub use enemy::*;
pub use physics::*;
pub use player::*;
pub use weapon::*;

use bevy::prelude::*;

/// Marker placed on every entity that belongs to an active game session
/// (player, enemies, projectiles, XP gems, whip effects, …).
///
/// A single `despawn_game_session` system queries this marker to clean up all
/// gameplay entities when the run ends (on entering [`crate::states::AppState::GameOver`],
/// [`crate::states::AppState::Victory`], or [`crate::states::AppState::Title`]).
/// Using one marker instead of per-type despawn calls keeps the cleanup logic
/// in one place and ensures newly added entity types are automatically covered.
#[derive(Component)]
pub struct GameSessionEntity;
