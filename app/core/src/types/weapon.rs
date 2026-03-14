use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Fallback constants for base_damage() and base_count()
//
// These mirror the values in assets/config/weapons/*.ron and serve as
// fallbacks when the RON assets have not yet finished loading.
// ---------------------------------------------------------------------------

// --- Whip / MagicWand (linear: base + per_level × (level − 1)) ---
const DEFAULT_WHIP_BASE_DAMAGE: f32 = 20.0;
const DEFAULT_WHIP_DAMAGE_PER_LEVEL: f32 = 10.0;
const DEFAULT_MAGIC_WAND_BASE_DAMAGE: f32 = 20.0;
const DEFAULT_MAGIC_WAND_DAMAGE_PER_LEVEL: f32 = 10.0;

// --- Knife (step formula: base + step × floor((level−1) / 2)) ---
const DEFAULT_KNIFE_BASE_DAMAGE: f32 = 15.0;
const DEFAULT_KNIFE_DAMAGE_PER_TWO_LEVELS: f32 = 5.0;
const DEFAULT_KNIFE_COUNT_BY_LEVEL: [u32; 8] = [1, 1, 2, 2, 3, 3, 4, 5];

// --- Fixed per-level damage tables (Lv1..Lv8) ---
const DEFAULT_GARLIC_DAMAGE_BY_LEVEL: [f32; 8] = [5.0, 5.0, 8.0, 8.0, 10.0, 12.0, 15.0, 20.0];
const DEFAULT_BIBLE_DAMAGE_BY_LEVEL: [f32; 8] = [20.0, 25.0, 30.0, 35.0, 40.0, 50.0, 60.0, 80.0];
const DEFAULT_THUNDER_RING_DAMAGE_BY_LEVEL: [f32; 8] =
    [40.0, 50.0, 60.0, 60.0, 70.0, 80.0, 90.0, 100.0];
const DEFAULT_CROSS_DAMAGE_BY_LEVEL: [f32; 8] = [50.0, 60.0, 70.0, 80.0, 90.0, 110.0, 130.0, 160.0];
const DEFAULT_FIRE_WAND_DAMAGE_BY_LEVEL: [f32; 8] =
    [80.0, 100.0, 120.0, 150.0, 180.0, 220.0, 270.0, 330.0];

// --- Fixed per-level count tables (Lv1..Lv8) ---
const DEFAULT_BIBLE_COUNT_BY_LEVEL: [u32; 8] = [1, 1, 2, 2, 3, 3, 3, 3];
const DEFAULT_THUNDER_RING_COUNT_BY_LEVEL: [u32; 8] = [1, 1, 2, 2, 3, 3, 3, 4];
const DEFAULT_CROSS_COUNT_BY_LEVEL: [u32; 8] = [1, 1, 1, 1, 2, 2, 2, 2];

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
        let level = self.level.clamp(1, 8);
        match self.weapon_type {
            WeaponType::Whip => match level {
                1..=3 => 1.00,
                4..=5 => 0.80,
                6..=7 => 0.70,
                _ => 0.60, // level 8
            },
            WeaponType::MagicWand => match level {
                1..=3 => 0.50,
                4..=5 => 0.40,
                6..=7 => 0.35,
                _ => 0.30, // level 8
            },
            WeaponType::Knife => match level {
                1 => 0.30,
                2..=3 => 0.25,
                4..=5 => 0.20,
                6..=7 => 0.18,
                _ => 0.15, // level 8
            },
            WeaponType::Garlic => match level {
                1..=3 => 0.50,
                4 => 0.45,
                5..=6 => 0.40,
                7 => 0.35,
                _ => 0.30, // level 8
            },
            // Bible orbits continuously; this cooldown gates spawning a new
            // orbit body when the weapon is first activated or levelled up.
            WeaponType::Bible => 1.00,
            WeaponType::ThunderRing => match level {
                1 => 2.00,
                2..=3 => 1.70,
                4..=5 => 1.50,
                6..=7 => 1.30,
                _ => 1.00, // level 8
            },
            WeaponType::Cross => match level {
                1 => 1.50,
                2..=3 => 1.30,
                4 => 1.20,
                5 => 1.10,
                6 => 1.00,
                7 => 0.90,
                _ => 0.80, // level 8
            },
            WeaponType::FireWand => match level {
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

    /// Base damage dealt per activation at the current weapon level.
    ///
    /// These values mirror the per-level tables in the RON config files and
    /// the formulas used by each weapon's fire system.  Multiplying by
    /// [`PlayerStats::damage_multiplier`] gives the effective damage dealt.
    ///
    /// | Weapon      | Lv1 | Lv2 | Lv3 | Lv4 | Lv5 | Lv6 | Lv7 | Lv8  |
    /// |-------------|-----|-----|-----|-----|-----|-----|-----|------|
    /// | Whip        | 20  | 30  | 40  | 50  | 60  | 70  | 80  | 90   |
    /// | MagicWand   | 20  | 30  | 40  | 50  | 60  | 70  | 80  | 90   |
    /// | Knife       | 15  | 15  | 20  | 20  | 25  | 25  | 30  | 30   |
    /// | Garlic      | 5   | 5   | 8   | 8   | 10  | 12  | 15  | 20   |
    /// | Bible       | 20  | 25  | 30  | 35  | 40  | 50  | 60  | 80   |
    /// | ThunderRing | 40  | 50  | 60  | 60  | 70  | 80  | 90  | 100  |
    /// | Cross       | 50  | 60  | 70  | 80  | 90  | 110 | 130 | 160  |
    /// | FireWand    | 80  | 100 | 120 | 150 | 180 | 220 | 270 | 330  |
    pub fn base_damage(&self) -> f32 {
        let level = self.level.clamp(1, 8) as usize;
        match self.weapon_type {
            // Linear scaling: base + per_level × (level − 1).
            WeaponType::Whip => {
                DEFAULT_WHIP_BASE_DAMAGE + DEFAULT_WHIP_DAMAGE_PER_LEVEL * (level as f32 - 1.0)
            }
            WeaponType::MagicWand => {
                DEFAULT_MAGIC_WAND_BASE_DAMAGE
                    + DEFAULT_MAGIC_WAND_DAMAGE_PER_LEVEL * (level as f32 - 1.0)
            }
            // Step formula: base + step × floor((level−1) / 2).
            WeaponType::Knife => {
                DEFAULT_KNIFE_BASE_DAMAGE
                    + DEFAULT_KNIFE_DAMAGE_PER_TWO_LEVELS * ((level - 1) / 2) as f32
            }
            // Fixed tables — values mirror the RON config files.
            WeaponType::Garlic => DEFAULT_GARLIC_DAMAGE_BY_LEVEL[level - 1],
            WeaponType::Bible => DEFAULT_BIBLE_DAMAGE_BY_LEVEL[level - 1],
            WeaponType::ThunderRing => DEFAULT_THUNDER_RING_DAMAGE_BY_LEVEL[level - 1],
            WeaponType::Cross => DEFAULT_CROSS_DAMAGE_BY_LEVEL[level - 1],
            WeaponType::FireWand => DEFAULT_FIRE_WAND_DAMAGE_BY_LEVEL[level - 1],
            // Evolved weapons are fixed at their max-level power.
            WeaponType::BloodyTear => 90.0,
            WeaponType::HolyWand => 90.0,
            WeaponType::ThousandEdge => 30.0,
            WeaponType::SoulEater => 20.0,
            WeaponType::UnholyVespers => 80.0,
            WeaponType::LightningRing => 100.0,
        }
    }

    /// Number of projectiles (or activations) fired per cooldown cycle at the
    /// current weapon level.
    ///
    /// Does not include the bonus from [`PlayerStats::extra_projectiles`]; add
    /// that on top to get the effective count.
    ///
    /// | Weapon      | Lv1 | Lv2 | Lv3 | Lv4 | Lv5 | Lv6 | Lv7 | Lv8 |
    /// |-------------|-----|-----|-----|-----|-----|-----|-----|-----|
    /// | Whip        | 1   | 1   | 1   | 1   | 1   | 1   | 1   | 1   |
    /// | MagicWand   | 1   | 1   | 1   | 1   | 1   | 1   | 1   | 1   |
    /// | Knife       | 1   | 1   | 2   | 2   | 3   | 3   | 4   | 5   |
    /// | Garlic      | 1   | 1   | 1   | 1   | 1   | 1   | 1   | 1   |
    /// | Bible       | 1   | 1   | 2   | 2   | 3   | 3   | 3   | 3   |
    /// | ThunderRing | 1   | 1   | 2   | 2   | 3   | 3   | 3   | 4   |
    /// | Cross       | 1   | 1   | 1   | 1   | 2   | 2   | 2   | 2   |
    /// | FireWand    | 1   | 1   | 1   | 1   | 1   | 1   | 1   | 1   |
    pub fn base_count(&self) -> u32 {
        let level = self.level.clamp(1, 8) as usize;
        match self.weapon_type {
            // Single-projectile weapons.
            WeaponType::Whip
            | WeaponType::MagicWand
            | WeaponType::Garlic
            | WeaponType::FireWand => 1,
            // Per-level count tables — values mirror the RON config files.
            WeaponType::Knife => DEFAULT_KNIFE_COUNT_BY_LEVEL[level - 1],
            WeaponType::Bible => DEFAULT_BIBLE_COUNT_BY_LEVEL[level - 1],
            WeaponType::ThunderRing => DEFAULT_THUNDER_RING_COUNT_BY_LEVEL[level - 1],
            WeaponType::Cross => DEFAULT_CROSS_COUNT_BY_LEVEL[level - 1],
            // Evolved weapons — fixed at max-level count of their base.
            WeaponType::BloodyTear | WeaponType::HolyWand | WeaponType::SoulEater => 1,
            // ThousandEdge: weapon_knife.rs fires count * 2 for the evolved form,
            // so the effective count at Lv8 (base count 5 × 2) is 10.
            WeaponType::ThousandEdge => 10,
            WeaponType::UnholyVespers => 3,
            WeaponType::LightningRing => 4,
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

    /// `level = 0` (invalid) is clamped to 1, returning the same cooldown as
    /// level 1 instead of silently acting like max level.
    #[test]
    fn base_cooldown_clamps_level_zero_to_level_one() {
        let mut state = WeaponState::new(WeaponType::Whip);
        let level_one_cd = state.base_cooldown(); // level = 1

        state.level = 0; // force invalid level
        assert_eq!(
            state.base_cooldown(),
            level_one_cd,
            "level 0 should be clamped to level 1"
        );
    }

    // -----------------------------------------------------------------------
    // base_damage tests
    // -----------------------------------------------------------------------

    // The parity tests below load and deserialize the live RON config files at
    // compile time via `include_str!`, so any drift between the DEFAULT_*
    // fallback constants and `assets/config/weapons/*.ron` is caught at
    // compile/test time rather than at runtime.

    /// Whip base_damage() matches the RON config formula for all 8 levels.
    #[test]
    fn base_damage_whip_matches_ron_config() {
        use crate::config::weapon::{WhipConfig, WhipConfigPartial};
        let partial: WhipConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(include_str!(
                "../../../vampire-survivors/assets/config/weapons/whip.ron"
            ))
            .expect("whip.ron should parse");
        let cfg = WhipConfig::from(partial);
        let mut state = WeaponState::new(WeaponType::Whip);
        for level in 1..=8u8 {
            state.level = level;
            let expected = cfg.base_damage + cfg.damage_per_level * (level as f32 - 1.0);
            assert_eq!(
                state.base_damage(),
                expected,
                "Whip lv{level}: expected {expected}"
            );
        }
    }

    /// MagicWand base_damage() matches the RON config formula for all 8 levels.
    #[test]
    fn base_damage_magic_wand_matches_ron_config() {
        use crate::config::weapon::{MagicWandConfig, MagicWandConfigPartial};
        let partial: MagicWandConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(include_str!(
                "../../../vampire-survivors/assets/config/weapons/magic_wand.ron"
            ))
            .expect("magic_wand.ron should parse");
        let cfg = MagicWandConfig::from(partial);
        let mut state = WeaponState::new(WeaponType::MagicWand);
        for level in 1..=8u8 {
            state.level = level;
            let expected = cfg.base_damage + cfg.damage_per_level * (level as f32 - 1.0);
            assert_eq!(
                state.base_damage(),
                expected,
                "MagicWand lv{level}: expected {expected}"
            );
        }
    }

    /// Knife base_damage() matches the RON config step formula for all 8 levels.
    #[test]
    fn base_damage_knife_matches_ron_config() {
        use crate::config::weapon::{KnifeConfig, KnifeConfigPartial};
        let partial: KnifeConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(include_str!(
                "../../../vampire-survivors/assets/config/weapons/knife.ron"
            ))
            .expect("knife.ron should parse");
        let cfg = KnifeConfig::from(partial);
        let mut state = WeaponState::new(WeaponType::Knife);
        for level in 1..=8u8 {
            state.level = level;
            let step = ((level as usize - 1) / 2) as f32;
            let expected = cfg.base_damage + cfg.damage_per_two_levels * step;
            assert_eq!(
                state.base_damage(),
                expected,
                "Knife lv{level}: expected {expected}"
            );
        }
    }

    /// Garlic base_damage() matches the per-level table in garlic.ron.
    #[test]
    fn base_damage_garlic_matches_ron_config() {
        use crate::config::weapon::{GarlicConfig, GarlicConfigPartial};
        let partial: GarlicConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(include_str!(
                "../../../vampire-survivors/assets/config/weapons/garlic.ron"
            ))
            .expect("garlic.ron should parse");
        let cfg = GarlicConfig::from(partial);
        let mut state = WeaponState::new(WeaponType::Garlic);
        for (i, &expected) in cfg.damage_by_level.iter().enumerate() {
            state.level = (i + 1) as u8;
            assert_eq!(state.base_damage(), expected, "Garlic lv{}", i + 1);
        }
    }

    /// Bible base_damage() matches the per-level table in bible.ron.
    #[test]
    fn base_damage_bible_matches_ron_config() {
        use crate::config::weapon::{BibleConfig, BibleConfigPartial};
        let partial: BibleConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(include_str!(
                "../../../vampire-survivors/assets/config/weapons/bible.ron"
            ))
            .expect("bible.ron should parse");
        let cfg = BibleConfig::from(partial);
        let mut state = WeaponState::new(WeaponType::Bible);
        for (i, &expected) in cfg.damage_by_level.iter().enumerate() {
            state.level = (i + 1) as u8;
            assert_eq!(state.base_damage(), expected, "Bible lv{}", i + 1);
        }
    }

    /// ThunderRing base_damage() matches the per-level table in thunder_ring.ron.
    #[test]
    fn base_damage_thunder_ring_matches_ron_config() {
        use crate::config::weapon::{ThunderRingConfig, ThunderRingConfigPartial};
        let partial: ThunderRingConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(include_str!(
                "../../../vampire-survivors/assets/config/weapons/thunder_ring.ron"
            ))
            .expect("thunder_ring.ron should parse");
        let cfg = ThunderRingConfig::from(partial);
        let mut state = WeaponState::new(WeaponType::ThunderRing);
        for (i, &expected) in cfg.damage_by_level.iter().enumerate() {
            state.level = (i + 1) as u8;
            assert_eq!(state.base_damage(), expected, "ThunderRing lv{}", i + 1);
        }
    }

    /// Cross base_damage() matches the per-level table in cross.ron.
    #[test]
    fn base_damage_cross_matches_ron_config() {
        use crate::config::weapon::{CrossConfig, CrossConfigPartial};
        let partial: CrossConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(include_str!(
                "../../../vampire-survivors/assets/config/weapons/cross.ron"
            ))
            .expect("cross.ron should parse");
        let cfg = CrossConfig::from(partial);
        let mut state = WeaponState::new(WeaponType::Cross);
        for (i, &expected) in cfg.damage_by_level.iter().enumerate() {
            state.level = (i + 1) as u8;
            assert_eq!(state.base_damage(), expected, "Cross lv{}", i + 1);
        }
    }

    /// FireWand base_damage() matches the per-level table in fire_wand.ron.
    #[test]
    fn base_damage_fire_wand_matches_ron_config() {
        use crate::config::weapon::{FireWandConfig, FireWandConfigPartial};
        let partial: FireWandConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(include_str!(
                "../../../vampire-survivors/assets/config/weapons/fire_wand.ron"
            ))
            .expect("fire_wand.ron should parse");
        let cfg = FireWandConfig::from(partial);
        let mut state = WeaponState::new(WeaponType::FireWand);
        for (i, &expected) in cfg.damage_by_level.iter().enumerate() {
            state.level = (i + 1) as u8;
            assert_eq!(state.base_damage(), expected, "FireWand lv{}", i + 1);
        }
    }

    /// Damage increases (or stays equal) with each level for every base weapon.
    #[test]
    fn base_damage_never_decreases_with_level() {
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
        for weapon_type in base_weapons {
            let mut state = WeaponState::new(weapon_type);
            let mut prev = state.base_damage();
            for level in 2..=8u8 {
                state.level = level;
                let curr = state.base_damage();
                assert!(
                    curr >= prev,
                    "{weapon_type:?} lv{level} ({curr}) < lv{} ({prev})",
                    level - 1
                );
                prev = curr;
            }
        }
    }

    /// base_damage clamps level 0 to level 1.
    #[test]
    fn base_damage_clamps_level_zero() {
        let mut state = WeaponState::new(WeaponType::Whip);
        let lv1 = state.base_damage();
        state.level = 0;
        assert_eq!(state.base_damage(), lv1, "level 0 should clamp to level 1");
    }

    // -----------------------------------------------------------------------
    // base_count tests
    // -----------------------------------------------------------------------

    /// Single-projectile weapons always return 1.
    #[test]
    fn base_count_single_projectile_weapons() {
        for weapon_type in [
            WeaponType::Whip,
            WeaponType::MagicWand,
            WeaponType::Garlic,
            WeaponType::FireWand,
        ] {
            let mut state = WeaponState::new(weapon_type);
            for level in 1..=8u8 {
                state.level = level;
                assert_eq!(
                    state.base_count(),
                    1,
                    "{weapon_type:?} lv{level} should always be 1"
                );
            }
        }
    }

    /// Knife base_count() matches the per-level table in knife.ron.
    #[test]
    fn base_count_knife_matches_ron_config() {
        use crate::config::weapon::{KnifeConfig, KnifeConfigPartial};
        let partial: KnifeConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(include_str!(
                "../../../vampire-survivors/assets/config/weapons/knife.ron"
            ))
            .expect("knife.ron should parse");
        let cfg = KnifeConfig::from(partial);
        let mut state = WeaponState::new(WeaponType::Knife);
        for (i, &expected) in cfg.count_by_level.iter().enumerate() {
            state.level = (i + 1) as u8;
            assert_eq!(state.base_count(), expected, "Knife lv{}", i + 1);
        }
    }

    /// Bible base_count() matches the per-level table in bible.ron.
    #[test]
    fn base_count_bible_matches_ron_config() {
        use crate::config::weapon::{BibleConfig, BibleConfigPartial};
        let partial: BibleConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(include_str!(
                "../../../vampire-survivors/assets/config/weapons/bible.ron"
            ))
            .expect("bible.ron should parse");
        let cfg = BibleConfig::from(partial);
        let mut state = WeaponState::new(WeaponType::Bible);
        for (i, &expected) in cfg.count_by_level.iter().enumerate() {
            state.level = (i + 1) as u8;
            assert_eq!(state.base_count(), expected, "Bible lv{}", i + 1);
        }
    }

    /// ThunderRing base_count() matches the per-level table in thunder_ring.ron.
    #[test]
    fn base_count_thunder_ring_matches_ron_config() {
        use crate::config::weapon::{ThunderRingConfig, ThunderRingConfigPartial};
        let partial: ThunderRingConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(include_str!(
                "../../../vampire-survivors/assets/config/weapons/thunder_ring.ron"
            ))
            .expect("thunder_ring.ron should parse");
        let cfg = ThunderRingConfig::from(partial);
        let mut state = WeaponState::new(WeaponType::ThunderRing);
        for (i, &expected) in cfg.count_by_level.iter().enumerate() {
            state.level = (i + 1) as u8;
            assert_eq!(state.base_count(), expected, "ThunderRing lv{}", i + 1);
        }
    }

    /// Cross base_count() matches the per-level table in cross.ron.
    #[test]
    fn base_count_cross_matches_ron_config() {
        use crate::config::weapon::{CrossConfig, CrossConfigPartial};
        let partial: CrossConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(include_str!(
                "../../../vampire-survivors/assets/config/weapons/cross.ron"
            ))
            .expect("cross.ron should parse");
        let cfg = CrossConfig::from(partial);
        let mut state = WeaponState::new(WeaponType::Cross);
        for (i, &expected) in cfg.count_by_level.iter().enumerate() {
            state.level = (i + 1) as u8;
            assert_eq!(state.base_count(), expected, "Cross lv{}", i + 1);
        }
    }

    /// Count never decreases with level for any base weapon.
    #[test]
    fn base_count_never_decreases_with_level() {
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
        for weapon_type in base_weapons {
            let mut state = WeaponState::new(weapon_type);
            let mut prev = state.base_count();
            for level in 2..=8u8 {
                state.level = level;
                let curr = state.base_count();
                assert!(
                    curr >= prev,
                    "{weapon_type:?} lv{level} count ({curr}) < lv{} ({prev})",
                    level - 1
                );
                prev = curr;
            }
        }
    }

    /// base_count clamps level 0 to level 1.
    #[test]
    fn base_count_clamps_level_zero() {
        let mut state = WeaponState::new(WeaponType::Knife);
        let lv1 = state.base_count();
        state.level = 0;
        assert_eq!(state.base_count(), lv1, "level 0 should clamp to level 1");
    }
}
