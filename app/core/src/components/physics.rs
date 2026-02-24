use bevy::prelude::*;

/// Circle collider used for manual collision detection.
#[derive(Component, Debug, Clone)]
pub struct CircleCollider {
    /// Collision radius in pixels.
    pub radius: f32,
}

/// Marks an item entity as currently being attracted toward the player.
#[derive(Component, Debug)]
pub struct AttractedToPlayer {
    /// Attraction movement speed in pixels/second.
    pub speed: f32,
}
