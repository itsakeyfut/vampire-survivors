//! Gold HUD configuration.
//!
//! Loaded from `assets/config/ui/hud/gameplay/gold.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

use crate::config::SrgbColor;

// ---------------------------------------------------------------------------
// Fallback constants (used while gold.ron is still loading)
// ---------------------------------------------------------------------------

const DEFAULT_FONT_SIZE: f32 = 14.0;
const DEFAULT_TEXT_COLOR: Color = Color::srgb(1.0, 0.85, 0.2);
const DEFAULT_VERTICAL_OFFSET: f32 = 20.0;

/// Deserialization mirror of [`GoldHudConfig`] — every field is `Option<T>` so
/// RON files with missing fields still load and emit a `warn!` instead of failing.
#[derive(Deserialize, Default)]
#[serde(default, rename = "GoldHudConfig")]
pub(crate) struct GoldHudConfigPartial {
    pub font_size: Option<f32>,
    pub text_color: Option<SrgbColor>,
    pub vertical_offset: Option<f32>,
}

/// Gold HUD config loaded from `config/ui/hud/gameplay/gold.ron`.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct GoldHudConfig {
    /// Font size of the gold label in points.
    pub font_size: f32,
    /// Text color of the gold label.
    pub text_color: SrgbColor,
    /// Extra vertical offset (px) added on top of `BOTTOM_WIDGET_OFFSET` to
    /// place the gold label one line above the kill count.
    pub vertical_offset: f32,
}

impl From<GoldHudConfigPartial> for GoldHudConfig {
    fn from(p: GoldHudConfigPartial) -> Self {
        GoldHudConfig {
            font_size: p.font_size.unwrap_or_else(|| {
                warn!("gold.ron: `font_size` missing → using default {DEFAULT_FONT_SIZE}");
                DEFAULT_FONT_SIZE
            }),
            text_color: p.text_color.unwrap_or_else(|| {
                warn!("gold.ron: `text_color` missing → using default");
                SrgbColor { r: 1.0, g: 0.85, b: 0.2 }
            }),
            vertical_offset: p.vertical_offset.unwrap_or_else(|| {
                warn!(
                    "gold.ron: `vertical_offset` missing → using default {DEFAULT_VERTICAL_OFFSET}"
                );
                DEFAULT_VERTICAL_OFFSET
            }),
        }
    }
}

/// Resource holding the handle to the loaded [`GoldHudConfig`].
#[derive(Resource)]
pub struct GoldHudConfigHandle(pub Handle<GoldHudConfig>);

/// SystemParam for accessing [`GoldHudConfig`].
///
/// Returns `None` while the asset is loading or the plugin is absent.
#[derive(SystemParam)]
pub struct GoldHudParams<'w> {
    handle: Option<Res<'w, GoldHudConfigHandle>>,
    assets: Option<Res<'w, Assets<GoldHudConfig>>>,
}

impl<'w> GoldHudParams<'w> {
    /// Returns the currently loaded config, or `None`.
    pub fn get(&self) -> Option<&GoldHudConfig> {
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

    pub fn vertical_offset(&self) -> f32 {
        self.get()
            .map(|c| c.vertical_offset)
            .unwrap_or(DEFAULT_VERTICAL_OFFSET)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const RON: &str = r#"
GoldHudConfig(
    font_size:       14.0,
    text_color:      (r: 1.0, g: 0.85, b: 0.2),
    vertical_offset: 20.0,
)
"#;

    #[test]
    fn gold_hud_config_deserialization() {
        let partial: GoldHudConfigPartial = ron::Options::default().with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME).from_str(RON).expect("RON parse must succeed");
        let cfg = GoldHudConfig::from(partial);
        assert_eq!(cfg.font_size, 14.0);
        assert!((cfg.text_color.r - 1.0).abs() < 1e-6);
        assert!((cfg.text_color.g - 0.85).abs() < 1e-6);
        assert_eq!(cfg.vertical_offset, 20.0);
    }

    #[test]
    fn font_size_is_positive() {
        let partial: GoldHudConfigPartial = ron::Options::default().with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME).from_str(RON).unwrap();
        let cfg = GoldHudConfig::from(partial);
        assert!(cfg.font_size > 0.0);
    }
}
