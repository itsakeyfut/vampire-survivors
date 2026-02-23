use std::collections::HashMap;

use bevy::prelude::*;

use crate::types::{AIType, EnemyType, PassiveState, WeaponState, WeaponType};

// ---------------------------------------------------------------------------
// Player components
// ---------------------------------------------------------------------------

/// Marker component identifying the player entity.
#[derive(Component, Debug)]
pub struct Player;

/// All mutable player statistics. Passive items modify these values.
#[derive(Component, Debug, Clone)]
pub struct PlayerStats {
    pub max_hp: f32,
    pub current_hp: f32,
    /// Base movement speed in pixels/second.
    pub move_speed: f32,
    /// Multiplicative damage bonus (1.0 = no bonus).
    pub damage_multiplier: f32,
    /// Fraction of cooldown removed (0.0–0.9).
    pub cooldown_reduction: f32,
    /// Projectile speed multiplier (1.0 = base speed).
    pub projectile_speed_mult: f32,
    /// Weapon duration multiplier (1.0 = base duration).
    pub duration_multiplier: f32,
    /// Radius within which XP gems are magnetically attracted.
    pub pickup_radius: f32,
    /// Weapon area-of-effect radius multiplier (1.0 = base area).
    pub area_multiplier: f32,
    /// Additional projectiles fired per activation (additive).
    pub extra_projectiles: u32,
    /// Luck multiplier affecting drop chances (1.0 = base luck).
    pub luck: f32,
    /// HP regeneration per second.
    pub hp_regen: f32,
}

impl Default for PlayerStats {
    fn default() -> Self {
        use crate::constants::*;
        Self {
            max_hp: PLAYER_BASE_HP,
            current_hp: PLAYER_BASE_HP,
            move_speed: PLAYER_BASE_SPEED,
            damage_multiplier: PLAYER_BASE_DAMAGE_MULT,
            cooldown_reduction: PLAYER_BASE_COOLDOWN_REDUCTION,
            projectile_speed_mult: PLAYER_BASE_PROJECTILE_SPEED,
            duration_multiplier: PLAYER_BASE_DURATION_MULT,
            pickup_radius: PLAYER_PICKUP_RADIUS,
            area_multiplier: PLAYER_BASE_AREA_MULT,
            extra_projectiles: 0,
            luck: PLAYER_BASE_LUCK,
            hp_regen: PLAYER_BASE_HP_REGEN,
        }
    }
}

/// All weapons currently carried by the player (max 6).
#[derive(Component, Debug, Default)]
pub struct WeaponInventory {
    pub weapons: Vec<WeaponState>,
}

/// All passive items currently carried by the player (max 6).
#[derive(Component, Debug, Default)]
pub struct PassiveInventory {
    pub items: Vec<PassiveState>,
}

/// Invincibility timer after taking damage; entity is immune while > 0.
#[derive(Component, Debug)]
pub struct InvincibilityTimer {
    /// Remaining invincibility duration in seconds.
    pub remaining: f32,
}

// ---------------------------------------------------------------------------
// Enemy components
// ---------------------------------------------------------------------------

/// Core enemy stats. Attached to every enemy entity.
#[derive(Component, Debug, Clone)]
pub struct Enemy {
    pub enemy_type: EnemyType,
    pub max_hp: f32,
    pub current_hp: f32,
    /// Movement speed in pixels/second.
    pub move_speed: f32,
    /// Contact damage dealt to the player per hit.
    pub damage: f32,
    /// XP gem value dropped on death.
    pub xp_value: u32,
    /// Probability (0.0–1.0) of dropping a gold coin on death.
    pub gold_chance: f32,
}

/// Drives enemy movement and attack behavior.
#[derive(Component, Debug)]
pub struct EnemyAI {
    pub ai_type: AIType,
    /// Timer between ranged attacks (used by KeepDistance AI).
    pub attack_timer: f32,
    /// Maximum distance at which this enemy will attack.
    pub attack_range: f32,
}

/// Brief color flash applied when an enemy takes damage.
#[derive(Component, Debug)]
pub struct DamageFlash {
    /// Remaining flash duration in seconds.
    pub timer: f32,
}

// ---------------------------------------------------------------------------
// Projectile / weapon-effect components
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Collectible components
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Physics / utility components
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_stats_default_values() {
        let stats = PlayerStats::default();
        assert_eq!(stats.max_hp, crate::constants::PLAYER_BASE_HP);
        assert_eq!(stats.current_hp, crate::constants::PLAYER_BASE_HP);
        assert_eq!(stats.move_speed, crate::constants::PLAYER_BASE_SPEED);
        assert_eq!(stats.damage_multiplier, 1.0);
        assert_eq!(stats.extra_projectiles, 0);
    }

    #[test]
    fn weapon_inventory_starts_empty() {
        let inv = WeaponInventory::default();
        assert!(inv.weapons.is_empty());
    }

    #[test]
    fn passive_inventory_starts_empty() {
        let inv = PassiveInventory::default();
        assert!(inv.items.is_empty());
    }
}
