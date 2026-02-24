use std::collections::HashMap;

use bevy::prelude::*;

use crate::types::WeaponType;

/// A flying projectile spawned by a weapon.
#[derive(Component, Debug)]
pub struct Projectile {
    pub damage: f32,
    /// How many more enemies this projectile can pierce (0 = no pierce).
    pub piercing: u32,
    /// Enemies already hit (prevents duplicate damage with piercing).
    pub hit_enemies: Vec<Entity>,
    /// Remaining lifetime in seconds.
    pub lifetime: f32,
    pub weapon_type: WeaponType,
}

/// Linear velocity of a projectile entity (pixels/second).
#[derive(Component, Debug)]
pub struct ProjectileVelocity(pub Vec2);

/// An orbiting weapon body (e.g. Bible). Attached to the player entity.
#[derive(Component, Debug)]
pub struct OrbitWeapon {
    pub damage: f32,
    /// Distance from the player center (pixels).
    pub orbit_radius: f32,
    /// Angular velocity (radians/second).
    pub orbit_speed: f32,
    /// Current orbital angle in radians.
    pub orbit_angle: f32,
    /// Per-enemy hit cooldown (seconds) to prevent damage spam.
    pub hit_cooldown: HashMap<Entity, f32>,
}

/// A damage aura that continuously harms nearby enemies (e.g. Garlic).
#[derive(Component, Debug)]
pub struct AuraWeapon {
    pub damage: f32,
    /// Aura radius in pixels.
    pub radius: f32,
    /// Accumulated time since the last damage tick.
    pub tick_timer: f32,
    /// How frequently damage is applied (seconds between ticks).
    pub tick_interval: f32,
}
