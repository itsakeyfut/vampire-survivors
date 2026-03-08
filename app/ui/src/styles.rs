//! UI style fallback constants used across multiple screens.
//!
//! All constants are prefixed `DEFAULT_` and serve as compile-time fallbacks
//! when [`crate::config::UiStyleConfig`] has not yet been loaded from RON.
//! UI systems should prefer reading via [`crate::config::UiStyleParams`] and
//! fall back to these only when the config returns `None`.
//!
//! Widget-specific fallbacks (button colors, heading font size, card layout)
//! are defined privately inside each consumer module, following the
//! project-wide convention.

use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Screen-level palette
// ---------------------------------------------------------------------------

/// Background color — near-black used as the base for all screens.
pub const DEFAULT_BG_COLOR: Color = Color::srgb(0.05, 0.05, 0.08);

/// Title color — blood red used for the game title heading.
pub const DEFAULT_TITLE_COLOR: Color = Color::srgb(0.85, 0.15, 0.15);

// ---------------------------------------------------------------------------
// Font paths
// ---------------------------------------------------------------------------

/// Path to the Japanese text font (DotGothic16).
///
/// Used for Japanese UI text.  Currently shares the same file as
/// [`FONT_TEXT_EN`]; swap this path when a dedicated JP font is added.
/// Relative to the `assets/` directory; pass to [`AssetServer::load`].
pub const FONT_TEXT_JP: &str = "fonts/DotGothic16/DotGothic16-Regular.ttf";

/// Path to the Latin-script text font (DotGothic16).
///
/// Used for English, German, and other Latin-alphabet languages.
/// Currently shares the same file as [`FONT_TEXT_JP`]; swap this path when
/// a dedicated Latin font is added (e.g. for Cyrillic, add `FONT_TEXT_RU`).
/// Relative to the `assets/` directory; pass to [`AssetServer::load`].
pub const FONT_TEXT_EN: &str = "fonts/DotGothic16/DotGothic16-Regular.ttf";

/// Path to the symbol / UI-icon font (Noto Sans JP).
///
/// Used for characters outside DotGothic16's coverage, such as punctuation
/// symbols not present in the pixel font.
/// Relative to the `assets/` directory; pass to [`AssetServer::load`].
pub const FONT_SYMBOL: &str = "fonts/NotoSansJP/NotoSansJP-Regular.ttf";

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colors_in_valid_range() {
        for color in [DEFAULT_BG_COLOR, DEFAULT_TITLE_COLOR] {
            let c = color.to_srgba();
            assert!((0.0..=1.0).contains(&c.red));
            assert!((0.0..=1.0).contains(&c.green));
            assert!((0.0..=1.0).contains(&c.blue));
        }
    }
}
