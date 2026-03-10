use bevy::prelude::*;

/// An XP gem dropped by a defeated enemy.
#[derive(Component, Debug)]
pub struct ExperienceGem {
    pub value: u32,
}

/// A gold coin dropped by a defeated enemy.
#[derive(Component, Debug)]
pub struct GoldCoin {
    pub value: u32,
}

/// A treasure chest spawned on the map.
#[derive(Component, Debug)]
pub struct Treasure;

/// Drives the white-flash-to-yellow transition played when a chest first spawns.
///
/// Inserted by [`spawn_treasure`] and removed by [`animate_treasure_spawn_flash`]
/// once `elapsed` reaches `duration`.
#[derive(Component, Debug)]
pub struct TreasureSpawnFlash {
    pub elapsed: f32,
    pub duration: f32,
}

/// Marker for the radial-glow overlay that is spawned as a child of each
/// [`Treasure`] entity.  Visibility is toggled by [`update_treasure_glow`]
/// based on the player's proximity.
#[derive(Component, Debug)]
pub struct TreasureGlow;
