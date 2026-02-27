//! Custom Bevy events for the Vampire Survivors clone.
//!
//! Events decouple systems: a producer sends an event, and one or more
//! consumers read it in a later system without direct inter-system coupling.

use bevy::prelude::*;

use crate::types::{EnemyType, WeaponType};

// ---------------------------------------------------------------------------
// Weapon events
// ---------------------------------------------------------------------------

/// Fired when a weapon's cooldown expires and it is ready to attack.
///
/// Consumers (weapon-specific fire systems) read this event and spawn the
/// appropriate projectiles, aura ticks, or instant-hit effects for the
/// given [`WeaponType`].
#[derive(Message, Debug, Clone)]
pub struct WeaponFiredEvent {
    /// The player entity that owns the weapon.
    pub player: Entity,
    /// Which weapon type fired.
    pub weapon_type: WeaponType,
    /// The weapon's current upgrade level at the time of firing.
    pub level: u8,
}

// ---------------------------------------------------------------------------
// Damage events
// ---------------------------------------------------------------------------

/// Fired when a weapon hits an enemy and should deal damage.
///
/// The [`apply_damage_to_enemies`](crate::systems::damage::apply_damage_to_enemies)
/// system reads this event each frame and applies the specified amount to the
/// target enemy's [`Enemy::current_hp`](crate::components::Enemy).
#[derive(Message, Debug, Clone)]
pub struct DamageEnemyEvent {
    /// The enemy entity to damage.
    pub entity: Entity,
    /// Raw damage amount (before resistances — none implemented yet).
    pub damage: f32,
    /// Which weapon type dealt this damage (for future effect routing).
    pub weapon_type: WeaponType,
}

/// Fired when an enemy's HP reaches zero and it is removed from the world.
///
/// Consumers use this event to spawn XP gems, gold coins, and other rewards
/// at the enemy's last known position.
#[derive(Message, Debug, Clone)]
pub struct EnemyDiedEvent {
    /// The entity that died (already despawned when consumers read this).
    pub entity: Entity,
    /// World-space position at the moment of death, for loot spawning.
    pub position: Vec2,
    /// The type of enemy that died, for loot-table lookups.
    pub enemy_type: EnemyType,
}

/// Fired when the player takes damage from an enemy or hazard.
///
/// Consumers apply the damage to [`PlayerStats::current_hp`](crate::components::PlayerStats),
/// trigger the invincibility timer, and play hurt effects.
#[derive(Message, Debug, Clone)]
pub struct PlayerDamagedEvent {
    /// The player entity that was hit.
    pub player: Entity,
    /// Raw damage amount to subtract from current HP.
    pub damage: f32,
}
