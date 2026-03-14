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

/// Deserialization mirror of [`KillCountHudConfig`] — every field is `Option<T>` so
/// RON files with missing fields still load and emit a `warn!` instead of failing.
#[derive(Deserialize, Default)]
#[serde(default, rename = "KillCountHudConfig")]
pub(crate) struct KillCountHudConfigPartial {
    pub font_size: Option<f32>,
    pub text_color: Option<SrgbColor>,
}

/// Kill count HUD config loaded from `config/ui/hud/gameplay/kill_count.ron`.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct KillCountHudConfig {
    /// Font size of the kill count label in points.
    pub font_size: f32,
    /// Text color of the kill count label.
    pub text_color: SrgbColor,
}

impl From<KillCountHudConfigPartial> for KillCountHudConfig {
    fn from(p: KillCountHudConfigPartial) -> Self {
        KillCountHudConfig {
            font_size: p.font_size.unwrap_or_else(|| {
                warn!("kill_count.ron: `font_size` missing → using default {DEFAULT_FONT_SIZE}");
                DEFAULT_FONT_SIZE
            }),
            text_color: p.text_color.unwrap_or_else(|| {
                warn!("kill_count.ron: `text_color` missing → using default");
                SrgbColor { r: 0.95, g: 0.90, b: 0.85 }
            }),
        }
    }
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
        let partial: KillCountHudConfigPartial =
            ron::Options::default().with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME).from_str(RON).expect("RON parse must succeed");
        let cfg = KillCountHudConfig::from(partial);
        assert_eq!(cfg.font_size, 14.0);
        assert!((cfg.text_color.r - 0.95).abs() < 1e-6);
    }

    #[test]
    fn font_size_is_positive() {
        let partial: KillCountHudConfigPartial = ron::Options::default().with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME).from_str(RON).unwrap();
        let cfg = KillCountHudConfig::from(partial);
        assert!(cfg.font_size > 0.0);
    }
}
