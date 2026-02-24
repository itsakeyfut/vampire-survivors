use crate::types::{PassiveItemType, WeaponType};

/// Content revealed when a treasure chest is opened.
#[derive(Debug, Clone)]
pub enum TreasureContent {
    /// Trigger a weapon evolution (requires base weapon at max level + passive).
    WeaponEvolution(WeaponType),
    /// Upgrade an existing weapon by one level.
    WeaponUpgrade(WeaponType),
    /// Add a new passive item (or upgrade existing one).
    PassiveItem(PassiveItemType),
    /// Gold reward.
    Gold(u32),
}

/// One option shown on the level-up card selection screen.
#[derive(Debug, Clone)]
pub enum UpgradeChoice {
    /// Acquire a new weapon.
    NewWeapon(WeaponType),
    /// Level up an already-owned weapon.
    WeaponUpgrade(WeaponType),
    /// Acquire a new passive item.
    PassiveItem(PassiveItemType),
    /// Level up an already-owned passive item.
    PassiveUpgrade(PassiveItemType),
}
