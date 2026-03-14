//! Gameplay HUD layout configuration.
//!
//! Loaded from `assets/config/ui/hud/gameplay/layout.ron`.
//! Controls the positioning of each widget anchor within the HUD overlay.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Fallback constants (used while layout.ron is still loading)
// ---------------------------------------------------------------------------

const DEFAULT_EDGE_MARGIN: f32 = 12.0;

/// Deserialization mirror of [`GameplayHudLayoutConfig`] — every field is `Option<T>` so
/// RON files with missing fields still load and emit a `warn!` instead of failing.
#[derive(Deserialize, Default)]
#[serde(default, rename = "GameplayHudLayoutConfig")]
pub(crate) struct GameplayHudLayoutConfigPartial {
    pub edge_margin: Option<f32>,
}

/// Gameplay HUD layout config loaded from `config/ui/hud/gameplay/layout.ron`.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct GameplayHudLayoutConfig {
    /// Distance in pixels from each screen edge to the nearest widget anchor.
    pub edge_margin: f32,
}

impl From<GameplayHudLayoutConfigPartial> for GameplayHudLayoutConfig {
    fn from(p: GameplayHudLayoutConfigPartial) -> Self {
        GameplayHudLayoutConfig {
            edge_margin: p.edge_margin.unwrap_or_else(|| {
                warn!("layout.ron: `edge_margin` missing → using default {DEFAULT_EDGE_MARGIN}");
                DEFAULT_EDGE_MARGIN
            }),
        }
    }
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

    pub fn edge_margin(&self) -> f32 {
        self.get()
            .map(|c| c.edge_margin)
            .unwrap_or(DEFAULT_EDGE_MARGIN)
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
        let partial: GameplayHudLayoutConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(RON)
            .expect("RON parse must succeed");
        let cfg = GameplayHudLayoutConfig::from(partial);
        assert_eq!(cfg.edge_margin, 12.0);
    }

    #[test]
    fn edge_margin_is_non_negative() {
        let partial: GameplayHudLayoutConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(RON)
            .unwrap();
        let cfg = GameplayHudLayoutConfig::from(partial);
        assert!(cfg.edge_margin >= 0.0);
    }
}
