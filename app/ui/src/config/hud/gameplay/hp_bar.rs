//! HP bar HUD configuration.
//!
//! Loaded from `assets/config/ui/hud/gameplay/hp_bar.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

use crate::config::SrgbColor;

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
    /// Fill color (shown for current HP).
    pub fill_color: SrgbColor,
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
    fill_color:       (r: 0.85, g: 0.20, b: 0.20),
    track_color:      (r: 0.20, g: 0.05, b: 0.05),
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
        assert!((cfg.fill_color.r - 0.85).abs() < 1e-6);
    }

    #[test]
    fn hp_bar_dimensions_are_positive() {
        let cfg: HpBarHudConfig = ron::de::from_str(RON).unwrap();
        assert!(cfg.bar_width > 0.0);
        assert!(cfg.bar_height > 0.0);
        assert!(cfg.label_font_size > 0.0);
    }
}
