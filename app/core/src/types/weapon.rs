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
            // Linear scaling: 20 + 10 per level above 1.
            WeaponType::Whip | WeaponType::MagicWand => 20.0 + 10.0 * (level as f32 - 1.0),
            // Steps every two levels: 15 + 5 * floor((level-1) / 2).
            WeaponType::Knife => 15.0 + 5.0 * ((level - 1) / 2) as f32,
            // Fixed tables matching the RON config files.
            WeaponType::Garlic => [5.0, 5.0, 8.0, 8.0, 10.0, 12.0, 15.0, 20.0][level - 1],
            WeaponType::Bible => [20.0, 25.0, 30.0, 35.0, 40.0, 50.0, 60.0, 80.0][level - 1],
            WeaponType::ThunderRing => [40.0, 50.0, 60.0, 60.0, 70.0, 80.0, 90.0, 100.0][level - 1],
            WeaponType::Cross => [50.0, 60.0, 70.0, 80.0, 90.0, 110.0, 130.0, 160.0][level - 1],
            WeaponType::FireWand => {
                [80.0, 100.0, 120.0, 150.0, 180.0, 220.0, 270.0, 330.0][level - 1]
            }
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
            // Per-level count tables matching the RON config files.
            WeaponType::Knife => [1, 1, 2, 2, 3, 3, 4, 5][level - 1],
            WeaponType::Bible => [1, 1, 2, 2, 3, 3, 3, 3][level - 1],
            WeaponType::ThunderRing => [1, 1, 2, 2, 3, 3, 3, 4][level - 1],
            WeaponType::Cross => [1, 1, 1, 1, 2, 2, 2, 2][level - 1],
            // Evolved weapons — fixed at max-level count of their base.
            WeaponType::BloodyTear | WeaponType::HolyWand | WeaponType::SoulEater => 1,
            WeaponType::ThousandEdge => 5,
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

    /// Whip and MagicWand scale linearly: 20 + 10*(level-1).
    #[test]
    fn base_damage_whip_magic_wand_linear() {
        for weapon_type in [WeaponType::Whip, WeaponType::MagicWand] {
            let mut state = WeaponState::new(weapon_type);
            assert_eq!(state.base_damage(), 20.0, "{weapon_type:?} lv1");
            state.level = 4;
            assert_eq!(state.base_damage(), 50.0, "{weapon_type:?} lv4");
            state.level = 8;
            assert_eq!(state.base_damage(), 90.0, "{weapon_type:?} lv8");
        }
    }

    /// Knife damage steps every two levels: 15 + 5*floor((level-1)/2).
    #[test]
    fn base_damage_knife_steps_every_two_levels() {
        let expected = [15.0, 15.0, 20.0, 20.0, 25.0, 25.0, 30.0, 30.0];
        let mut state = WeaponState::new(WeaponType::Knife);
        for (i, &expected_dmg) in expected.iter().enumerate() {
            state.level = (i + 1) as u8;
            assert_eq!(
                state.base_damage(),
                expected_dmg,
                "Knife lv{} expected {expected_dmg}",
                i + 1
            );
        }
    }

    /// Garlic damage matches the RON config table.
    #[test]
    fn base_damage_garlic_matches_table() {
        let expected = [5.0, 5.0, 8.0, 8.0, 10.0, 12.0, 15.0, 20.0];
        let mut state = WeaponState::new(WeaponType::Garlic);
        for (i, &expected_dmg) in expected.iter().enumerate() {
            state.level = (i + 1) as u8;
            assert_eq!(state.base_damage(), expected_dmg, "Garlic lv{}", i + 1);
        }
    }

    /// Bible damage matches the RON config table.
    #[test]
    fn base_damage_bible_matches_table() {
        let expected = [20.0, 25.0, 30.0, 35.0, 40.0, 50.0, 60.0, 80.0];
        let mut state = WeaponState::new(WeaponType::Bible);
        for (i, &expected_dmg) in expected.iter().enumerate() {
            state.level = (i + 1) as u8;
            assert_eq!(state.base_damage(), expected_dmg, "Bible lv{}", i + 1);
        }
    }

    /// ThunderRing damage matches the RON config table.
    #[test]
    fn base_damage_thunder_ring_matches_table() {
        let expected = [40.0, 50.0, 60.0, 60.0, 70.0, 80.0, 90.0, 100.0];
        let mut state = WeaponState::new(WeaponType::ThunderRing);
        for (i, &expected_dmg) in expected.iter().enumerate() {
            state.level = (i + 1) as u8;
            assert_eq!(state.base_damage(), expected_dmg, "ThunderRing lv{}", i + 1);
        }
    }

    /// Cross damage matches the RON config table.
    #[test]
    fn base_damage_cross_matches_table() {
        let expected = [50.0, 60.0, 70.0, 80.0, 90.0, 110.0, 130.0, 160.0];
        let mut state = WeaponState::new(WeaponType::Cross);
        for (i, &expected_dmg) in expected.iter().enumerate() {
            state.level = (i + 1) as u8;
            assert_eq!(state.base_damage(), expected_dmg, "Cross lv{}", i + 1);
        }
    }

    /// FireWand damage matches the RON config table.
    #[test]
    fn base_damage_fire_wand_matches_table() {
        let expected = [80.0, 100.0, 120.0, 150.0, 180.0, 220.0, 270.0, 330.0];
        let mut state = WeaponState::new(WeaponType::FireWand);
        for (i, &expected_dmg) in expected.iter().enumerate() {
            state.level = (i + 1) as u8;
            assert_eq!(state.base_damage(), expected_dmg, "FireWand lv{}", i + 1);
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

    /// Knife count matches the RON config table.
    #[test]
    fn base_count_knife_matches_table() {
        let expected = [1u32, 1, 2, 2, 3, 3, 4, 5];
        let mut state = WeaponState::new(WeaponType::Knife);
        for (i, &expected_count) in expected.iter().enumerate() {
            state.level = (i + 1) as u8;
            assert_eq!(state.base_count(), expected_count, "Knife lv{}", i + 1);
        }
    }

    /// Bible count matches the RON config table.
    #[test]
    fn base_count_bible_matches_table() {
        let expected = [1u32, 1, 2, 2, 3, 3, 3, 3];
        let mut state = WeaponState::new(WeaponType::Bible);
        for (i, &expected_count) in expected.iter().enumerate() {
            state.level = (i + 1) as u8;
            assert_eq!(state.base_count(), expected_count, "Bible lv{}", i + 1);
        }
    }

    /// ThunderRing count matches the RON config table.
    #[test]
    fn base_count_thunder_ring_matches_table() {
        let expected = [1u32, 1, 2, 2, 3, 3, 3, 4];
        let mut state = WeaponState::new(WeaponType::ThunderRing);
        for (i, &expected_count) in expected.iter().enumerate() {
            state.level = (i + 1) as u8;
            assert_eq!(
                state.base_count(),
                expected_count,
                "ThunderRing lv{}",
                i + 1
            );
        }
    }

    /// Cross count matches the RON config table.
    #[test]
    fn base_count_cross_matches_table() {
        let expected = [1u32, 1, 1, 1, 2, 2, 2, 2];
        let mut state = WeaponState::new(WeaponType::Cross);
        for (i, &expected_count) in expected.iter().enumerate() {
            state.level = (i + 1) as u8;
            assert_eq!(state.base_count(), expected_count, "Cross lv{}", i + 1);
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
