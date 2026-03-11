//! User-configurable game settings.
//!
//! [`GameSettings`] is a Bevy resource holding all player preferences that
//! persist across sessions.  Call [`GameSettings::load`] at startup and
//! [`GameSettings::save`] to persist to `save/settings.json`.
//! The autosave Bevy system lives in [`crate::systems::persistence`].

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Path to the settings JSON save file.
const DEFAULT_SAVE_PATH: &str = "save/settings.json";
/// Directory that contains the settings file.
const DEFAULT_SAVE_DIR: &str = "save";

// ---------------------------------------------------------------------------
// Language
// ---------------------------------------------------------------------------

/// UI language choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Language {
    /// Japanese (日本語) — default.
    #[default]
    Japanese,
    /// English.
    English,
}

impl Language {
    /// Cycles to the next language (wraps around).
    pub fn next(self) -> Self {
        match self {
            Language::Japanese => Language::English,
            Language::English => Language::Japanese,
        }
    }
}

// ---------------------------------------------------------------------------
// GameSettings
// ---------------------------------------------------------------------------

/// User-configurable settings stored as a Bevy resource.
///
/// Loaded from `save/settings.json` at startup via [`GameSettings::load`] and
/// saved automatically when the player leaves the settings screen via
/// [`save_settings_on_exit`].
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct GameSettings {
    /// UI and text language.
    pub language: Language,
}

impl GameSettings {
    /// Load settings from `save/settings.json`.
    ///
    /// Returns `Self::default()` when:
    /// - the file does not exist (first launch)
    /// - the file cannot be read
    /// - the JSON is malformed or has unknown fields
    pub fn load() -> Self {
        Self::load_from(Path::new(DEFAULT_SAVE_PATH))
    }

    /// Load settings from an arbitrary path (used in tests).
    pub fn load_from(path: &Path) -> Self {
        match fs::read_to_string(path) {
            Ok(json) => match serde_json::from_str(&json) {
                Ok(settings) => settings,
                Err(e) => {
                    warn!("Failed to parse settings from {}: {e}", path.display());
                    Self::default()
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Self::default(),
            Err(e) => {
                warn!("Failed to read settings from {}: {e}", path.display());
                Self::default()
            }
        }
    }

    /// Save settings to `save/settings.json`.
    ///
    /// Creates the `save/` directory if needed.  Logs a warning on failure
    /// (non-fatal — the game continues running).
    pub fn save(&self) {
        self.save_to(Path::new(DEFAULT_SAVE_DIR), "settings.json");
    }

    /// Save settings to `{dir}/{filename}` (used in tests).
    pub fn save_to(&self, dir: &Path, filename: &str) {
        if let Err(e) = self.try_save_to(dir, filename) {
            warn!("Failed to save settings: {e}");
        }
    }

    fn try_save_to(&self, dir: &Path, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(dir)?;
        let json = serde_json::to_string_pretty(self)?;
        let tmp_path = dir.join(format!("{filename}.tmp"));
        fs::write(&tmp_path, &json)?;
        fs::rename(&tmp_path, dir.join(filename))?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn language_default_is_japanese() {
        assert_eq!(Language::default(), Language::Japanese);
    }

    #[test]
    fn language_next_cycles() {
        assert_eq!(Language::Japanese.next(), Language::English);
        assert_eq!(Language::English.next(), Language::Japanese);
    }

    #[test]
    fn game_settings_default() {
        let s = GameSettings::default();
        assert_eq!(s.language, Language::Japanese);
    }

    #[test]
    fn game_settings_serde_roundtrip() {
        let original = GameSettings {
            language: Language::English,
        };
        let json = serde_json::to_string(&original).unwrap();
        let back: GameSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(back.language, Language::English);
    }

    // -----------------------------------------------------------------------
    // Save / load
    // -----------------------------------------------------------------------

    #[test]
    fn load_from_nonexistent_file_returns_default() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("missing.json");
        let s = GameSettings::load_from(&path);
        assert_eq!(s.language, Language::Japanese);
    }

    #[test]
    fn load_from_corrupt_file_returns_default() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("settings.json");
        fs::write(&path, "{ not valid json }").unwrap();
        let s = GameSettings::load_from(&path);
        assert_eq!(s.language, Language::Japanese);
    }

    #[test]
    fn save_to_and_load_from_round_trip() {
        let dir = TempDir::new().unwrap();
        let original = GameSettings {
            language: Language::English,
        };
        original.save_to(dir.path(), "settings.json");
        let path = dir.path().join("settings.json");
        let restored = GameSettings::load_from(&path);
        assert_eq!(restored.language, Language::English);
    }

    #[test]
    fn save_to_creates_directory_if_missing() {
        let dir = TempDir::new().unwrap();
        let nested = dir.path().join("nested").join("save");
        let s = GameSettings::default();
        s.save_to(&nested, "settings.json");
        assert!(nested.join("settings.json").exists());
    }

    #[test]
    fn saved_file_is_valid_json_with_language_field() {
        let dir = TempDir::new().unwrap();
        let s = GameSettings {
            language: Language::English,
        };
        s.save_to(dir.path(), "settings.json");
        let content = fs::read_to_string(dir.path().join("settings.json")).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["language"], "English");
    }

    #[test]
    fn save_overwrites_existing_file() {
        let dir = TempDir::new().unwrap();
        GameSettings {
            language: Language::Japanese,
        }
        .save_to(dir.path(), "settings.json");
        GameSettings {
            language: Language::English,
        }
        .save_to(dir.path(), "settings.json");
        let path = dir.path().join("settings.json");
        let restored = GameSettings::load_from(&path);
        assert_eq!(restored.language, Language::English);
    }
}
