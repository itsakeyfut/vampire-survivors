//! Timer HUD configuration.
//!
//! Loaded from `assets/config/ui/hud/gameplay/timer.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

use crate::config::SrgbColor;

/// Timer HUD config loaded from `config/ui/hud/gameplay/timer.ron`.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct TimerHudConfig {
    /// Font size of the timer text in points.
    pub font_size: f32,
    /// Timer text color.
    pub text_color: SrgbColor,
}

/// Resource holding the handle to the loaded [`TimerHudConfig`].
#[derive(Resource)]
pub struct TimerHudConfigHandle(pub Handle<TimerHudConfig>);

/// SystemParam for accessing [`TimerHudConfig`].
///
/// Returns `None` while the asset is loading or the plugin is absent.
#[derive(SystemParam)]
pub struct TimerHudParams<'w> {
    handle: Option<Res<'w, TimerHudConfigHandle>>,
    assets: Option<Res<'w, Assets<TimerHudConfig>>>,
}

impl<'w> TimerHudParams<'w> {
    /// Returns the currently loaded config, or `None`.
    pub fn get(&self) -> Option<&TimerHudConfig> {
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
TimerHudConfig(
    font_size:  28.0,
    text_color: (r: 0.95, g: 0.90, b: 0.85),
)
"#;

    #[test]
    fn timer_hud_config_deserialization() {
        let cfg: TimerHudConfig = ron::de::from_str(RON).expect("RON parse must succeed");
        assert_eq!(cfg.font_size, 28.0);
        assert!((cfg.text_color.r - 0.95).abs() < 1e-6);
    }

    #[test]
    fn timer_font_size_is_positive() {
        let cfg: TimerHudConfig = ron::de::from_str(RON).unwrap();
        assert!(cfg.font_size > 0.0);
    }
}
