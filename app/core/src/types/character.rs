use serde::{Deserialize, Serialize};

use super::WeaponType;

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

/// Base statistics for a playable character.
///
/// Loaded from `assets/config/character.ron` at runtime via [`CharacterConfig`].
/// [`get_character_stats`] provides hardcoded fallback values used when the
/// asset has not yet finished loading.
///
/// [`CharacterConfig`]: crate::config::CharacterConfig
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct CharacterBaseStats {
    /// Maximum hit-points at the start of a run.
    pub max_hp: f32,
    /// Base movement speed in pixels per second.
    pub move_speed: f32,
    /// Weapon equipped when the run begins.
    pub starting_weapon: WeaponType,
    /// Flat multiplier applied to all outgoing damage (1.0 = no change).
    pub damage_multiplier: f32,
    /// Fraction subtracted from all weapon cooldowns (0.1 = −10 %).
    pub cooldown_reduction: f32,
    /// Short display name shown on the character-select screen.
    pub name: String,
    /// One-line description shown below the character name.
    pub description: String,
    /// Gold cost to unlock this character in the gold shop.
    ///
    /// `0` means always available (i.e. [`CharacterType::DefaultCharacter`]).
    pub unlock_cost: u32,
}

/// Returns hardcoded fallback statistics for a given [`CharacterType`].
///
/// These values mirror the defaults in `assets/config/character.ron` and are
/// used only while that asset is still loading.  Prefer the RON values via
/// [`CharacterParams::stats_for`] in gameplay systems.
///
/// [`CharacterParams::stats_for`]: crate::config::CharacterParams
pub fn get_character_stats(char_type: CharacterType) -> CharacterBaseStats {
    match char_type {
        CharacterType::DefaultCharacter => CharacterBaseStats {
            max_hp: 100.0,
            move_speed: 200.0,
            starting_weapon: WeaponType::Whip,
            damage_multiplier: 1.0,
            cooldown_reduction: 0.0,
            name: "Default".to_string(),
            description: "Balanced all-rounder with the Whip.".to_string(),
            unlock_cost: 0,
        },
        CharacterType::Magician => CharacterBaseStats {
            max_hp: 80.0,
            move_speed: 200.0,
            starting_weapon: WeaponType::MagicWand,
            damage_multiplier: 1.0,
            cooldown_reduction: 0.1,
            name: "Magician".to_string(),
            description: "-10 % cooldown. Starts with the Magic Wand.".to_string(),
            unlock_cost: 500,
        },
        CharacterType::Thief => CharacterBaseStats {
            max_hp: 90.0,
            move_speed: 250.0,
            starting_weapon: WeaponType::Knife,
            damage_multiplier: 1.0,
            cooldown_reduction: 0.0,
            name: "Thief".to_string(),
            description: "+25 % move speed. Starts with the Knife.".to_string(),
            unlock_cost: 500,
        },
        CharacterType::Knight => CharacterBaseStats {
            max_hp: 150.0,
            move_speed: 180.0,
            starting_weapon: WeaponType::Whip,
            damage_multiplier: 1.0,
            cooldown_reduction: 0.0,
            name: "Knight".to_string(),
            description: "+50 % max HP, -10 % move speed. Starts with the Whip.".to_string(),
            unlock_cost: 1000,
        },
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_characters_have_positive_hp_and_speed() {
        for char_type in [
            CharacterType::DefaultCharacter,
            CharacterType::Magician,
            CharacterType::Thief,
            CharacterType::Knight,
        ] {
            let stats = get_character_stats(char_type);
            assert!(
                stats.max_hp > 0.0,
                "{:?} max_hp must be positive",
                char_type
            );
            assert!(
                stats.move_speed > 0.0,
                "{:?} move_speed must be positive",
                char_type
            );
        }
    }

    #[test]
    fn default_character_starts_with_whip() {
        let stats = get_character_stats(CharacterType::DefaultCharacter);
        assert_eq!(stats.starting_weapon, WeaponType::Whip);
        assert_eq!(stats.max_hp, 100.0);
        assert_eq!(stats.move_speed, 200.0);
        assert_eq!(stats.cooldown_reduction, 0.0);
    }

    #[test]
    fn magician_has_cooldown_reduction_and_magic_wand() {
        let stats = get_character_stats(CharacterType::Magician);
        assert_eq!(stats.starting_weapon, WeaponType::MagicWand);
        assert!(
            stats.cooldown_reduction > 0.0,
            "Magician must have positive cooldown_reduction"
        );
        assert!(
            stats.max_hp < 100.0,
            "Magician HP should be lower than DefaultCharacter"
        );
    }

    #[test]
    fn thief_has_higher_speed_and_knife() {
        let default_stats = get_character_stats(CharacterType::DefaultCharacter);
        let thief_stats = get_character_stats(CharacterType::Thief);
        assert_eq!(thief_stats.starting_weapon, WeaponType::Knife);
        assert!(
            thief_stats.move_speed > default_stats.move_speed,
            "Thief move_speed must exceed DefaultCharacter"
        );
    }

    #[test]
    fn knight_has_higher_hp_and_whip() {
        let default_stats = get_character_stats(CharacterType::DefaultCharacter);
        let knight_stats = get_character_stats(CharacterType::Knight);
        assert_eq!(knight_stats.starting_weapon, WeaponType::Whip);
        assert!(
            knight_stats.max_hp > default_stats.max_hp,
            "Knight max_hp must exceed DefaultCharacter"
        );
    }

    #[test]
    fn all_characters_have_non_empty_name_and_description() {
        for char_type in [
            CharacterType::DefaultCharacter,
            CharacterType::Magician,
            CharacterType::Thief,
            CharacterType::Knight,
        ] {
            let stats = get_character_stats(char_type);
            assert!(
                !stats.name.is_empty(),
                "{:?} name must not be empty",
                char_type
            );
            assert!(
                !stats.description.is_empty(),
                "{:?} description must not be empty",
                char_type
            );
        }
    }

    #[test]
    fn default_character_unlock_cost_is_zero() {
        let stats = get_character_stats(CharacterType::DefaultCharacter);
        assert_eq!(
            stats.unlock_cost, 0,
            "DefaultCharacter must be free (always unlocked)"
        );
    }

    #[test]
    fn non_default_characters_have_pinned_unlock_costs() {
        let magician = get_character_stats(CharacterType::Magician);
        let thief = get_character_stats(CharacterType::Thief);
        let knight = get_character_stats(CharacterType::Knight);
        assert_eq!(
            magician.unlock_cost, 500,
            "Magician unlock cost must be 500G"
        );
        assert_eq!(thief.unlock_cost, 500, "Thief unlock cost must be 500G");
        assert_eq!(knight.unlock_cost, 1000, "Knight unlock cost must be 1000G");
    }

    #[test]
    fn character_base_stats_ron_deserialization() {
        let ron_str = r#"
(
    max_hp: 100.0,
    move_speed: 200.0,
    starting_weapon: Whip,
    damage_multiplier: 1.0,
    cooldown_reduction: 0.0,
    name: "Default",
    description: "Balanced all-rounder with the Whip.",
    unlock_cost: 0,
)
"#;
        let stats: CharacterBaseStats = ron::de::from_str(ron_str).unwrap();
        assert_eq!(stats.max_hp, 100.0);
        assert_eq!(stats.starting_weapon, WeaponType::Whip);
        assert_eq!(stats.name, "Default");
        assert_eq!(stats.unlock_cost, 0);
    }
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
