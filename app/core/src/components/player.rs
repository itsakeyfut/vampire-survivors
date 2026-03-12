use bevy::prelude::*;

use crate::types::{PassiveState, WeaponState, WhipSide};

// ---------------------------------------------------------------------------
// Fallback constants (used when RON config is not yet loaded)
// ---------------------------------------------------------------------------

pub(crate) const DEFAULT_PLAYER_BASE_HP: f32 = 100.0;
pub(crate) const DEFAULT_PLAYER_BASE_SPEED: f32 = 200.0;
const DEFAULT_PLAYER_BASE_DAMAGE_MULT: f32 = 1.0;
const DEFAULT_PLAYER_BASE_XP_MULT: f32 = 1.0;
const DEFAULT_PLAYER_BASE_COOLDOWN_REDUCTION: f32 = 0.0;
const DEFAULT_PLAYER_BASE_PROJECTILE_SPEED: f32 = 1.0;
const DEFAULT_PLAYER_BASE_DURATION_MULT: f32 = 1.0;
const DEFAULT_PLAYER_BASE_AREA_MULT: f32 = 1.0;
const DEFAULT_PLAYER_BASE_LUCK: f32 = 1.0;
const DEFAULT_PLAYER_BASE_HP_REGEN: f32 = 0.0;
const DEFAULT_PLAYER_PICKUP_RADIUS: f32 = 80.0;
const DEFAULT_GEM_ATTRACTION_SPEED: f32 = 200.0;
const DEFAULT_GEM_ABSORPTION_RADIUS: f32 = 8.0;

/// Marker component identifying the player entity.
#[derive(Component, Debug)]
pub struct Player;

/// Tracks which side the Whip last struck.
///
/// Attached to the player entity and flipped by
/// [`crate::systems::weapons::whip::fire_whip`] after each activation so that
/// consecutive swings alternate left and right.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerWhipSide(pub WhipSide);

/// The direction the player is currently facing (or last moved toward).
///
/// Updated by [`crate::systems::player::player_movement`] whenever the player
/// moves.  Defaults to `Vec2::X` (facing right) so directional weapons like
/// Knife have a valid target even before the player's first input.
#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerFacingDirection(pub Vec2);

impl Default for PlayerFacingDirection {
    fn default() -> Self {
        Self(Vec2::X)
    }
}

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
    /// Speed at which attracted XP gems move toward the player (pixels/second).
    pub gem_attraction_speed: f32,
    /// Distance at which an attracted gem is absorbed by the player (pixels).
    pub gem_absorption_radius: f32,
    /// Weapon area-of-effect radius multiplier (1.0 = base area).
    pub area_multiplier: f32,
    /// Additional projectiles fired per activation (additive).
    pub extra_projectiles: u32,
    /// Luck multiplier affecting drop chances (1.0 = base luck).
    pub luck: f32,
    /// HP regeneration per second.
    pub hp_regen: f32,
    /// XP gain multiplier (1.0 = base; boosted by the BonusXp meta upgrade).
    pub xp_multiplier: f32,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            max_hp: DEFAULT_PLAYER_BASE_HP,
            current_hp: DEFAULT_PLAYER_BASE_HP,
            move_speed: DEFAULT_PLAYER_BASE_SPEED,
            damage_multiplier: DEFAULT_PLAYER_BASE_DAMAGE_MULT,
            cooldown_reduction: DEFAULT_PLAYER_BASE_COOLDOWN_REDUCTION,
            projectile_speed_mult: DEFAULT_PLAYER_BASE_PROJECTILE_SPEED,
            duration_multiplier: DEFAULT_PLAYER_BASE_DURATION_MULT,
            pickup_radius: DEFAULT_PLAYER_PICKUP_RADIUS,
            gem_attraction_speed: DEFAULT_GEM_ATTRACTION_SPEED,
            gem_absorption_radius: DEFAULT_GEM_ABSORPTION_RADIUS,
            area_multiplier: DEFAULT_PLAYER_BASE_AREA_MULT,
            extra_projectiles: 0,
            luck: DEFAULT_PLAYER_BASE_LUCK,
            hp_regen: DEFAULT_PLAYER_BASE_HP_REGEN,
            xp_multiplier: DEFAULT_PLAYER_BASE_XP_MULT,
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

/// Base player statistics at run start, before passive items are applied.
///
/// Stored on the player entity by [`crate::systems::player::spawn_player`] and
/// used by [`crate::systems::xp::apply::recalculate_player_stats`] to recompute
/// [`PlayerStats`] from scratch whenever [`PassiveInventory`] changes.
///
/// Does **not** include `current_hp` — the system preserves actual health when
/// recalculating, adjusting it by the change in `max_hp`.
#[derive(Component, Debug, Clone)]
pub struct BasePlayerStats {
    pub max_hp: f32,
    pub move_speed: f32,
    pub damage_multiplier: f32,
    pub cooldown_reduction: f32,
    pub projectile_speed_mult: f32,
    pub duration_multiplier: f32,
    pub pickup_radius: f32,
    pub gem_attraction_speed: f32,
    pub gem_absorption_radius: f32,
    pub area_multiplier: f32,
    pub extra_projectiles: u32,
    pub luck: f32,
    pub hp_regen: f32,
    /// XP gain multiplier (1.0 = base; boosted by the BonusXp meta upgrade).
    pub xp_multiplier: f32,
}

impl From<&PlayerStats> for BasePlayerStats {
    fn from(s: &PlayerStats) -> Self {
        Self {
            max_hp: s.max_hp,
            move_speed: s.move_speed,
            damage_multiplier: s.damage_multiplier,
            cooldown_reduction: s.cooldown_reduction,
            projectile_speed_mult: s.projectile_speed_mult,
            duration_multiplier: s.duration_multiplier,
            pickup_radius: s.pickup_radius,
            gem_attraction_speed: s.gem_attraction_speed,
            gem_absorption_radius: s.gem_absorption_radius,
            area_multiplier: s.area_multiplier,
            extra_projectiles: s.extra_projectiles,
            luck: s.luck,
            hp_regen: s.hp_regen,
            xp_multiplier: s.xp_multiplier,
        }
    }
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
        assert_eq!(stats.max_hp, DEFAULT_PLAYER_BASE_HP);
        assert_eq!(stats.current_hp, DEFAULT_PLAYER_BASE_HP);
        assert_eq!(stats.move_speed, DEFAULT_PLAYER_BASE_SPEED);
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
