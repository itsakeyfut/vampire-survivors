//! Gold HUD configuration.
//!
//! Loaded from `assets/config/ui/hud/gameplay/gold.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

use crate::config::SrgbColor;

/// Gold HUD config loaded from `config/ui/hud/gameplay/gold.ron`.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct GoldHudConfig {
    /// Font size of the gold label in points.
    pub font_size: f32,
    /// Text color of the gold label.
    pub text_color: SrgbColor,
    /// Extra vertical offset (px) added on top of `BOTTOM_WIDGET_OFFSET` to
    /// place the gold label one line above the kill count.
    pub vertical_offset: f32,
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
        let cfg: GoldHudConfig = ron::de::from_str(RON).expect("RON parse must succeed");
        assert_eq!(cfg.font_size, 14.0);
        assert!((cfg.text_color.r - 1.0).abs() < 1e-6);
        assert!((cfg.text_color.g - 0.85).abs() < 1e-6);
        assert_eq!(cfg.vertical_offset, 20.0);
    }

    #[test]
    fn font_size_is_positive() {
        let cfg: GoldHudConfig = ron::de::from_str(RON).unwrap();
        assert!(cfg.font_size > 0.0);
    }
}
