//! User-configurable game settings.
//!
//! [`GameSettings`] is a Bevy resource holding all player preferences that
//! persist across screens within a session.  Persisting to disk can be added
//! later by serialising this resource to `save/settings.json`.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

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
/// Inserted by [`crate::GameCorePlugin`] with sensible defaults so the game
/// works without any saved preferences.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameSettings {
    /// UI and text language.
    pub language: Language,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

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
}
