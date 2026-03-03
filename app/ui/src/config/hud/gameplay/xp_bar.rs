//! XP bar HUD configuration.
//!
//! Loaded from `assets/config/ui/hud/gameplay/xp_bar.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

use crate::config::SrgbColor;

/// XP bar HUD config loaded from `config/ui/hud/gameplay/xp_bar.ron`.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct XpBarHudConfig {
    /// Height of the bar track in pixels.
    pub bar_height: f32,
    /// Fill color (shown for earned XP).
    pub fill_color: SrgbColor,
    /// Track color (shown for remaining XP).
    pub track_color: SrgbColor,
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
        let cfg: XpBarHudConfig = ron::de::from_str(RON).expect("RON parse must succeed");
        assert_eq!(cfg.bar_height, 10.0);
        assert!((cfg.fill_color.g - 0.65).abs() < 1e-6);
    }

    #[test]
    fn xp_bar_height_is_positive() {
        let cfg: XpBarHudConfig = ron::de::from_str(RON).unwrap();
        assert!(cfg.bar_height > 0.0);
    }
}
