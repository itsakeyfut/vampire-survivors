//! Upgrade card HUD configuration.
//!
//! Loaded from `assets/config/ui/hud/upgrade_card.ron`.
//! Controls all visual properties of the upgrade selection cards shown on
//! the level-up screen: dimensions, spacing, interaction colors, and typography.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

use crate::config::SrgbColor;

// ---------------------------------------------------------------------------
// Config asset
// ---------------------------------------------------------------------------

/// Upgrade card HUD config loaded from `config/ui/hud/upgrade_card.ron`.
///
/// Covers layout (dimensions, gaps, padding) and visuals (colors, font sizes)
/// for each interactive upgrade card and the row that contains them.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct UpgradeCardHudConfig {
    /// Card width in pixels.
    pub card_width: f32,
    /// Card height in pixels.
    pub card_height: f32,
    /// Horizontal gap between adjacent cards in pixels.
    pub card_gap: f32,
    /// Internal padding inside each card in pixels (all sides).
    pub padding: f32,
    /// Vertical gap between elements inside a card in pixels.
    pub inner_gap: f32,
    /// Card background color in the resting state.
    pub card_normal: SrgbColor,
    /// Card background color when the cursor is hovering.
    pub card_hover: SrgbColor,
    /// Card background color while the mouse button is held.
    pub card_pressed: SrgbColor,
    /// Color of the upgrade-type subtitle (e.g. "New Weapon").
    pub subtitle_color: SrgbColor,
    /// Primary text color for the item name and description.
    pub text_color: SrgbColor,
    /// Font size for the item name text in points.
    pub font_size_name: f32,
    /// Font size for the subtitle text in points.
    pub font_size_subtitle: f32,
    /// Font size for the description text in points.
    pub font_size_desc: f32,
    /// Side length of the square icon placeholder in pixels.
    pub icon_size: f32,
}

/// Resource holding the handle to the loaded [`UpgradeCardHudConfig`].
#[derive(Resource)]
pub struct UpgradeCardHudConfigHandle(pub Handle<UpgradeCardHudConfig>);

// ---------------------------------------------------------------------------
// SystemParam bundle
// ---------------------------------------------------------------------------

/// SystemParam bundle for accessing [`UpgradeCardHudConfig`].
///
/// Returns `None` while the asset is still loading or when
/// [`crate::config::UiConfigPlugin`] is not registered (e.g. in unit tests).
/// Call `.get()` to obtain `Option<&UpgradeCardHudConfig>`.
#[derive(SystemParam)]
pub struct UpgradeCardHudParams<'w> {
    handle: Option<Res<'w, UpgradeCardHudConfigHandle>>,
    assets: Option<Res<'w, Assets<UpgradeCardHudConfig>>>,
}

impl<'w> UpgradeCardHudParams<'w> {
    /// Returns the currently loaded config, or `None`.
    pub fn get(&self) -> Option<&UpgradeCardHudConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upgrade_card_hud_config_deserialization() {
        let ron_data = r#"
UpgradeCardHudConfig(
    card_width:  260.0,
    card_height: 320.0,
    card_gap:     30.0,
    padding:      16.0,
    inner_gap:    12.0,
    card_normal:    (r: 0.12, g: 0.08, b: 0.28),
    card_hover:     (r: 0.22, g: 0.14, b: 0.48),
    card_pressed:   (r: 0.08, g: 0.05, b: 0.18),
    subtitle_color: (r: 0.85, g: 0.70, b: 0.30),
    text_color:     (r: 0.95, g: 0.90, b: 0.85),
    font_size_name:     32.0,
    font_size_subtitle: 24.0,
    font_size_desc:     24.0,
    icon_size:          64.0,
)
"#;
        let cfg: UpgradeCardHudConfig =
            ron::de::from_str(ron_data).expect("RON parse must succeed");
        assert_eq!(cfg.card_width, 260.0);
        assert_eq!(cfg.card_height, 320.0);
        assert_eq!(cfg.card_gap, 30.0);
        assert_eq!(cfg.padding, 16.0);
        assert_eq!(cfg.inner_gap, 12.0);
        assert!((cfg.card_normal.r - 0.12).abs() < 1e-6);
        assert!((cfg.card_hover.r - 0.22).abs() < 1e-6);
        assert!((cfg.card_pressed.r - 0.08).abs() < 1e-6);
        assert!((cfg.subtitle_color.r - 0.85).abs() < 1e-6);
        assert!((cfg.text_color.r - 0.95).abs() < 1e-6);
        assert_eq!(cfg.font_size_name, 32.0);
        assert_eq!(cfg.font_size_subtitle, 24.0);
        assert_eq!(cfg.font_size_desc, 24.0);
    }

    #[test]
    fn card_dimensions_are_positive() {
        // card_width must be > 32 so description max_width (card_width - 32) stays positive.
        let cfg = UpgradeCardHudConfig {
            card_width: 260.0,
            card_height: 320.0,
            card_gap: 30.0,
            padding: 16.0,
            inner_gap: 12.0,
            card_normal: SrgbColor {
                r: 0.12,
                g: 0.08,
                b: 0.28,
            },
            card_hover: SrgbColor {
                r: 0.22,
                g: 0.14,
                b: 0.48,
            },
            card_pressed: SrgbColor {
                r: 0.08,
                g: 0.05,
                b: 0.18,
            },
            subtitle_color: SrgbColor {
                r: 0.85,
                g: 0.70,
                b: 0.30,
            },
            text_color: SrgbColor {
                r: 0.95,
                g: 0.90,
                b: 0.85,
            },
            font_size_name: 32.0,
            font_size_subtitle: 24.0,
            font_size_desc: 24.0,
            icon_size: 64.0,
        };
        assert!(
            cfg.card_width > 32.0,
            "card_width must exceed padding*2 for positive max_width"
        );
        assert!(cfg.card_height > 0.0);
        assert!(cfg.card_gap >= 0.0);
    }
}
