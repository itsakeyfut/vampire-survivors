use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Weapon types
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Passive item types
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Character types
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Meta-progression
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Enemy types
// ---------------------------------------------------------------------------

/// All enemy types, ordered by earliest appearance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnemyType {
    /// Appears from 0 min. Fast but fragile flier.
    Bat,
    /// Appears from 0 min. Basic melee enemy.
    Skeleton,
    /// Appears from 5 min. Slow but tanky.
    Zombie,
    /// Appears from 10 min. Passes through walls.
    Ghost,
    /// Appears from 15 min. High damage.
    Demon,
    /// Appears from 20 min. Ranged attacker, keeps distance.
    Medusa,
    /// Appears from 25 min. Charge attack.
    Dragon,
    /// Appears at 30 min. Final boss, multi-phase.
    BossDeath,
}

/// Enemy AI behavior mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AIType {
    /// Moves directly toward the player each frame.
    ChasePlayer,
    /// Maintains attack distance and fires ranged projectiles.
    KeepDistance,
    /// Charges at the player in a straight line.
    ChargeAttack,
    /// Multi-phase boss behavior.
    BossMultiPhase,
}

/// Boss fight phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossPhase {
    Phase1,
    Phase2,
    Phase3,
}

// ---------------------------------------------------------------------------
// Misc game types
// ---------------------------------------------------------------------------

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

    // -- EnemyType tests -----------------------------------------------------

    #[test]
    fn enemy_type_is_copy() {
        let e = EnemyType::Bat;
        let _copy = e;
        let _original = e; // should not move
    }

    #[test]
    fn enemy_type_all_eight_variants_exist() {
        // Ensure every variant listed in the spec compiles and is distinct.
        let variants = [
            EnemyType::Bat,
            EnemyType::Skeleton,
            EnemyType::Zombie,
            EnemyType::Ghost,
            EnemyType::Demon,
            EnemyType::Medusa,
            EnemyType::Dragon,
            EnemyType::BossDeath,
        ];
        assert_eq!(variants.len(), 8);
        // All variants must be distinct (PartialEq).
        for i in 0..variants.len() {
            for j in 0..variants.len() {
                if i == j {
                    assert_eq!(variants[i], variants[j]);
                } else {
                    assert_ne!(variants[i], variants[j]);
                }
            }
        }
    }

    #[test]
    fn ai_type_is_copy() {
        let a = AIType::ChasePlayer;
        let _copy = a;
        let _original = a;
    }
}
