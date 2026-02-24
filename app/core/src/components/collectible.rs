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
