//! HP bar HUD configuration.
//!
//! Loaded from `assets/config/ui/hud/gameplay/hp_bar.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

use crate::config::SrgbColor;

// ---------------------------------------------------------------------------
// Fallback constants (used while hp_bar.ron is still loading)
// ---------------------------------------------------------------------------

const DEFAULT_BAR_WIDTH: f32 = 200.0;
const DEFAULT_BAR_HEIGHT: f32 = 16.0;
const DEFAULT_BAR_RADIUS: f32 = 4.0;
const DEFAULT_LABEL_FONT_SIZE: f32 = 14.0;
const DEFAULT_LABEL_GAP: f32 = 4.0;
const DEFAULT_FILL_COLOR: Color = Color::srgb(0.20, 0.80, 0.20);
const DEFAULT_FILL_COLOR_MID: Color = Color::srgb(0.90, 0.80, 0.10);
const DEFAULT_FILL_COLOR_LOW: Color = Color::srgb(0.85, 0.20, 0.20);
const DEFAULT_TRACK_COLOR: Color = Color::srgb(0.10, 0.10, 0.10);
const DEFAULT_TEXT_COLOR: Color = Color::srgb(0.95, 0.90, 0.85);

/// HP bar HUD config loaded from `config/ui/hud/gameplay/hp_bar.ron`.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct HpBarHudConfig {
    /// Width of the bar track in pixels.
    pub bar_width: f32,
    /// Height of the bar track in pixels.
    pub bar_height: f32,
    /// Corner radius of the track and fill in pixels.
    pub bar_radius: f32,
    /// Font size of the "HP" label in points.
    pub label_font_size: f32,
    /// Gap between the label and the track in pixels.
    pub label_gap: f32,
    /// Fill color when HP > 50% (green).
    pub fill_color: SrgbColor,
    /// Fill color when HP is 25–50% (yellow).
    pub fill_color_mid: SrgbColor,
    /// Fill color when HP ≤ 25% (red).
    pub fill_color_low: SrgbColor,
    /// Track color (shown for missing HP).
    pub track_color: SrgbColor,
    /// Label text color.
    pub text_color: SrgbColor,
}

/// Resource holding the handle to the loaded [`HpBarHudConfig`].
#[derive(Resource)]
pub struct HpBarHudConfigHandle(pub Handle<HpBarHudConfig>);

/// SystemParam for accessing [`HpBarHudConfig`].
///
/// Returns `None` while the asset is loading or the plugin is absent.
#[derive(SystemParam)]
pub struct HpBarHudParams<'w> {
    handle: Option<Res<'w, HpBarHudConfigHandle>>,
    assets: Option<Res<'w, Assets<HpBarHudConfig>>>,
}

impl<'w> HpBarHudParams<'w> {
    /// Returns the currently loaded config, or `None`.
    pub fn get(&self) -> Option<&HpBarHudConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }

    pub fn bar_width(&self) -> f32 {
        self.get().map(|c| c.bar_width).unwrap_or(DEFAULT_BAR_WIDTH)
    }

    pub fn bar_height(&self) -> f32 {
        self.get()
            .map(|c| c.bar_height)
            .unwrap_or(DEFAULT_BAR_HEIGHT)
    }

    pub fn bar_radius(&self) -> f32 {
        self.get()
            .map(|c| c.bar_radius)
            .unwrap_or(DEFAULT_BAR_RADIUS)
    }

    pub fn label_font_size(&self) -> f32 {
        self.get()
            .map(|c| c.label_font_size)
            .unwrap_or(DEFAULT_LABEL_FONT_SIZE)
    }

    pub fn label_gap(&self) -> f32 {
        self.get().map(|c| c.label_gap).unwrap_or(DEFAULT_LABEL_GAP)
    }

    pub fn fill_color(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.fill_color))
            .unwrap_or(DEFAULT_FILL_COLOR)
    }

    pub fn fill_color_mid(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.fill_color_mid))
            .unwrap_or(DEFAULT_FILL_COLOR_MID)
    }

    pub fn fill_color_low(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.fill_color_low))
            .unwrap_or(DEFAULT_FILL_COLOR_LOW)
    }

    pub fn track_color(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.track_color))
            .unwrap_or(DEFAULT_TRACK_COLOR)
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

    const RON: &str = r#"
HpBarHudConfig(
    bar_width:        200.0,
    bar_height:       16.0,
    bar_radius:       4.0,
    label_font_size:  14.0,
    label_gap:        4.0,
    fill_color:       (r: 0.20, g: 0.80, b: 0.20),
    fill_color_mid:   (r: 0.90, g: 0.80, b: 0.10),
    fill_color_low:   (r: 0.85, g: 0.20, b: 0.20),
    track_color:      (r: 0.10, g: 0.10, b: 0.10),
    text_color:       (r: 0.95, g: 0.90, b: 0.85),
)
"#;

    #[test]
    fn hp_bar_hud_config_deserialization() {
        let cfg: HpBarHudConfig = ron::de::from_str(RON).expect("RON parse must succeed");
        assert_eq!(cfg.bar_width, 200.0);
        assert_eq!(cfg.bar_height, 16.0);
        assert_eq!(cfg.bar_radius, 4.0);
        assert_eq!(cfg.label_font_size, 14.0);
        assert_eq!(cfg.label_gap, 4.0);
        assert!((cfg.fill_color.g - 0.80).abs() < 1e-6); // green high-HP
        assert!((cfg.fill_color_mid.r - 0.90).abs() < 1e-6); // yellow mid-HP
        assert!((cfg.fill_color_low.r - 0.85).abs() < 1e-6); // red low-HP
    }

    #[test]
    fn hp_bar_dimensions_are_positive() {
        let cfg: HpBarHudConfig = ron::de::from_str(RON).unwrap();
        assert!(cfg.bar_width > 0.0);
        assert!(cfg.bar_height > 0.0);
        assert!(cfg.label_font_size > 0.0);
    }
}
