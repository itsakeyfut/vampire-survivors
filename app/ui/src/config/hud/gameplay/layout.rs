//! Gameplay HUD layout configuration.
//!
//! Loaded from `assets/config/ui/hud/gameplay/layout.ron`.
//! Controls the positioning of each widget anchor within the HUD overlay.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

/// Gameplay HUD layout config loaded from `config/ui/hud/gameplay/layout.ron`.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct GameplayHudLayoutConfig {
    /// Distance in pixels from each screen edge to the nearest widget anchor.
    pub edge_margin: f32,
}

/// Resource holding the handle to the loaded [`GameplayHudLayoutConfig`].
#[derive(Resource)]
pub struct GameplayHudLayoutConfigHandle(pub Handle<GameplayHudLayoutConfig>);

/// SystemParam for accessing [`GameplayHudLayoutConfig`].
///
/// Returns `None` while the asset is loading or the plugin is absent.
#[derive(SystemParam)]
pub struct GameplayHudLayoutParams<'w> {
    handle: Option<Res<'w, GameplayHudLayoutConfigHandle>>,
    assets: Option<Res<'w, Assets<GameplayHudLayoutConfig>>>,
}

impl<'w> GameplayHudLayoutParams<'w> {
    /// Returns the currently loaded config, or `None`.
    pub fn get(&self) -> Option<&GameplayHudLayoutConfig> {
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
GameplayHudLayoutConfig(
    edge_margin: 12.0,
)
"#;

    #[test]
    fn gameplay_hud_layout_config_deserialization() {
        let cfg: GameplayHudLayoutConfig = ron::de::from_str(RON).expect("RON parse must succeed");
        assert_eq!(cfg.edge_margin, 12.0);
    }

    #[test]
    fn edge_margin_is_non_negative() {
        let cfg: GameplayHudLayoutConfig = ron::de::from_str(RON).unwrap();
        assert!(cfg.edge_margin >= 0.0);
    }
}
