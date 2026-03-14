//! Level label HUD configuration.
//!
//! Loaded from `assets/config/ui/hud/gameplay/level.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

use crate::config::SrgbColor;

// ---------------------------------------------------------------------------
// Fallback constants (used while level.ron is still loading)
// ---------------------------------------------------------------------------

const DEFAULT_FONT_SIZE: f32 = 22.0;
const DEFAULT_TEXT_COLOR: Color = Color::srgb(0.95, 0.90, 0.85);

/// Level label HUD config loaded from `config/ui/hud/gameplay/level.ron`.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct LevelHudConfig {
    /// Font size of the level label in points.
    pub font_size: f32,
    /// Level label text color.
    pub text_color: SrgbColor,
}

/// Resource holding the handle to the loaded [`LevelHudConfig`].
#[derive(Resource)]
pub struct LevelHudConfigHandle(pub Handle<LevelHudConfig>);

/// SystemParam for accessing [`LevelHudConfig`].
///
/// Returns `None` while the asset is loading or the plugin is absent.
#[derive(SystemParam)]
pub struct LevelHudParams<'w> {
    handle: Option<Res<'w, LevelHudConfigHandle>>,
    assets: Option<Res<'w, Assets<LevelHudConfig>>>,
}

impl<'w> LevelHudParams<'w> {
    /// Returns the currently loaded config, or `None`.
    pub fn get(&self) -> Option<&LevelHudConfig> {
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
LevelHudConfig(
    font_size:  22.0,
    text_color: (r: 0.95, g: 0.90, b: 0.85),
)
"#;

    #[test]
    fn level_hud_config_deserialization() {
        let cfg: LevelHudConfig = ron::de::from_str(RON).expect("RON parse must succeed");
        assert_eq!(cfg.font_size, 22.0);
        assert!((cfg.text_color.b - 0.85).abs() < 1e-6);
    }

    #[test]
    fn level_font_size_is_positive() {
        let cfg: LevelHudConfig = ron::de::from_str(RON).unwrap();
        assert!(cfg.font_size > 0.0);
    }
}
