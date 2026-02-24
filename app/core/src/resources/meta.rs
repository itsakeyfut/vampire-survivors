use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::types::{CharacterType, MetaUpgradeType};

/// Which character the player selected on the character-select screen.
#[derive(Resource, Debug)]
pub struct SelectedCharacter(pub CharacterType);

impl Default for SelectedCharacter {
    fn default() -> Self {
        Self(CharacterType::DefaultCharacter)
    }
}

/// Persistent cross-run data. Loaded from `save/meta.json` at startup.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct MetaProgress {
    /// Total gold accumulated across all runs.
    pub total_gold: u32,
    /// Characters that have been unlocked via the gold shop.
    pub unlocked_characters: Vec<CharacterType>,
    /// Permanent upgrades that have been purchased.
    pub purchased_upgrades: Vec<MetaUpgradeType>,
}

impl Default for MetaProgress {
    fn default() -> Self {
        Self {
            total_gold: 0,
            unlocked_characters: vec![CharacterType::DefaultCharacter],
            purchased_upgrades: vec![],
        }
    }
}

impl MetaProgress {
    /// Load meta-progression from `save/meta.json`.
    /// Falls back to default values if the file does not exist or is corrupt.
    pub fn load() -> Self {
        // TODO: implement file I/O in Phase 14
        Self::default()
    }

    /// Save meta-progression to `save/meta.json`.
    pub fn save(&self) {
        // TODO: implement file I/O in Phase 14
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

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
}
