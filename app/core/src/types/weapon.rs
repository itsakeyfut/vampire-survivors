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

impl WeaponState {
    /// Creates a new weapon at level 1 with no active cooldown.
    pub fn new(weapon_type: WeaponType) -> Self {
        Self {
            weapon_type,
            level: 1,
            cooldown_timer: 0.0,
            evolved: false,
        }
    }

    /// Base cooldown in seconds for this weapon at its current level.
    ///
    /// Values match the design document level tables. Passing these through
    /// [`Self::effective_cooldown`] applies the player's cooldown reduction.
    pub fn base_cooldown(&self) -> f32 {
        match self.weapon_type {
            WeaponType::Whip => match self.level {
                1..=3 => 1.00,
                4..=5 => 0.80,
                6..=7 => 0.70,
                _ => 0.60, // level 8
            },
            WeaponType::MagicWand => match self.level {
                1..=3 => 0.50,
                4..=5 => 0.40,
                6..=7 => 0.35,
                _ => 0.30, // level 8
            },
            WeaponType::Knife => match self.level {
                1 => 0.30,
                2..=3 => 0.25,
                4..=5 => 0.20,
                6..=7 => 0.18,
                _ => 0.15, // level 8
            },
            WeaponType::Garlic => match self.level {
                1..=3 => 0.50,
                4 => 0.45,
                5..=6 => 0.40,
                7 => 0.35,
                _ => 0.30, // level 8
            },
            // Bible orbits continuously; this cooldown gates spawning a new
            // orbit body when the weapon is first activated or levelled up.
            WeaponType::Bible => 1.00,
            WeaponType::ThunderRing => match self.level {
                1 => 2.00,
                2..=3 => 1.70,
                4..=5 => 1.50,
                6..=7 => 1.30,
                _ => 1.00, // level 8
            },
            WeaponType::Cross => match self.level {
                1 => 1.50,
                2..=3 => 1.30,
                4 => 1.20,
                5 => 1.10,
                6 => 1.00,
                7 => 0.90,
                _ => 0.80, // level 8
            },
            WeaponType::FireWand => match self.level {
                1 => 3.00,
                2 => 2.70,
                3 => 2.50,
                4 => 2.30,
                5 => 2.10,
                6 => 2.00,
                7 => 1.80,
                _ => 1.50, // level 8
            },
            // Evolved weapons — use tighter cooldowns befitting their power.
            WeaponType::BloodyTear => 0.50,
            WeaponType::HolyWand => 0.25,
            WeaponType::ThousandEdge => 0.12,
            WeaponType::SoulEater => 0.50,
            WeaponType::UnholyVespers => 0.80,
            WeaponType::LightningRing => 0.70,
        }
    }

    /// Effective cooldown after applying the player's cooldown reduction.
    ///
    /// `cooldown_reduction` is the fraction removed (e.g. `0.3` = 30% shorter).
    /// It is clamped to `[0.0, 0.9]` so the effective cooldown is always at
    /// least 10 % of the base value, preventing degenerate sub-frame intervals.
    pub fn effective_cooldown(&self, cooldown_reduction: f32) -> f32 {
        let factor = (1.0 - cooldown_reduction.clamp(0.0, 0.9)).max(0.1);
        self.base_cooldown() * factor
    }
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

    #[test]
    fn weapon_state_new_starts_at_level_one() {
        let state = WeaponState::new(WeaponType::Whip);
        assert_eq!(state.weapon_type, WeaponType::Whip);
        assert_eq!(state.level, 1);
        assert_eq!(state.cooldown_timer, 0.0);
        assert!(!state.evolved);
    }

    /// All 8 base weapons must be constructable and start at level 1.
    #[test]
    fn weapon_state_new_all_eight_base_weapons() {
        let base_weapons = [
            WeaponType::Whip,
            WeaponType::MagicWand,
            WeaponType::Knife,
            WeaponType::Garlic,
            WeaponType::Bible,
            WeaponType::ThunderRing,
            WeaponType::Cross,
            WeaponType::FireWand,
        ];
        assert_eq!(base_weapons.len(), 8, "exactly 8 base weapons required");
        for weapon_type in base_weapons {
            let state = WeaponState::new(weapon_type);
            assert_eq!(state.level, 1);
            assert_eq!(state.cooldown_timer, 0.0);
            assert!(!state.evolved);
        }
    }
}
