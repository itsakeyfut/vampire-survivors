//! UI style fallback constants: colors, font sizes, and layout values.
//!
//! All constants are prefixed `DEFAULT_` and serve as compile-time fallbacks
//! when [`crate::config::UiStyleConfig`] has not yet been loaded from RON.
//! UI systems should prefer reading via [`crate::config::UiStyleParams`] and
//! fall back to these only when the config returns `None`.

use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Color palette
// ---------------------------------------------------------------------------

/// Background color — near-black used as the base for all screens.
pub const DEFAULT_BG_COLOR: Color = Color::srgb(0.05, 0.05, 0.08);

/// Title color — blood red used for the game title and emphasis.
pub const DEFAULT_TITLE_COLOR: Color = Color::srgb(0.85, 0.15, 0.15);

/// Text color — off-white used for body text and button labels.
pub const DEFAULT_TEXT_COLOR: Color = Color::srgb(0.95, 0.90, 0.85);

// ---------------------------------------------------------------------------
// Button colors
// ---------------------------------------------------------------------------

/// Button color in its default (resting) state.
pub const DEFAULT_BUTTON_NORMAL: Color = Color::srgb(0.30, 0.05, 0.05);

/// Button color when the cursor hovers over it.
pub const DEFAULT_BUTTON_HOVER: Color = Color::srgb(0.60, 0.10, 0.10);

/// Button color while the mouse button is held down.
pub const DEFAULT_BUTTON_PRESSED: Color = Color::srgb(0.20, 0.02, 0.02);

// ---------------------------------------------------------------------------
// Font sizes
// ---------------------------------------------------------------------------

/// Huge font size (72 px) — used for screen titles such as the game title.
pub const DEFAULT_FONT_SIZE_HUGE: f32 = 72.0;

/// Large font size (48 px) — used for headings and primary button labels.
pub const DEFAULT_FONT_SIZE_LARGE: f32 = 48.0;

/// Medium font size (32 px) — used for secondary UI labels.
pub const DEFAULT_FONT_SIZE_MEDIUM: f32 = 32.0;

/// Small font size (24 px) — used for supplementary information.
pub const DEFAULT_FONT_SIZE_SMALL: f32 = 24.0;

// ---------------------------------------------------------------------------
// Button sizes
// ---------------------------------------------------------------------------

/// Width of a large button (px) — used for primary actions like Start/Retry.
pub const DEFAULT_BUTTON_LARGE_WIDTH: f32 = 280.0;

/// Height of a large button (px) — used for primary actions like Start/Retry.
pub const DEFAULT_BUTTON_LARGE_HEIGHT: f32 = 80.0;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_sizes_are_ordered() {
        assert!(DEFAULT_FONT_SIZE_HUGE > DEFAULT_FONT_SIZE_LARGE);
        assert!(DEFAULT_FONT_SIZE_LARGE > DEFAULT_FONT_SIZE_MEDIUM);
        assert!(DEFAULT_FONT_SIZE_MEDIUM > DEFAULT_FONT_SIZE_SMALL);
        assert!(DEFAULT_FONT_SIZE_SMALL > 0.0);
    }

    #[test]
    fn button_colors_are_distinct() {
        let normal = DEFAULT_BUTTON_NORMAL.to_srgba();
        let hover = DEFAULT_BUTTON_HOVER.to_srgba();
        let pressed = DEFAULT_BUTTON_PRESSED.to_srgba();
        assert_ne!(normal.red, hover.red, "NORMAL and HOVER should differ");
        assert_ne!(normal.red, pressed.red, "NORMAL and PRESSED should differ");
    }

    #[test]
    fn colors_in_valid_range() {
        for color in [
            DEFAULT_BG_COLOR,
            DEFAULT_TITLE_COLOR,
            DEFAULT_TEXT_COLOR,
            DEFAULT_BUTTON_NORMAL,
            DEFAULT_BUTTON_HOVER,
            DEFAULT_BUTTON_PRESSED,
        ] {
            let c = color.to_srgba();
            assert!((0.0..=1.0).contains(&c.red));
            assert!((0.0..=1.0).contains(&c.green));
            assert!((0.0..=1.0).contains(&c.blue));
        }
    }
}
