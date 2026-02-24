use serde::{Deserialize, Serialize};

/// Playable character archetypes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CharacterType {
    /// Starting character, balanced stats.
    DefaultCharacter,
    /// Higher cooldown reduction, starts with MagicWand.
    Magician,
    /// Higher move speed, starts with Knife.
    Thief,
    /// Higher HP, starts with Whip.
    Knight,
}

/// Purchasable permanent upgrades in the gold shop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetaUpgradeType {
    /// Permanent max HP bonus.
    BonusHp,
    /// Permanent move speed bonus.
    BonusSpeed,
    /// Permanent damage bonus.
    BonusDamage,
    /// Permanent XP gain bonus.
    BonusXp,
    /// Unlock a new starting weapon option.
    StartingWeapon,
}
