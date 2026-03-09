//! Character configuration loaded from `assets/config/character.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

use crate::types::{CharacterBaseStats, CharacterType};

// ---------------------------------------------------------------------------
// Asset type
// ---------------------------------------------------------------------------

/// Full character configuration, loaded from `assets/config/character.ron`.
///
/// Contains one [`CharacterBaseStats`] block per playable character.
/// Call [`CharacterConfig::stats_for`] to look up a character by type.
/// Hot-reloading this file takes effect the next time a run starts.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct CharacterConfig {
    pub default_character: CharacterBaseStats,
    pub magician: CharacterBaseStats,
    pub thief: CharacterBaseStats,
    pub knight: CharacterBaseStats,
}

impl CharacterConfig {
    /// Returns the stat block for a given [`CharacterType`].
    pub fn stats_for(&self, char_type: CharacterType) -> &CharacterBaseStats {
        match char_type {
            CharacterType::DefaultCharacter => &self.default_character,
            CharacterType::Magician => &self.magician,
            CharacterType::Thief => &self.thief,
            CharacterType::Knight => &self.knight,
        }
    }
}

/// Resource holding the handle to the loaded character configuration.
#[derive(Resource)]
pub struct CharacterConfigHandle(pub Handle<CharacterConfig>);

// ---------------------------------------------------------------------------
// SystemParam bundle
// ---------------------------------------------------------------------------

/// SystemParam bundle for accessing [`CharacterConfig`].
///
/// Returns `None` while the asset is still loading or when
/// [`GameConfigPlugin`] has not been registered (e.g. in unit tests).
/// Call `.get()` to obtain `Option<&CharacterConfig>`.
///
/// [`GameConfigPlugin`]: crate::config::GameConfigPlugin
#[derive(SystemParam)]
pub struct CharacterParams<'w> {
    handle: Option<Res<'w, CharacterConfigHandle>>,
    assets: Option<Res<'w, Assets<CharacterConfig>>>,
}

impl<'w> CharacterParams<'w> {
    /// Returns the currently loaded [`CharacterConfig`], or `None` while loading.
    pub fn get(&self) -> Option<&CharacterConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }

    /// Returns the stat block for the given character, or the hardcoded fallback
    /// when the config asset is not yet available.
    pub fn stats_for(&self, char_type: CharacterType) -> CharacterBaseStats {
        self.get()
            .map(|c| c.stats_for(char_type).clone())
            .unwrap_or_else(|| crate::types::get_character_stats(char_type))
    }
}

// ---------------------------------------------------------------------------
// Hot-reload system
// ---------------------------------------------------------------------------

/// Handles hot-reloading of character configuration.
///
/// Changes take effect the next time a run starts; in-progress runs are
/// not affected (player stats are baked into the player entity at spawn time).
pub fn hot_reload_character_config(mut events: MessageReader<AssetEvent<CharacterConfig>>) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id: _ } => {
                info!("✅ Character config loaded");
            }
            AssetEvent::Modified { id: _ } => {
                info!("🔥 Hot-reloading character config! New runs will use updated stats.");
            }
            AssetEvent::Removed { id: _ } => {
                warn!("⚠️ Character config removed");
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::WeaponType;

    fn sample_ron() -> &'static str {
        r#"
CharacterConfig(
    default_character: (
        max_hp: 100.0,
        move_speed: 200.0,
        starting_weapon: Whip,
        damage_multiplier: 1.0,
        cooldown_reduction: 0.0,
        name: "Default",
        description: "Balanced all-rounder with the Whip.",
        unlock_cost: 0,
    ),
    magician: (
        max_hp: 80.0,
        move_speed: 200.0,
        starting_weapon: MagicWand,
        damage_multiplier: 1.0,
        cooldown_reduction: 0.1,
        name: "Magician",
        description: "-10 % cooldown. Starts with the Magic Wand.",
        unlock_cost: 500,
    ),
    thief: (
        max_hp: 90.0,
        move_speed: 250.0,
        starting_weapon: Knife,
        damage_multiplier: 1.0,
        cooldown_reduction: 0.0,
        name: "Thief",
        description: "+25 % move speed. Starts with the Knife.",
        unlock_cost: 500,
    ),
    knight: (
        max_hp: 150.0,
        move_speed: 180.0,
        starting_weapon: Whip,
        damage_multiplier: 1.0,
        cooldown_reduction: 0.0,
        name: "Knight",
        description: "+50 % max HP, -10 % move speed. Starts with the Whip.",
        unlock_cost: 1000,
    ),
)
"#
    }

    #[test]
    fn character_config_deserializes() {
        let config: CharacterConfig = ron::de::from_str(sample_ron()).unwrap();
        assert_eq!(config.default_character.max_hp, 100.0);
        assert_eq!(config.default_character.starting_weapon, WeaponType::Whip);
        assert_eq!(config.default_character.unlock_cost, 0);
        assert_eq!(config.magician.starting_weapon, WeaponType::MagicWand);
        assert_eq!(config.magician.cooldown_reduction, 0.1);
        assert_eq!(config.magician.unlock_cost, 500);
        assert_eq!(config.thief.starting_weapon, WeaponType::Knife);
        assert_eq!(config.thief.move_speed, 250.0);
        assert_eq!(config.thief.unlock_cost, 500);
        assert_eq!(config.knight.max_hp, 150.0);
        assert_eq!(config.knight.starting_weapon, WeaponType::Whip);
        assert_eq!(config.knight.unlock_cost, 1000);
    }

    #[test]
    fn stats_for_returns_correct_entry() {
        let config: CharacterConfig = ron::de::from_str(sample_ron()).unwrap();
        assert_eq!(
            config.stats_for(CharacterType::DefaultCharacter).max_hp,
            100.0
        );
        assert_eq!(
            config.stats_for(CharacterType::Magician).cooldown_reduction,
            0.1
        );
        assert_eq!(config.stats_for(CharacterType::Thief).move_speed, 250.0);
        assert_eq!(config.stats_for(CharacterType::Knight).max_hp, 150.0);
    }

    #[test]
    fn all_entries_have_positive_hp_and_speed() {
        let config: CharacterConfig = ron::de::from_str(sample_ron()).unwrap();
        for char_type in [
            CharacterType::DefaultCharacter,
            CharacterType::Magician,
            CharacterType::Thief,
            CharacterType::Knight,
        ] {
            let stats = config.stats_for(char_type);
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
    fn all_entries_have_non_empty_name_and_description() {
        let config: CharacterConfig = ron::de::from_str(sample_ron()).unwrap();
        for char_type in [
            CharacterType::DefaultCharacter,
            CharacterType::Magician,
            CharacterType::Thief,
            CharacterType::Knight,
        ] {
            let stats = config.stats_for(char_type);
            assert!(!stats.name.is_empty(), "{:?} name is empty", char_type);
            assert!(
                !stats.description.is_empty(),
                "{:?} description is empty",
                char_type
            );
        }
    }
}
