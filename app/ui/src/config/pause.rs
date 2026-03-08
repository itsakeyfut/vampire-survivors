//! Pause screen configuration.
//!
//! Loaded from `assets/config/ui/screen/pause.ron`.
//! Controls the visual appearance of the pause overlay.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

use super::{SrgbColor, SrgbaColor};

// ---------------------------------------------------------------------------
// Config asset
// ---------------------------------------------------------------------------

/// Pause screen style config loaded from `config/ui/screen/pause.ron`.
///
/// The pause overlay is positioned absolutely over the game scene, so
/// `overlay_color` should carry a meaningful alpha value.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct PauseScreenConfig {
    /// Semi-transparent overlay background color (supports alpha).
    pub overlay_color: SrgbaColor,
    /// "PAUSED" / "ポーズ" heading text color.
    pub heading_color: SrgbColor,
}

/// Resource holding the handle to the loaded [`PauseScreenConfig`].
#[derive(Resource)]
pub struct PauseScreenConfigHandle(pub Handle<PauseScreenConfig>);

// ---------------------------------------------------------------------------
// SystemParam bundle
// ---------------------------------------------------------------------------

/// SystemParam bundle for accessing [`PauseScreenConfig`].
///
/// Returns `None` while the asset is still loading or when
/// [`super::UiConfigPlugin`] has not been registered (e.g. in unit tests).
/// Call `.get()` to obtain `Option<&PauseScreenConfig>`.
#[derive(SystemParam)]
pub struct PauseScreenParams<'w> {
    handle: Option<Res<'w, PauseScreenConfigHandle>>,
    assets: Option<Res<'w, Assets<PauseScreenConfig>>>,
}

impl<'w> PauseScreenParams<'w> {
    /// Returns the currently loaded [`PauseScreenConfig`], or `None`.
    pub fn get(&self) -> Option<&PauseScreenConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }
}

// ---------------------------------------------------------------------------
// Hot-reload system
// ---------------------------------------------------------------------------

/// Logs when `config/ui/screen/pause.ron` is loaded or hot-reloaded.
pub fn hot_reload_pause_screen(mut events: MessageReader<AssetEvent<PauseScreenConfig>>) {
    for event in events.read() {
        match event {
            AssetEvent::Added { .. } => {
                info!("✅ Pause screen config loaded");
            }
            AssetEvent::Modified { .. } => {
                info!("🔥 Pause screen config hot-reloaded");
            }
            AssetEvent::Removed { .. } => {
                warn!("⚠️ Pause screen config removed");
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pause_screen_config_deserialization() {
        let ron_data = r#"
PauseScreenConfig(
    overlay_color: (r: 0.02, g: 0.02, b: 0.06, a: 0.88),
    heading_color: (r: 1.0, g: 0.85, b: 0.20),
)
"#;
        let cfg: PauseScreenConfig = ron::de::from_str(ron_data).expect("RON parse must succeed");

        assert!(
            (cfg.overlay_color.a - 0.88).abs() < 1e-6,
            "alpha must be 0.88"
        );
        assert!((cfg.heading_color.r - 1.0).abs() < 1e-6);
    }
}
