use bevy::prelude::*;

use crate::types::{PassiveState, WeaponState};

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
