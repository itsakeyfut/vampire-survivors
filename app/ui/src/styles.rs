//! UI style constants: colors, font sizes, and layout values.
//!
//! All UI systems should source their visual properties from this module to
//! keep a consistent look across every screen.

use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Color palette
// ---------------------------------------------------------------------------

/// Background color — near-black used as the base for all screens.
pub const BG_COLOR: Color = Color::srgb(0.05, 0.05, 0.08);

/// Title color — blood red used for the game title and emphasis.
pub const TITLE_COLOR: Color = Color::srgb(0.85, 0.15, 0.15);

/// Text color — off-white used for body text and button labels.
pub const TEXT_COLOR: Color = Color::srgb(0.95, 0.90, 0.85);

// ---------------------------------------------------------------------------
// Button colors
// ---------------------------------------------------------------------------

/// Button color in its default (resting) state.
pub const BUTTON_NORMAL: Color = Color::srgb(0.30, 0.05, 0.05);

/// Button color when the cursor hovers over it.
pub const BUTTON_HOVER: Color = Color::srgb(0.60, 0.10, 0.10);

/// Button color while the mouse button is held down.
pub const BUTTON_PRESSED: Color = Color::srgb(0.20, 0.02, 0.02);

// ---------------------------------------------------------------------------
// Font sizes
// ---------------------------------------------------------------------------

/// Huge font size (72 px) — used for screen titles such as the game title.
pub const FONT_SIZE_HUGE: f32 = 72.0;

/// Large font size (48 px) — used for headings and primary button labels.
pub const FONT_SIZE_LARGE: f32 = 48.0;

/// Medium font size (32 px) — used for secondary UI labels.
pub const FONT_SIZE_MEDIUM: f32 = 32.0;

/// Small font size (24 px) — used for supplementary information.
pub const FONT_SIZE_SMALL: f32 = 24.0;

// ---------------------------------------------------------------------------
// Button sizes
// ---------------------------------------------------------------------------

/// Width of a large button (px) — used for primary actions like Start/Retry.
pub const BUTTON_LARGE_WIDTH: f32 = 280.0;

/// Height of a large button (px) — used for primary actions like Start/Retry.
pub const BUTTON_LARGE_HEIGHT: f32 = 80.0;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_sizes_are_ordered() {
        assert!(FONT_SIZE_HUGE > FONT_SIZE_LARGE);
        assert!(FONT_SIZE_LARGE > FONT_SIZE_MEDIUM);
        assert!(FONT_SIZE_MEDIUM > FONT_SIZE_SMALL);
        assert!(FONT_SIZE_SMALL > 0.0);
    }

    #[test]
    fn button_colors_are_distinct() {
        let normal = BUTTON_NORMAL.to_srgba();
        let hover = BUTTON_HOVER.to_srgba();
        let pressed = BUTTON_PRESSED.to_srgba();
        assert_ne!(normal.red, hover.red, "NORMAL and HOVER should differ");
        assert_ne!(normal.red, pressed.red, "NORMAL and PRESSED should differ");
    }

    #[test]
    fn colors_in_valid_range() {
        for color in [
            BG_COLOR,
            TITLE_COLOR,
            TEXT_COLOR,
            BUTTON_NORMAL,
            BUTTON_HOVER,
            BUTTON_PRESSED,
        ] {
            let c = color.to_srgba();
            assert!((0.0..=1.0).contains(&c.red));
            assert!((0.0..=1.0).contains(&c.green));
            assert!((0.0..=1.0).contains(&c.blue));
        }
    }
}
