//! Weapon evolution condition checks.
//!
//! A base weapon can evolve when **two** conditions are both met:
//!
//! 1. The weapon is at **level 8** (`weapon_state.level == 8`).
//! 2. The player owns **the required passive item** for that weapon (any level).
//!
//! Evolution is triggered when a treasure chest is opened.  The caller should
//! prefer evolution over other treasure rewards whenever at least one weapon
//! is eligible (see [`find_evolution`]).
//!
//! ## Evolution table
//!
//! | Base weapon  | Required passive | Evolved form    |
//! |--------------|------------------|-----------------|
//! | Whip         | HollowHeart      | BloodyTear      |
//! | MagicWand    | EmptyTome        | HolyWand        |
//! | Knife        | Bracer           | ThousandEdge    |
//! | Garlic       | Pummarola        | SoulEater       |
//! | Bible        | Spellbinder      | UnholyVespers   |
//! | ThunderRing  | Duplicator       | LightningRing   |
//! | Cross        | —                | (no evolution)  |
//! | FireWand     | —                | (no evolution)  |

use crate::{
    components::{PassiveInventory, WeaponInventory},
    types::{PassiveItemType, WeaponState, WeaponType},
};

// ---------------------------------------------------------------------------
// Pure query functions
// ---------------------------------------------------------------------------

/// Returns the passive item required to evolve `weapon`, or `None` if that
/// weapon has no evolution path.
pub fn get_evolution_requirement(weapon: WeaponType) -> Option<PassiveItemType> {
    match weapon {
        WeaponType::Whip => Some(PassiveItemType::HollowHeart),
        WeaponType::MagicWand => Some(PassiveItemType::EmptyTome),
        WeaponType::Knife => Some(PassiveItemType::Bracer),
        WeaponType::Garlic => Some(PassiveItemType::Pummarola),
        WeaponType::Bible => Some(PassiveItemType::Spellbinder),
        WeaponType::ThunderRing => Some(PassiveItemType::Duplicator),
        // Cross and FireWand have no evolution in the current scope.
        _ => None,
    }
}

/// Returns the evolved form of `weapon`.
///
/// # Panics
///
/// Panics in debug builds if `weapon` has no evolution path.  In release
/// builds the panic is elided; callers should only call this after
/// [`get_evolution_requirement`] returns `Some`.
pub fn get_evolved_weapon(weapon: WeaponType) -> WeaponType {
    match weapon {
        WeaponType::Whip => WeaponType::BloodyTear,
        WeaponType::MagicWand => WeaponType::HolyWand,
        WeaponType::Knife => WeaponType::ThousandEdge,
        WeaponType::Garlic => WeaponType::SoulEater,
        WeaponType::Bible => WeaponType::UnholyVespers,
        WeaponType::ThunderRing => WeaponType::LightningRing,
        other => panic!("{other:?} has no evolved form"),
    }
}

/// Checks whether a single weapon can evolve given the current passive
/// inventory.
///
/// Returns `Some(evolved_type)` when both conditions are met:
/// - `weapon_state.level == 8` and `!weapon_state.evolved`
/// - the required passive item is owned (at any level)
///
/// Returns `None` otherwise.
pub fn can_evolve_weapon(
    weapon_state: &WeaponState,
    passive_inventory: &PassiveInventory,
) -> Option<WeaponType> {
    if weapon_state.level < 8 || weapon_state.evolved {
        return None;
    }
    let required = get_evolution_requirement(weapon_state.weapon_type)?;
    let owned = passive_inventory
        .items
        .iter()
        .any(|p| p.item_type == required);
    if owned {
        Some(get_evolved_weapon(weapon_state.weapon_type))
    } else {
        None
    }
}

/// Scans the full weapon inventory and returns the first weapon that can
/// evolve, or `None` if none qualify.
///
/// This is the entry point used by the treasure-opening system to decide
/// whether to trigger an evolution instead of a generic reward.
pub fn find_evolution(
    weapon_inv: &WeaponInventory,
    passive_inv: &PassiveInventory,
) -> Option<WeaponType> {
    weapon_inv
        .weapons
        .iter()
        .find_map(|ws| can_evolve_weapon(ws, passive_inv))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PassiveState, WeaponState};

    fn make_passive_inv(items: &[PassiveItemType]) -> PassiveInventory {
        PassiveInventory {
            items: items
                .iter()
                .map(|&item_type| PassiveState {
                    item_type,
                    level: 1,
                })
                .collect(),
        }
    }

    fn make_weapon_inv(states: Vec<WeaponState>) -> WeaponInventory {
        WeaponInventory { weapons: states }
    }

    fn lv8(weapon_type: WeaponType) -> WeaponState {
        WeaponState {
            weapon_type,
            level: 8,
            cooldown_timer: 0.0,
            evolved: false,
        }
    }

    // --- get_evolution_requirement ---

    #[test]
    fn evolution_requirement_whip_needs_hollow_heart() {
        assert_eq!(
            get_evolution_requirement(WeaponType::Whip),
            Some(PassiveItemType::HollowHeart)
        );
    }

    #[test]
    fn evolution_requirement_magic_wand_needs_empty_tome() {
        assert_eq!(
            get_evolution_requirement(WeaponType::MagicWand),
            Some(PassiveItemType::EmptyTome)
        );
    }

    #[test]
    fn evolution_requirement_knife_needs_bracer() {
        assert_eq!(
            get_evolution_requirement(WeaponType::Knife),
            Some(PassiveItemType::Bracer)
        );
    }

    #[test]
    fn evolution_requirement_garlic_needs_pummarola() {
        assert_eq!(
            get_evolution_requirement(WeaponType::Garlic),
            Some(PassiveItemType::Pummarola)
        );
    }

    #[test]
    fn evolution_requirement_bible_needs_spellbinder() {
        assert_eq!(
            get_evolution_requirement(WeaponType::Bible),
            Some(PassiveItemType::Spellbinder)
        );
    }

    #[test]
    fn evolution_requirement_thunder_ring_needs_duplicator() {
        assert_eq!(
            get_evolution_requirement(WeaponType::ThunderRing),
            Some(PassiveItemType::Duplicator)
        );
    }

    #[test]
    fn evolution_requirement_cross_has_none() {
        assert_eq!(get_evolution_requirement(WeaponType::Cross), None);
    }

    #[test]
    fn evolution_requirement_fire_wand_has_none() {
        assert_eq!(get_evolution_requirement(WeaponType::FireWand), None);
    }

    // --- get_evolved_weapon ---

    #[test]
    fn get_evolved_weapon_all_six_paths() {
        assert_eq!(get_evolved_weapon(WeaponType::Whip), WeaponType::BloodyTear);
        assert_eq!(
            get_evolved_weapon(WeaponType::MagicWand),
            WeaponType::HolyWand
        );
        assert_eq!(
            get_evolved_weapon(WeaponType::Knife),
            WeaponType::ThousandEdge
        );
        assert_eq!(
            get_evolved_weapon(WeaponType::Garlic),
            WeaponType::SoulEater
        );
        assert_eq!(
            get_evolved_weapon(WeaponType::Bible),
            WeaponType::UnholyVespers
        );
        assert_eq!(
            get_evolved_weapon(WeaponType::ThunderRing),
            WeaponType::LightningRing
        );
    }

    // --- can_evolve_weapon ---

    #[test]
    fn can_evolve_returns_evolved_type_when_conditions_met() {
        let ws = lv8(WeaponType::Whip);
        let inv = make_passive_inv(&[PassiveItemType::HollowHeart]);
        assert_eq!(can_evolve_weapon(&ws, &inv), Some(WeaponType::BloodyTear));
    }

    #[test]
    fn can_evolve_returns_none_when_level_too_low() {
        let mut ws = lv8(WeaponType::Whip);
        ws.level = 7;
        let inv = make_passive_inv(&[PassiveItemType::HollowHeart]);
        assert_eq!(can_evolve_weapon(&ws, &inv), None);
    }

    #[test]
    fn can_evolve_returns_none_when_already_evolved() {
        let mut ws = lv8(WeaponType::Whip);
        ws.evolved = true;
        let inv = make_passive_inv(&[PassiveItemType::HollowHeart]);
        assert_eq!(can_evolve_weapon(&ws, &inv), None);
    }

    #[test]
    fn can_evolve_returns_none_when_passive_missing() {
        let ws = lv8(WeaponType::Whip);
        let inv = make_passive_inv(&[]); // no passives
        assert_eq!(can_evolve_weapon(&ws, &inv), None);
    }

    #[test]
    fn can_evolve_returns_none_when_wrong_passive_owned() {
        let ws = lv8(WeaponType::Whip);
        let inv = make_passive_inv(&[PassiveItemType::EmptyTome]); // wrong passive
        assert_eq!(can_evolve_weapon(&ws, &inv), None);
    }

    #[test]
    fn can_evolve_returns_none_for_no_evolution_weapon() {
        let ws = lv8(WeaponType::Cross);
        let inv = make_passive_inv(&[
            PassiveItemType::HollowHeart,
            PassiveItemType::EmptyTome,
            PassiveItemType::Bracer,
            PassiveItemType::Pummarola,
            PassiveItemType::Spellbinder,
            PassiveItemType::Duplicator,
        ]); // all passives present, still no evolution
        assert_eq!(can_evolve_weapon(&ws, &inv), None);
    }

    // --- find_evolution ---

    #[test]
    fn find_evolution_returns_first_eligible_weapon() {
        let inv = make_weapon_inv(vec![lv8(WeaponType::Whip), lv8(WeaponType::Knife)]);
        let passive = make_passive_inv(&[PassiveItemType::HollowHeart, PassiveItemType::Bracer]);
        // Whip is first in the list and eligible → returns BloodyTear
        assert_eq!(find_evolution(&inv, &passive), Some(WeaponType::BloodyTear));
    }

    #[test]
    fn find_evolution_skips_ineligible_weapons() {
        let mut ws_knife = lv8(WeaponType::Knife);
        ws_knife.level = 7; // not yet max level
        let inv = make_weapon_inv(vec![ws_knife, lv8(WeaponType::MagicWand)]);
        let passive = make_passive_inv(&[PassiveItemType::EmptyTome]);
        assert_eq!(find_evolution(&inv, &passive), Some(WeaponType::HolyWand));
    }

    #[test]
    fn find_evolution_returns_none_when_nothing_qualifies() {
        let inv = make_weapon_inv(vec![lv8(WeaponType::Whip)]);
        let passive = make_passive_inv(&[]); // no passive
        assert_eq!(find_evolution(&inv, &passive), None);
    }

    #[test]
    fn find_evolution_returns_none_for_empty_inventory() {
        let inv = make_weapon_inv(vec![]);
        let passive = make_passive_inv(&[PassiveItemType::HollowHeart]);
        assert_eq!(find_evolution(&inv, &passive), None);
    }
}
