//! Large menu button HUD configuration.
//!
//! Loaded from `assets/config/ui/hud/menu_button.ron`.
//! Controls dimensions, font size, and colors for the standard large button
//! widget used on the title and game-over screens.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

use crate::config::SrgbColor;

// ---------------------------------------------------------------------------
// Fallback constants (used while menu_button.ron is still loading)
// ---------------------------------------------------------------------------

const DEFAULT_WIDTH: f32 = 280.0;
const DEFAULT_HEIGHT: f32 = 80.0;
const DEFAULT_FONT_SIZE: f32 = 48.0;
const DEFAULT_COLOR_NORMAL: Color = Color::srgb(0.30, 0.05, 0.05);
const DEFAULT_COLOR_HOVER: Color = Color::srgb(0.60, 0.10, 0.10);
const DEFAULT_COLOR_PRESSED: Color = Color::srgb(0.20, 0.02, 0.02);
const DEFAULT_TEXT_COLOR: Color = Color::srgb(0.95, 0.90, 0.85);

// ---------------------------------------------------------------------------
// Config asset
// ---------------------------------------------------------------------------

/// Deserialization mirror of [`MenuButtonHudConfig`] — every field is `Option<T>` so
/// RON files with missing fields still load and emit a `warn!` instead of failing.
#[derive(Deserialize, Default)]
#[serde(default, rename = "MenuButtonHudConfig")]
pub(crate) struct MenuButtonHudConfigPartial {
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub font_size: Option<f32>,
    pub color_normal: Option<SrgbColor>,
    pub color_hover: Option<SrgbColor>,
    pub color_pressed: Option<SrgbColor>,
    pub text_color: Option<SrgbColor>,
}

/// Large menu button HUD config loaded from `config/ui/hud/menu_button.ron`.
///
/// Covers all visual properties of the primary action button: dimensions,
/// label font size, and the three interaction-state colors.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct MenuButtonHudConfig {
    /// Button width in pixels.
    pub width: f32,
    /// Button height in pixels.
    pub height: f32,
    /// Label text font size in points.
    pub font_size: f32,
    /// Button background color in the resting state.
    pub color_normal: SrgbColor,
    /// Button background color when the cursor is hovering.
    pub color_hover: SrgbColor,
    /// Button background color while the mouse button is held.
    pub color_pressed: SrgbColor,
    /// Label text color.
    pub text_color: SrgbColor,
}

impl From<MenuButtonHudConfigPartial> for MenuButtonHudConfig {
    fn from(p: MenuButtonHudConfigPartial) -> Self {
        MenuButtonHudConfig {
            width: p.width.unwrap_or_else(|| {
                warn!("menu_button.ron: `width` missing → using default {DEFAULT_WIDTH}");
                DEFAULT_WIDTH
            }),
            height: p.height.unwrap_or_else(|| {
                warn!("menu_button.ron: `height` missing → using default {DEFAULT_HEIGHT}");
                DEFAULT_HEIGHT
            }),
            font_size: p.font_size.unwrap_or_else(|| {
                warn!("menu_button.ron: `font_size` missing → using default {DEFAULT_FONT_SIZE}");
                DEFAULT_FONT_SIZE
            }),
            color_normal: p.color_normal.unwrap_or_else(|| {
                warn!("menu_button.ron: `color_normal` missing → using default");
                SrgbColor {
                    r: 0.30,
                    g: 0.05,
                    b: 0.05,
                }
            }),
            color_hover: p.color_hover.unwrap_or_else(|| {
                warn!("menu_button.ron: `color_hover` missing → using default");
                SrgbColor {
                    r: 0.60,
                    g: 0.10,
                    b: 0.10,
                }
            }),
            color_pressed: p.color_pressed.unwrap_or_else(|| {
                warn!("menu_button.ron: `color_pressed` missing → using default");
                SrgbColor {
                    r: 0.20,
                    g: 0.02,
                    b: 0.02,
                }
            }),
            text_color: p.text_color.unwrap_or_else(|| {
                warn!("menu_button.ron: `text_color` missing → using default");
                SrgbColor {
                    r: 0.95,
                    g: 0.90,
                    b: 0.85,
                }
            }),
        }
    }
}

/// Resource holding the handle to the loaded [`MenuButtonHudConfig`].
#[derive(Resource)]
pub struct MenuButtonHudConfigHandle(pub Handle<MenuButtonHudConfig>);

// ---------------------------------------------------------------------------
// SystemParam bundle
// ---------------------------------------------------------------------------

/// SystemParam bundle for accessing [`MenuButtonHudConfig`].
///
/// Returns `None` while the asset is still loading or when
/// [`crate::config::UiConfigPlugin`] is not registered (e.g. in unit tests).
/// Call `.get()` to obtain `Option<&MenuButtonHudConfig>`.
#[derive(SystemParam)]
pub struct MenuButtonHudParams<'w> {
    handle: Option<Res<'w, MenuButtonHudConfigHandle>>,
    assets: Option<Res<'w, Assets<MenuButtonHudConfig>>>,
}

impl<'w> MenuButtonHudParams<'w> {
    /// Returns the currently loaded config, or `None`.
    pub fn get(&self) -> Option<&MenuButtonHudConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }

    pub fn width(&self) -> f32 {
        self.get().map(|c| c.width).unwrap_or(DEFAULT_WIDTH)
    }

    pub fn height(&self) -> f32 {
        self.get().map(|c| c.height).unwrap_or(DEFAULT_HEIGHT)
    }

    pub fn font_size(&self) -> f32 {
        self.get().map(|c| c.font_size).unwrap_or(DEFAULT_FONT_SIZE)
    }

    pub fn color_normal(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.color_normal))
            .unwrap_or(DEFAULT_COLOR_NORMAL)
    }

    pub fn color_hover(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.color_hover))
            .unwrap_or(DEFAULT_COLOR_HOVER)
    }

    pub fn color_pressed(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.color_pressed))
            .unwrap_or(DEFAULT_COLOR_PRESSED)
    }

    pub fn text_color(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.text_color))
            .unwrap_or(DEFAULT_TEXT_COLOR)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_button_hud_config_deserialization() {
        let ron_data = r#"
MenuButtonHudConfig(
    width: 280.0,
    height: 80.0,
    font_size: 48.0,
    color_normal:  (r: 0.30, g: 0.05, b: 0.05),
    color_hover:   (r: 0.60, g: 0.10, b: 0.10),
    color_pressed: (r: 0.20, g: 0.02, b: 0.02),
    text_color:    (r: 0.95, g: 0.90, b: 0.85),
)
"#;
        let partial: MenuButtonHudConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(ron_data)
            .expect("RON parse must succeed");
        let cfg = MenuButtonHudConfig::from(partial);
        assert_eq!(cfg.width, 280.0);
        assert_eq!(cfg.height, 80.0);
        assert_eq!(cfg.font_size, 48.0);
        assert!((cfg.color_normal.r - 0.30).abs() < 1e-6);
        assert!((cfg.color_hover.r - 0.60).abs() < 1e-6);
        assert!((cfg.color_pressed.r - 0.20).abs() < 1e-6);
        assert!((cfg.text_color.r - 0.95).abs() < 1e-6);
    }

    #[test]
    fn button_dimensions_are_positive() {
        let cfg = MenuButtonHudConfig {
            width: 280.0,
            height: 80.0,
            font_size: 48.0,
            color_normal: SrgbColor {
                r: 0.30,
                g: 0.05,
                b: 0.05,
            },
            color_hover: SrgbColor {
                r: 0.60,
                g: 0.10,
                b: 0.10,
            },
            color_pressed: SrgbColor {
                r: 0.20,
                g: 0.02,
                b: 0.02,
            },
            text_color: SrgbColor {
                r: 0.95,
                g: 0.90,
                b: 0.85,
            },
        };
        assert!(cfg.width > 0.0);
        assert!(cfg.height > 0.0);
        assert!(cfg.font_size > 0.0);
    }
}
