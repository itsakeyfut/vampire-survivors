use serde::{Deserialize, Serialize};

/// All weapon types, including evolved forms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeaponType {
    // Base weapons
    /// Fan-shaped swing, alternating left/right.
    Whip,
    /// Fires homing projectile at nearest enemy.
    MagicWand,
    /// Fast piercing projectile in movement direction.
    Knife,
    /// Continuous damage aura around player.
    Garlic,
    /// Orbiting projectile that circles the player.
    Bible,
    /// Random lightning strikes on screen.
    ThunderRing,
    /// Boomerang that flies out and returns.
    Cross,
    /// Fireball targeting the highest-HP enemy.
    FireWand,

    // Evolved weapons (Lv8 base + required passive)
    /// Whip + HollowHeart
    BloodyTear,
    /// MagicWand + EmptyTome
    HolyWand,
    /// Knife + Bracer
    ThousandEdge,
    /// Garlic + Pummarola
    SoulEater,
    /// Bible + Spellbinder
    UnholyVespers,
    /// ThunderRing + Duplicator
    LightningRing,
}

/// Per-weapon runtime state stored inside `WeaponInventory`.
#[derive(Debug, Clone)]
pub struct WeaponState {
    pub weapon_type: WeaponType,
    /// Current weapon level (1–8).
    pub level: u8,
    /// Remaining cooldown in seconds before next activation.
    pub cooldown_timer: f32,
    /// Whether this weapon has already been evolved.
    pub evolved: bool,
}

/// All passive item types. Each has 5 upgrade levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PassiveItemType {
    /// +10% damage per level.
    Spinach,
    /// +10% move speed per level.
    Wings,
    /// +20% max HP per level. Enables Whip → BloodyTear evolution.
    HollowHeart,
    /// +10% luck per level.
    Clover,
    /// -8% cooldown per level. Enables MagicWand → HolyWand evolution.
    EmptyTome,
    /// +10% projectile speed per level. Enables Knife → ThousandEdge evolution.
    Bracer,
    /// +10% weapon duration per level. Enables Bible → UnholyVespers evolution.
    Spellbinder,
    /// +1 projectile count per level. Enables ThunderRing → LightningRing evolution.
    Duplicator,
    /// +0.5 HP regen/s per level. Enables Garlic → SoulEater evolution.
    Pummarola,
}

/// Per-passive runtime state stored inside `PassiveInventory`.
#[derive(Debug, Clone)]
pub struct PassiveState {
    pub item_type: PassiveItemType,
    /// Current upgrade level (1–5).
    pub level: u8,
}

/// Which side the Whip last struck; alternates each activation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhipSide {
    Left,
    Right,
}

impl WhipSide {
    /// Returns the opposite side.
    pub fn flip(&self) -> Self {
        match self {
            WhipSide::Left => WhipSide::Right,
            WhipSide::Right => WhipSide::Left,
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
    fn whip_side_flip() {
        assert_eq!(WhipSide::Left.flip(), WhipSide::Right);
        assert_eq!(WhipSide::Right.flip(), WhipSide::Left);
        // Double flip returns to original
        assert_eq!(WhipSide::Left.flip().flip(), WhipSide::Left);
    }

    #[test]
    fn weapon_type_is_copy() {
        let w = WeaponType::MagicWand;
        let _copy = w;
        let _original = w; // should not move
    }

    #[test]
    fn passive_item_type_is_copy() {
        let p = PassiveItemType::HollowHeart;
        let _copy = p;
        let _original = p;
    }
}
