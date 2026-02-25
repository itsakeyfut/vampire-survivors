//! Custom Bevy events for the Vampire Survivors clone.
//!
//! Events decouple systems: a producer sends an event, and one or more
//! consumers read it in a later system without direct inter-system coupling.

use bevy::prelude::*;

use crate::types::WeaponType;

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
