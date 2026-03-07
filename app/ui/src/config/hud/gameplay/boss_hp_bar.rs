//! Boss HP bar HUD configuration.
//!
//! Loaded from `assets/config/ui/hud/gameplay/boss_hp_bar.ron`.
//!
//! The bar is rendered as world-space child sprites of the boss entity so it
//! moves with the boss rather than being fixed to the screen.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

use crate::config::SrgbColor;

// ---------------------------------------------------------------------------
// Config asset
// ---------------------------------------------------------------------------

/// Boss HP bar config loaded from `config/ui/hud/gameplay/boss_hp_bar.ron`.
///
/// All values are in world-space pixels relative to the boss entity's origin.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct BossHpBarHudConfig {
    /// Name label shown above the HP track (e.g. `"DEATH"`).
    pub label_text: String,
    /// Width of the bar track in pixels.
    pub bar_width: f32,
    /// Height of the bar track in pixels.
    pub bar_height: f32,
    /// Font size of the label in points.
    pub label_font_size: f32,
    /// Gap between the label and the bar track in pixels.
    pub label_gap: f32,
    /// Vertical offset from the boss center to the bar track center (pixels).
    /// Negative values place the bar below the boss center.
    pub y_offset: f32,
    /// Fill color of the HP bar.
    pub fill_color: SrgbColor,
    /// Background track color.
    pub track_color: SrgbColor,
    /// Text color of the label.
    pub text_color: SrgbColor,
}

/// Resource holding the handle to the loaded [`BossHpBarHudConfig`].
#[derive(Resource)]
pub struct BossHpBarHudConfigHandle(pub Handle<BossHpBarHudConfig>);

// ---------------------------------------------------------------------------
// SystemParam bundle
// ---------------------------------------------------------------------------

/// SystemParam for accessing [`BossHpBarHudConfig`].
///
/// Returns `None` while the asset is loading or the plugin is absent.
#[derive(SystemParam)]
pub struct BossHpBarHudParams<'w> {
    handle: Option<Res<'w, BossHpBarHudConfigHandle>>,
    assets: Option<Res<'w, Assets<BossHpBarHudConfig>>>,
}

impl<'w> BossHpBarHudParams<'w> {
    /// Returns the currently loaded config, or `None`.
    pub fn get(&self) -> Option<&BossHpBarHudConfig> {
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
BossHpBarHudConfig(
    label_text:      "DEATH",
    bar_width:       160.0,
    bar_height:        8.0,
    label_font_size:  12.0,
    label_gap:         4.0,
    y_offset:        -90.0,
    fill_color:  (r: 0.65, g: 0.10, b: 0.85),
    track_color: (r: 0.15, g: 0.05, b: 0.20),
    text_color:  (r: 0.95, g: 0.90, b: 0.85),
)
"#;

    #[test]
    fn boss_hp_bar_hud_config_deserialization() {
        let cfg: BossHpBarHudConfig = ron::de::from_str(RON).expect("RON parse must succeed");
        assert_eq!(cfg.label_text, "DEATH");
        assert_eq!(cfg.bar_width, 160.0);
        assert_eq!(cfg.bar_height, 8.0);
        assert_eq!(cfg.label_font_size, 12.0);
        assert_eq!(cfg.label_gap, 4.0);
        assert_eq!(cfg.y_offset, -90.0);
        assert!((cfg.fill_color.r - 0.65).abs() < 1e-6);
        assert!((cfg.track_color.r - 0.15).abs() < 1e-6);
        assert!((cfg.text_color.r - 0.95).abs() < 1e-6);
    }

    #[test]
    fn bar_dimensions_are_positive() {
        let cfg: BossHpBarHudConfig = ron::de::from_str(RON).unwrap();
        assert!(cfg.bar_width > 0.0);
        assert!(cfg.bar_height > 0.0);
    }
}
