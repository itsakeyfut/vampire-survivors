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
// Fallback constants (used while upgrade_card.ron is still loading)
// ---------------------------------------------------------------------------

const DEFAULT_CARD_WIDTH: f32 = 260.0;
const DEFAULT_CARD_HEIGHT: f32 = 320.0;
const DEFAULT_CARD_GAP: f32 = 30.0;
const DEFAULT_PADDING: f32 = 16.0;
const DEFAULT_INNER_GAP: f32 = 12.0;
const DEFAULT_CARD_NORMAL: Color = Color::srgb(0.12, 0.08, 0.28);
const DEFAULT_CARD_HOVER: Color = Color::srgb(0.22, 0.14, 0.48);
const DEFAULT_CARD_PRESSED: Color = Color::srgb(0.08, 0.05, 0.18);
const DEFAULT_SUBTITLE_COLOR: Color = Color::srgb(0.85, 0.70, 0.30);
const DEFAULT_TEXT_COLOR: Color = Color::srgb(0.95, 0.90, 0.85);
const DEFAULT_FONT_SIZE_NAME: f32 = 32.0;
const DEFAULT_FONT_SIZE_SUBTITLE: f32 = 24.0;
const DEFAULT_FONT_SIZE_DESC: f32 = 24.0;
const DEFAULT_ICON_SIZE: f32 = 64.0;
const DEFAULT_ICON_COLOR_NEW_WEAPON: Color = Color::srgb(0.25, 0.50, 1.00);
const DEFAULT_ICON_COLOR_WEAPON_UPGRADE: Color = Color::srgb(0.40, 0.70, 1.00);
const DEFAULT_ICON_COLOR_NEW_PASSIVE: Color = Color::srgb(0.20, 0.75, 0.50);
const DEFAULT_ICON_COLOR_PASSIVE_UPGRADE: Color = Color::srgb(0.40, 0.90, 0.65);

// ---------------------------------------------------------------------------
// Config asset
// ---------------------------------------------------------------------------

/// Deserialization mirror of [`UpgradeCardHudConfig`] — every field is `Option<T>` so
/// RON files with missing fields still load and emit a `warn!` instead of failing.
#[derive(Deserialize, Default)]
#[serde(default, rename = "UpgradeCardHudConfig")]
pub(crate) struct UpgradeCardHudConfigPartial {
    pub card_width: Option<f32>,
    pub card_height: Option<f32>,
    pub card_gap: Option<f32>,
    pub padding: Option<f32>,
    pub inner_gap: Option<f32>,
    pub card_normal: Option<SrgbColor>,
    pub card_hover: Option<SrgbColor>,
    pub card_pressed: Option<SrgbColor>,
    pub subtitle_color: Option<SrgbColor>,
    pub text_color: Option<SrgbColor>,
    pub font_size_name: Option<f32>,
    pub font_size_subtitle: Option<f32>,
    pub font_size_desc: Option<f32>,
    pub icon_size: Option<f32>,
    pub icon_color_new_weapon: Option<SrgbColor>,
    pub icon_color_weapon_upgrade: Option<SrgbColor>,
    pub icon_color_new_passive: Option<SrgbColor>,
    pub icon_color_passive_upgrade: Option<SrgbColor>,
}

/// Upgrade card HUD config loaded from `config/ui/hud/upgrade_card.ron`.
///
/// Covers layout (dimensions, gaps, padding) and visuals (colors, font sizes)
/// for each interactive upgrade card and the row that contains them.
#[derive(Asset, TypePath, Debug, Clone)]
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
    /// Icon placeholder color for "New Weapon" upgrade cards.
    pub icon_color_new_weapon: SrgbColor,
    /// Icon placeholder color for "Weapon Upgrade" cards.
    pub icon_color_weapon_upgrade: SrgbColor,
    /// Icon placeholder color for "New Passive" upgrade cards.
    pub icon_color_new_passive: SrgbColor,
    /// Icon placeholder color for "Passive Upgrade" cards.
    pub icon_color_passive_upgrade: SrgbColor,
}

impl From<UpgradeCardHudConfigPartial> for UpgradeCardHudConfig {
    fn from(p: UpgradeCardHudConfigPartial) -> Self {
        UpgradeCardHudConfig {
            card_width: p.card_width.unwrap_or_else(|| {
                warn!(
                    "upgrade_card.ron: `card_width` missing → using default {DEFAULT_CARD_WIDTH}"
                );
                DEFAULT_CARD_WIDTH
            }),
            card_height: p.card_height.unwrap_or_else(|| {
                warn!(
                    "upgrade_card.ron: `card_height` missing → using default {DEFAULT_CARD_HEIGHT}"
                );
                DEFAULT_CARD_HEIGHT
            }),
            card_gap: p.card_gap.unwrap_or_else(|| {
                warn!("upgrade_card.ron: `card_gap` missing → using default {DEFAULT_CARD_GAP}");
                DEFAULT_CARD_GAP
            }),
            padding: p.padding.unwrap_or_else(|| {
                warn!("upgrade_card.ron: `padding` missing → using default {DEFAULT_PADDING}");
                DEFAULT_PADDING
            }),
            inner_gap: p.inner_gap.unwrap_or_else(|| {
                warn!("upgrade_card.ron: `inner_gap` missing → using default {DEFAULT_INNER_GAP}");
                DEFAULT_INNER_GAP
            }),
            card_normal: p.card_normal.unwrap_or_else(|| {
                warn!("upgrade_card.ron: `card_normal` missing → using default");
                SrgbColor { r: 0.12, g: 0.08, b: 0.28 }
            }),
            card_hover: p.card_hover.unwrap_or_else(|| {
                warn!("upgrade_card.ron: `card_hover` missing → using default");
                SrgbColor { r: 0.22, g: 0.14, b: 0.48 }
            }),
            card_pressed: p.card_pressed.unwrap_or_else(|| {
                warn!("upgrade_card.ron: `card_pressed` missing → using default");
                SrgbColor { r: 0.08, g: 0.05, b: 0.18 }
            }),
            subtitle_color: p.subtitle_color.unwrap_or_else(|| {
                warn!("upgrade_card.ron: `subtitle_color` missing → using default");
                SrgbColor { r: 0.85, g: 0.70, b: 0.30 }
            }),
            text_color: p.text_color.unwrap_or_else(|| {
                warn!("upgrade_card.ron: `text_color` missing → using default");
                SrgbColor { r: 0.95, g: 0.90, b: 0.85 }
            }),
            font_size_name: p.font_size_name.unwrap_or_else(|| {
                warn!(
                    "upgrade_card.ron: `font_size_name` missing → using default {DEFAULT_FONT_SIZE_NAME}"
                );
                DEFAULT_FONT_SIZE_NAME
            }),
            font_size_subtitle: p.font_size_subtitle.unwrap_or_else(|| {
                warn!(
                    "upgrade_card.ron: `font_size_subtitle` missing → using default {DEFAULT_FONT_SIZE_SUBTITLE}"
                );
                DEFAULT_FONT_SIZE_SUBTITLE
            }),
            font_size_desc: p.font_size_desc.unwrap_or_else(|| {
                warn!(
                    "upgrade_card.ron: `font_size_desc` missing → using default {DEFAULT_FONT_SIZE_DESC}"
                );
                DEFAULT_FONT_SIZE_DESC
            }),
            icon_size: p.icon_size.unwrap_or_else(|| {
                warn!("upgrade_card.ron: `icon_size` missing → using default {DEFAULT_ICON_SIZE}");
                DEFAULT_ICON_SIZE
            }),
            icon_color_new_weapon: p.icon_color_new_weapon.unwrap_or_else(|| {
                warn!("upgrade_card.ron: `icon_color_new_weapon` missing → using default");
                SrgbColor { r: 0.25, g: 0.50, b: 1.00 }
            }),
            icon_color_weapon_upgrade: p.icon_color_weapon_upgrade.unwrap_or_else(|| {
                warn!("upgrade_card.ron: `icon_color_weapon_upgrade` missing → using default");
                SrgbColor { r: 0.40, g: 0.70, b: 1.00 }
            }),
            icon_color_new_passive: p.icon_color_new_passive.unwrap_or_else(|| {
                warn!("upgrade_card.ron: `icon_color_new_passive` missing → using default");
                SrgbColor { r: 0.20, g: 0.75, b: 0.50 }
            }),
            icon_color_passive_upgrade: p.icon_color_passive_upgrade.unwrap_or_else(|| {
                warn!("upgrade_card.ron: `icon_color_passive_upgrade` missing → using default");
                SrgbColor { r: 0.40, g: 0.90, b: 0.65 }
            }),
        }
    }
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

    pub fn card_width(&self) -> f32 {
        self.get()
            .map(|c| c.card_width)
            .unwrap_or(DEFAULT_CARD_WIDTH)
    }

    pub fn card_height(&self) -> f32 {
        self.get()
            .map(|c| c.card_height)
            .unwrap_or(DEFAULT_CARD_HEIGHT)
    }

    pub fn card_gap(&self) -> f32 {
        self.get().map(|c| c.card_gap).unwrap_or(DEFAULT_CARD_GAP)
    }

    pub fn padding(&self) -> f32 {
        self.get().map(|c| c.padding).unwrap_or(DEFAULT_PADDING)
    }

    pub fn inner_gap(&self) -> f32 {
        self.get().map(|c| c.inner_gap).unwrap_or(DEFAULT_INNER_GAP)
    }

    pub fn card_normal(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.card_normal))
            .unwrap_or(DEFAULT_CARD_NORMAL)
    }

    pub fn card_hover(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.card_hover))
            .unwrap_or(DEFAULT_CARD_HOVER)
    }

    pub fn card_pressed(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.card_pressed))
            .unwrap_or(DEFAULT_CARD_PRESSED)
    }

    pub fn subtitle_color(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.subtitle_color))
            .unwrap_or(DEFAULT_SUBTITLE_COLOR)
    }

    pub fn text_color(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.text_color))
            .unwrap_or(DEFAULT_TEXT_COLOR)
    }

    pub fn font_size_name(&self) -> f32 {
        self.get()
            .map(|c| c.font_size_name)
            .unwrap_or(DEFAULT_FONT_SIZE_NAME)
    }

    pub fn font_size_subtitle(&self) -> f32 {
        self.get()
            .map(|c| c.font_size_subtitle)
            .unwrap_or(DEFAULT_FONT_SIZE_SUBTITLE)
    }

    pub fn font_size_desc(&self) -> f32 {
        self.get()
            .map(|c| c.font_size_desc)
            .unwrap_or(DEFAULT_FONT_SIZE_DESC)
    }

    pub fn icon_size(&self) -> f32 {
        self.get().map(|c| c.icon_size).unwrap_or(DEFAULT_ICON_SIZE)
    }

    pub fn icon_color_new_weapon(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.icon_color_new_weapon))
            .unwrap_or(DEFAULT_ICON_COLOR_NEW_WEAPON)
    }

    pub fn icon_color_weapon_upgrade(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.icon_color_weapon_upgrade))
            .unwrap_or(DEFAULT_ICON_COLOR_WEAPON_UPGRADE)
    }

    pub fn icon_color_new_passive(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.icon_color_new_passive))
            .unwrap_or(DEFAULT_ICON_COLOR_NEW_PASSIVE)
    }

    pub fn icon_color_passive_upgrade(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.icon_color_passive_upgrade))
            .unwrap_or(DEFAULT_ICON_COLOR_PASSIVE_UPGRADE)
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
    icon_color_new_weapon:      (r: 0.25, g: 0.50, b: 1.00),
    icon_color_weapon_upgrade:  (r: 0.40, g: 0.70, b: 1.00),
    icon_color_new_passive:     (r: 0.20, g: 0.75, b: 0.50),
    icon_color_passive_upgrade: (r: 0.40, g: 0.90, b: 0.65),
)
"#;
        let partial: UpgradeCardHudConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(ron_data)
            .expect("RON parse must succeed");
        let cfg = UpgradeCardHudConfig::from(partial);
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
            icon_color_new_weapon: SrgbColor {
                r: 0.25,
                g: 0.50,
                b: 1.00,
            },
            icon_color_weapon_upgrade: SrgbColor {
                r: 0.40,
                g: 0.70,
                b: 1.00,
            },
            icon_color_new_passive: SrgbColor {
                r: 0.20,
                g: 0.75,
                b: 0.50,
            },
            icon_color_passive_upgrade: SrgbColor {
                r: 0.40,
                g: 0.90,
                b: 0.65,
            },
        };
        assert!(
            cfg.card_width > 32.0,
            "card_width must exceed padding*2 for positive max_width"
        );
        assert!(cfg.card_height > 0.0);
        assert!(cfg.card_gap >= 0.0);
    }
}
