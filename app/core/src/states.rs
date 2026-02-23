use bevy::prelude::*;

/// All possible game states.
///
/// Transitions:
/// ```text
/// Title ──────────────────────────→ MetaShop
///   │                                  │
///   ↓                                  ↓
/// CharacterSelect ──────────────→ Title
///   │
///   ↓
/// Playing ←──── LevelUp (returns after choice)
///   │  ↑
///   │  │ ESC
///   ↓  │
/// Paused → Playing (resume) / Title (quit)
///   │
///   ├──→ GameOver  (HP = 0)
///   └──→ Victory   (boss defeated)
/// ```
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    /// Title screen (default entry point).
    #[default]
    Title,
    /// Character selection screen.
    CharacterSelect,
    /// Main gameplay.
    Playing,
    /// Level-up card selection overlay (gameplay paused underneath).
    LevelUp,
    /// Paused via ESC during gameplay.
    Paused,
    /// Game over screen (player died).
    GameOver,
    /// Victory screen (boss defeated after 30 min).
    Victory,
    /// Meta-progression gold shop (accessible from Title).
    MetaShop,
}
