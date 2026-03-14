//! Boss warning HUD configuration.
//!
//! Loaded from `assets/config/ui/hud/gameplay/boss_warning.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

use crate::config::SrgbColor;

// ---------------------------------------------------------------------------
// Fallback constants (used while boss_warning.ron is still loading)
// ---------------------------------------------------------------------------

const DEFAULT_DISPLAY_DURATION: f32 = 4.0;
const DEFAULT_FADE_START: f32 = 2.0;
const DEFAULT_FONT_SIZE: f32 = 52.0;
const DEFAULT_TOP_PERCENT: f32 = 35.0;
const DEFAULT_TEXT_COLOR: Color = Color::srgb(1.0, 0.15, 0.15);

/// Boss warning HUD config loaded from
/// `config/ui/hud/gameplay/boss_warning.ron`.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct BossWarningHudConfig {
    /// Total display duration before the notification is despawned (seconds).
    pub display_duration: f32,
    /// Time after which the alpha fade-out begins (seconds).
    pub fade_start: f32,
    /// Font size of the warning text in points.
    pub font_size: f32,
    /// Vertical position as a percentage of the screen height.
    pub top_percent: f32,
    /// Warning text color.
    pub text_color: SrgbColor,
}

/// Resource holding the handle to the loaded [`BossWarningHudConfig`].
#[derive(Resource)]
pub struct BossWarningHudConfigHandle(pub Handle<BossWarningHudConfig>);

/// SystemParam for accessing [`BossWarningHudConfig`].
///
/// Returns `None` while the asset is loading or the plugin is absent.
#[derive(SystemParam)]
pub struct BossWarningHudParams<'w> {
    handle: Option<Res<'w, BossWarningHudConfigHandle>>,
    assets: Option<Res<'w, Assets<BossWarningHudConfig>>>,
}

impl<'w> BossWarningHudParams<'w> {
    /// Returns the currently loaded config, or `None`.
    pub fn get(&self) -> Option<&BossWarningHudConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }

    pub fn display_duration(&self) -> f32 {
        self.get()
            .map(|c| c.display_duration)
            .unwrap_or(DEFAULT_DISPLAY_DURATION)
    }

    pub fn fade_start(&self) -> f32 {
        self.get()
            .map(|c| c.fade_start)
            .unwrap_or(DEFAULT_FADE_START)
    }

    pub fn font_size(&self) -> f32 {
        self.get().map(|c| c.font_size).unwrap_or(DEFAULT_FONT_SIZE)
    }

    pub fn top_percent(&self) -> f32 {
        self.get()
            .map(|c| c.top_percent)
            .unwrap_or(DEFAULT_TOP_PERCENT)
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
BossWarningHudConfig(
    display_duration: 4.0,
    fade_start:       2.0,
    font_size:        52.0,
    top_percent:      35.0,
    text_color:       (r: 1.0, g: 0.15, b: 0.15),
)
"#;

    #[test]
    fn boss_warning_hud_config_deserialization() {
        let cfg: BossWarningHudConfig = ron::de::from_str(RON).expect("RON parse must succeed");
        assert_eq!(cfg.display_duration, 4.0);
        assert_eq!(cfg.fade_start, 2.0);
        assert_eq!(cfg.font_size, 52.0);
        assert_eq!(cfg.top_percent, 35.0);
        assert!((cfg.text_color.r - 1.0).abs() < 1e-6);
        assert!((cfg.text_color.g - 0.15).abs() < 1e-6);
        assert!((cfg.text_color.b - 0.15).abs() < 1e-6);
    }

    #[test]
    fn display_duration_is_positive() {
        let cfg: BossWarningHudConfig = ron::de::from_str(RON).unwrap();
        assert!(cfg.display_duration > 0.0);
    }

    #[test]
    fn fade_start_is_before_display_duration() {
        let cfg: BossWarningHudConfig = ron::de::from_str(RON).unwrap();
        assert!(cfg.fade_start < cfg.display_duration);
    }
}
