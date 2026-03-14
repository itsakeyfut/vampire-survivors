//! XP bar HUD configuration.
//!
//! Loaded from `assets/config/ui/hud/gameplay/xp_bar.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

use crate::config::SrgbColor;

// ---------------------------------------------------------------------------
// Fallback constants (used while xp_bar.ron is still loading)
// ---------------------------------------------------------------------------

const DEFAULT_BAR_HEIGHT: f32 = 10.0;
const DEFAULT_FILL_COLOR: Color = Color::srgb(0.25, 0.65, 1.00);
const DEFAULT_TRACK_COLOR: Color = Color::srgb(0.05, 0.08, 0.20);

/// Deserialization mirror of [`XpBarHudConfig`] — every field is `Option<T>` so
/// RON files with missing fields still load and emit a `warn!` instead of failing.
#[derive(Deserialize, Default)]
#[serde(default, rename = "XpBarHudConfig")]
pub(crate) struct XpBarHudConfigPartial {
    pub bar_height: Option<f32>,
    pub fill_color: Option<SrgbColor>,
    pub track_color: Option<SrgbColor>,
}

/// XP bar HUD config loaded from `config/ui/hud/gameplay/xp_bar.ron`.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct XpBarHudConfig {
    /// Height of the bar track in pixels.
    pub bar_height: f32,
    /// Fill color (shown for earned XP).
    pub fill_color: SrgbColor,
    /// Track color (shown for remaining XP).
    pub track_color: SrgbColor,
}

impl From<XpBarHudConfigPartial> for XpBarHudConfig {
    fn from(p: XpBarHudConfigPartial) -> Self {
        XpBarHudConfig {
            bar_height: p.bar_height.unwrap_or_else(|| {
                warn!("xp_bar.ron: `bar_height` missing → using default {DEFAULT_BAR_HEIGHT}");
                DEFAULT_BAR_HEIGHT
            }),
            fill_color: p.fill_color.unwrap_or_else(|| {
                warn!("xp_bar.ron: `fill_color` missing → using default");
                SrgbColor {
                    r: 0.25,
                    g: 0.65,
                    b: 1.00,
                }
            }),
            track_color: p.track_color.unwrap_or_else(|| {
                warn!("xp_bar.ron: `track_color` missing → using default");
                SrgbColor {
                    r: 0.05,
                    g: 0.08,
                    b: 0.20,
                }
            }),
        }
    }
}

/// Resource holding the handle to the loaded [`XpBarHudConfig`].
#[derive(Resource)]
pub struct XpBarHudConfigHandle(pub Handle<XpBarHudConfig>);

/// SystemParam for accessing [`XpBarHudConfig`].
///
/// Returns `None` while the asset is loading or the plugin is absent.
#[derive(SystemParam)]
pub struct XpBarHudParams<'w> {
    handle: Option<Res<'w, XpBarHudConfigHandle>>,
    assets: Option<Res<'w, Assets<XpBarHudConfig>>>,
}

impl<'w> XpBarHudParams<'w> {
    /// Returns the currently loaded config, or `None`.
    pub fn get(&self) -> Option<&XpBarHudConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }

    pub fn bar_height(&self) -> f32 {
        self.get()
            .map(|c| c.bar_height)
            .unwrap_or(DEFAULT_BAR_HEIGHT)
    }

    pub fn fill_color(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.fill_color))
            .unwrap_or(DEFAULT_FILL_COLOR)
    }

    pub fn track_color(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.track_color))
            .unwrap_or(DEFAULT_TRACK_COLOR)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const RON: &str = r#"
XpBarHudConfig(
    bar_height:  10.0,
    fill_color:  (r: 0.25, g: 0.65, b: 1.00),
    track_color: (r: 0.05, g: 0.08, b: 0.20),
)
"#;

    #[test]
    fn xp_bar_hud_config_deserialization() {
        let partial: XpBarHudConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(RON)
            .expect("RON parse must succeed");
        let cfg = XpBarHudConfig::from(partial);
        assert_eq!(cfg.bar_height, 10.0);
        assert!((cfg.fill_color.g - 0.65).abs() < 1e-6);
    }

    #[test]
    fn xp_bar_height_is_positive() {
        let partial: XpBarHudConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(RON)
            .unwrap();
        let cfg = XpBarHudConfig::from(partial);
        assert!(cfg.bar_height > 0.0);
    }
}
