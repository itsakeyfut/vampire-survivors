//! Screen heading HUD configuration.
//!
//! Loaded from `assets/config/ui/hud/screen_heading.ron`.
//! Controls font size and bottom margin for all screen heading widgets.
//! Text colour is screen-specific and passed directly to the spawn function.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Fallback constants (used while screen_heading.ron is still loading)
// ---------------------------------------------------------------------------

const DEFAULT_FONT_SIZE: f32 = 72.0;
const DEFAULT_MARGIN_BOTTOM: f32 = 80.0;

// ---------------------------------------------------------------------------
// Config asset
// ---------------------------------------------------------------------------

/// Screen heading HUD config loaded from `config/ui/hud/screen_heading.ron`.
///
/// Controls layout parameters shared by all screen heading widgets.
/// Text colour is screen-specific and passed to the spawn function directly.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct ScreenHeadingHudConfig {
    /// Font size of the heading text in points.
    pub font_size: f32,
    /// Bottom margin below the heading in pixels (spacing before the next element).
    pub margin_bottom: f32,
}

/// Resource holding the handle to the loaded [`ScreenHeadingHudConfig`].
#[derive(Resource)]
pub struct ScreenHeadingHudConfigHandle(pub Handle<ScreenHeadingHudConfig>);

// ---------------------------------------------------------------------------
// SystemParam bundle
// ---------------------------------------------------------------------------

/// SystemParam bundle for accessing [`ScreenHeadingHudConfig`].
///
/// Returns `None` while the asset is still loading or when
/// [`crate::config::UiConfigPlugin`] is not registered (e.g. in unit tests).
/// Call `.get()` to obtain `Option<&ScreenHeadingHudConfig>`.
#[derive(SystemParam)]
pub struct ScreenHeadingHudParams<'w> {
    handle: Option<Res<'w, ScreenHeadingHudConfigHandle>>,
    assets: Option<Res<'w, Assets<ScreenHeadingHudConfig>>>,
}

impl<'w> ScreenHeadingHudParams<'w> {
    /// Returns the currently loaded config, or `None`.
    pub fn get(&self) -> Option<&ScreenHeadingHudConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }

    pub fn font_size(&self) -> f32 {
        self.get().map(|c| c.font_size).unwrap_or(DEFAULT_FONT_SIZE)
    }

    pub fn margin_bottom(&self) -> f32 {
        self.get()
            .map(|c| c.margin_bottom)
            .unwrap_or(DEFAULT_MARGIN_BOTTOM)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screen_heading_hud_config_deserialization() {
        let ron_data = r#"
ScreenHeadingHudConfig(
    font_size: 72.0,
    margin_bottom: 80.0,
)
"#;
        let cfg: ScreenHeadingHudConfig =
            ron::de::from_str(ron_data).expect("RON parse must succeed");
        assert_eq!(cfg.font_size, 72.0);
        assert_eq!(cfg.margin_bottom, 80.0);
    }

    #[test]
    fn default_values_are_positive() {
        // Sanity check for expected default values in the RON file.
        let cfg = ScreenHeadingHudConfig {
            font_size: 72.0,
            margin_bottom: 80.0,
        };
        assert!(cfg.font_size > 0.0);
        assert!(cfg.margin_bottom > 0.0);
    }
}
