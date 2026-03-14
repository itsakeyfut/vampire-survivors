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

/// Deserialization mirror of [`LevelHudConfig`] — every field is `Option<T>` so
/// RON files with missing fields still load and emit a `warn!` instead of failing.
#[derive(Deserialize, Default)]
#[serde(default, rename = "LevelHudConfig")]
pub(crate) struct LevelHudConfigPartial {
    pub font_size: Option<f32>,
    pub text_color: Option<SrgbColor>,
}

/// Level label HUD config loaded from `config/ui/hud/gameplay/level.ron`.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct LevelHudConfig {
    /// Font size of the level label in points.
    pub font_size: f32,
    /// Level label text color.
    pub text_color: SrgbColor,
}

impl From<LevelHudConfigPartial> for LevelHudConfig {
    fn from(p: LevelHudConfigPartial) -> Self {
        LevelHudConfig {
            font_size: p.font_size.unwrap_or_else(|| {
                warn!("level.ron: `font_size` missing → using default {DEFAULT_FONT_SIZE}");
                DEFAULT_FONT_SIZE
            }),
            text_color: p.text_color.unwrap_or_else(|| {
                warn!("level.ron: `text_color` missing → using default");
                SrgbColor {
                    r: 0.95,
                    g: 0.90,
                    b: 0.85,
                }
            }),
        }
    }
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
        let partial: LevelHudConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(RON)
            .expect("RON parse must succeed");
        let cfg = LevelHudConfig::from(partial);
        assert_eq!(cfg.font_size, 22.0);
        assert!((cfg.text_color.b - 0.85).abs() < 1e-6);
    }

    #[test]
    fn level_font_size_is_positive() {
        let partial: LevelHudConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(RON)
            .unwrap();
        let cfg = LevelHudConfig::from(partial);
        assert!(cfg.font_size > 0.0);
    }
}
