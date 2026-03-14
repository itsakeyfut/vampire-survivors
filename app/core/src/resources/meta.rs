use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::types::{CharacterType, MetaUpgradeType, StageType};

/// Path to the save directory, relative to the working directory.
const DEFAULT_SAVE_DIR: &str = "save";
/// Path to the meta-progression JSON save file.
const DEFAULT_SAVE_PATH: &str = "save/meta.json";

/// Which character the player selected on the character-select screen.
#[derive(Resource, Debug)]
pub struct SelectedCharacter(pub CharacterType);

impl Default for SelectedCharacter {
    fn default() -> Self {
        Self(CharacterType::DefaultCharacter)
    }
}

/// Which stage the player selected on the stage-select screen.
#[derive(Resource, Debug)]
pub struct SelectedStage(pub StageType);

impl Default for SelectedStage {
    fn default() -> Self {
        Self(StageType::MadForest)
    }
}

/// Persistent cross-run data. Loaded from `save/meta.json` at startup and
/// saved automatically after game-over, victory, and shop purchases.
///
/// `#[serde(default)]` ensures that old save files with missing fields
/// deserialize successfully using each field's `Default` value, making
/// schema evolution safely additive.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MetaProgress {
    /// Total gold accumulated across all runs.
    pub total_gold: u32,
    /// Characters that have been unlocked via the gold shop.
    pub unlocked_characters: Vec<CharacterType>,
    /// Permanent upgrades that have been purchased.
    pub purchased_upgrades: Vec<MetaUpgradeType>,
    /// Set to `true` when the file existed but could not be read or parsed.
    /// Autosave is suppressed while this flag is set so the broken file on
    /// disk is preserved for manual recovery.
    #[serde(skip)]
    pub(crate) load_failed: bool,
}

impl Default for MetaProgress {
    fn default() -> Self {
        Self {
            total_gold: 0,
            unlocked_characters: vec![CharacterType::DefaultCharacter],
            purchased_upgrades: vec![],
            load_failed: false,
        }
    }
}

impl MetaProgress {
    /// Load meta-progression from `save/meta.json`.
    ///
    /// Returns `Self::default()` when:
    /// - the file does not exist (first launch)
    /// - the file cannot be read (permission error, etc.)
    /// - the JSON is malformed or missing fields
    pub fn load() -> Self {
        Self::load_from(Path::new(DEFAULT_SAVE_PATH))
    }

    /// Load meta-progression from an arbitrary path.
    ///
    /// Separated from [`Self::load`] so tests can use a temporary directory
    /// without touching the real save file.
    pub fn load_from(path: &Path) -> Self {
        match fs::read_to_string(path) {
            Ok(json) => match serde_json::from_str(&json) {
                Ok(meta) => meta,
                Err(e) => {
                    warn!(
                        "Failed to parse meta progress from {}: {e} \
                         — autosave suppressed to preserve original file",
                        path.display()
                    );
                    Self {
                        load_failed: true,
                        ..Self::default()
                    }
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Self::default(),
            Err(e) => {
                warn!(
                    "Failed to read meta progress from {}: {e} \
                     — autosave suppressed to preserve original file",
                    path.display()
                );
                Self {
                    load_failed: true,
                    ..Self::default()
                }
            }
        }
    }

    /// Save meta-progression to `save/meta.json`.
    ///
    /// Creates the `save/` directory if it does not yet exist.
    /// Logs a warning if the save fails (non-fatal — the game keeps running).
    pub fn save(&self) {
        self.save_to(Path::new(DEFAULT_SAVE_DIR), "meta.json");
    }

    /// Save meta-progression to `{dir}/{filename}`.
    ///
    /// Separated from [`Self::save`] so tests can use a temporary directory.
    pub fn save_to(&self, dir: &Path, filename: &str) {
        if let Err(e) = self.try_save_to(dir, filename) {
            warn!("Failed to save meta progress: {e}");
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

    // -----------------------------------------------------------------------
    // Default / serde
    // -----------------------------------------------------------------------

    #[test]
    fn selected_character_default_is_default_character() {
        let sc = SelectedCharacter::default();
        assert_eq!(sc.0, CharacterType::DefaultCharacter);
    }

    #[test]
    fn meta_progress_default_unlocks_default_character() {
        let mp = MetaProgress::default();
        assert!(
            mp.unlocked_characters
                .contains(&CharacterType::DefaultCharacter)
        );
        assert_eq!(mp.total_gold, 0);
    }

    #[test]
    fn meta_progress_serializes_and_deserializes() {
        let original = MetaProgress {
            total_gold: 1234,
            unlocked_characters: vec![CharacterType::DefaultCharacter],
            purchased_upgrades: vec![MetaUpgradeType::BonusHp],
            load_failed: false,
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: MetaProgress = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.total_gold, 1234);
        assert_eq!(restored.unlocked_characters.len(), 1);
        assert_eq!(restored.purchased_upgrades.len(), 1);
    }

    // -----------------------------------------------------------------------
    // Save / load
    // -----------------------------------------------------------------------

    #[test]
    fn load_from_nonexistent_file_returns_default() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("missing.json");
        let mp = MetaProgress::load_from(&path);
        assert_eq!(mp.total_gold, 0);
        assert!(
            mp.unlocked_characters
                .contains(&CharacterType::DefaultCharacter)
        );
    }

    #[test]
    fn load_from_corrupt_file_returns_default_with_load_failed() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("meta.json");
        fs::write(&path, "{ not valid json }").unwrap();
        let mp = MetaProgress::load_from(&path);
        assert_eq!(mp.total_gold, 0);
        assert!(
            mp.load_failed,
            "load_failed should be true for corrupt files"
        );
    }

    #[test]
    fn load_from_nonexistent_file_has_no_load_failed() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("missing.json");
        let mp = MetaProgress::load_from(&path);
        assert!(!mp.load_failed, "first launch should not set load_failed");
    }

    #[test]
    fn save_to_and_load_from_round_trip() {
        let dir = TempDir::new().unwrap();
        let original = MetaProgress {
            total_gold: 999,
            unlocked_characters: vec![CharacterType::DefaultCharacter],
            purchased_upgrades: vec![MetaUpgradeType::BonusHp],
            load_failed: false,
        };
        original.save_to(dir.path(), "meta.json");

        let path = dir.path().join("meta.json");
        let restored = MetaProgress::load_from(&path);

        assert_eq!(restored.total_gold, 999);
        assert_eq!(restored.purchased_upgrades.len(), 1);
    }

    #[test]
    fn save_to_creates_directory_if_missing() {
        let dir = TempDir::new().unwrap();
        let nested = dir.path().join("nested").join("save");
        let mp = MetaProgress::default();
        mp.save_to(&nested, "meta.json");
        assert!(nested.join("meta.json").exists(), "file should be created");
    }

    #[test]
    fn saved_file_is_valid_json() {
        let dir = TempDir::new().unwrap();
        let mp = MetaProgress {
            total_gold: 42,
            ..MetaProgress::default()
        };
        mp.save_to(dir.path(), "meta.json");
        let content = fs::read_to_string(dir.path().join("meta.json")).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["total_gold"], 42);
    }

    #[test]
    fn save_overwrites_existing_file() {
        let dir = TempDir::new().unwrap();

        let first = MetaProgress {
            total_gold: 100,
            ..MetaProgress::default()
        };
        first.save_to(dir.path(), "meta.json");

        let second = MetaProgress {
            total_gold: 200,
            ..MetaProgress::default()
        };
        second.save_to(dir.path(), "meta.json");

        let path = dir.path().join("meta.json");
        let restored = MetaProgress::load_from(&path);
        assert_eq!(restored.total_gold, 200);
    }
}
