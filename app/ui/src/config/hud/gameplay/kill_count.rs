//! Kill count HUD configuration.
//!
//! Loaded from `assets/config/ui/hud/gameplay/kill_count.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

use crate::config::SrgbColor;

// ---------------------------------------------------------------------------
// Fallback constants (used while kill_count.ron is still loading)
// ---------------------------------------------------------------------------

const DEFAULT_FONT_SIZE: f32 = 14.0;
const DEFAULT_TEXT_COLOR: Color = Color::srgb(0.95, 0.90, 0.85);

/// Kill count HUD config loaded from `config/ui/hud/gameplay/kill_count.ron`.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct KillCountHudConfig {
    /// Font size of the kill count label in points.
    pub font_size: f32,
    /// Text color of the kill count label.
    pub text_color: SrgbColor,
}

/// Resource holding the handle to the loaded [`KillCountHudConfig`].
#[derive(Resource)]
pub struct KillCountHudConfigHandle(pub Handle<KillCountHudConfig>);

/// SystemParam for accessing [`KillCountHudConfig`].
///
/// Returns `None` while the asset is loading or the plugin is absent.
#[derive(SystemParam)]
pub struct KillCountHudParams<'w> {
    handle: Option<Res<'w, KillCountHudConfigHandle>>,
    assets: Option<Res<'w, Assets<KillCountHudConfig>>>,
}

impl<'w> KillCountHudParams<'w> {
    /// Returns the currently loaded config, or `None`.
    pub fn get(&self) -> Option<&KillCountHudConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }

    pub fn font_size(&self) -> f32 {
        self.get().map(|c| c.font_size).unwrap_or(DEFAULT_FONT_SIZE)
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
KillCountHudConfig(
    font_size:  14.0,
    text_color: (r: 0.95, g: 0.90, b: 0.85),
)
"#;

    #[test]
    fn kill_count_hud_config_deserialization() {
        let cfg: KillCountHudConfig = ron::de::from_str(RON).expect("RON parse must succeed");
        assert_eq!(cfg.font_size, 14.0);
        assert!((cfg.text_color.r - 0.95).abs() < 1e-6);
    }

    #[test]
    fn font_size_is_positive() {
        let cfg: KillCountHudConfig = ron::de::from_str(RON).unwrap();
        assert!(cfg.font_size > 0.0);
    }
}
