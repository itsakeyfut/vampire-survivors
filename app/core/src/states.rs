use bevy::prelude::*;

/// All possible game states.
///
/// Transitions:
/// ```text
/// Loading ─────────────────────────→ Title
///   (waits for all RON configs)
///
/// Title ──────────────────────────→ MetaShop
///   │                                  │
///   ↓                                  ↓
/// CharacterSelect ──────────────→ Title
///   │
///   ↓
/// Playing ←──── LevelUp (returns after choice)
///   │  ↑
///   │  │ ESC
///   │  ↓
///   │  Paused → Playing (resume) / Title (quit)
///   │
///   ├──→ GameOver  (HP = 0)
///   └──→ Victory   (boss defeated)
/// ```
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    /// Initial loading state — waits for all RON config assets to be ready,
    /// then automatically transitions to [`AppState::Title`].
    ///
    /// To test `Playing`-state systems during development, temporarily change
    /// `#[default]` to `Playing` in this enum.
    #[default]
    Loading,
    /// Title screen.
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
