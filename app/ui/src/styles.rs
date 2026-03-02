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
