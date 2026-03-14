//! Level-up card selection screen configuration.
//!
//! Loaded from `assets/config/ui/screen/level_up.ron`.
//! Systems read the current values via [`LevelUpScreenParams`] and fall back
//! to private `DEFAULT_*` constants defined in each consumer module.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

use super::{SrgbColor, SrgbaColor};

// ---------------------------------------------------------------------------
// Fallback constants (used while level_up.ron is still loading)
// ---------------------------------------------------------------------------

const DEFAULT_OVERLAY_COLOR: Color = Color::srgba(0.02, 0.02, 0.06, 0.92);
const DEFAULT_HEADING_COLOR: Color = Color::srgb(1.0, 0.85, 0.20);

// ---------------------------------------------------------------------------
// Config asset
// ---------------------------------------------------------------------------

/// Level-up screen style config loaded from
/// `config/ui/screen/level_up.ron`.
///
/// Controls the visual appearance of the card selection overlay.
/// Card-specific colors and dimensions live in
/// [`super::hud::upgrade_card::UpgradeCardHudConfig`].
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct LevelUpScreenConfig {
    /// Semi-transparent overlay background color (supports alpha).
    pub overlay_color: SrgbaColor,
    /// "LEVEL UP!" heading text color.
    pub heading_color: SrgbColor,
}

/// Resource holding the handle to the loaded [`LevelUpScreenConfig`].
#[derive(Resource)]
pub struct LevelUpScreenConfigHandle(pub Handle<LevelUpScreenConfig>);

// ---------------------------------------------------------------------------
// SystemParam bundle
// ---------------------------------------------------------------------------

/// SystemParam bundle for accessing [`LevelUpScreenConfig`].
///
/// Returns `None` while the asset is still loading or when
/// [`super::UiConfigPlugin`] has not been registered (e.g. in unit tests).
/// Call `.get()` to obtain `Option<&LevelUpScreenConfig>`.
#[derive(SystemParam)]
pub struct LevelUpScreenParams<'w> {
    handle: Option<Res<'w, LevelUpScreenConfigHandle>>,
    assets: Option<Res<'w, Assets<LevelUpScreenConfig>>>,
}

impl<'w> LevelUpScreenParams<'w> {
    /// Returns the currently loaded [`LevelUpScreenConfig`], or `None`.
    pub fn get(&self) -> Option<&LevelUpScreenConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }

    pub fn overlay_color(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.overlay_color))
            .unwrap_or(DEFAULT_OVERLAY_COLOR)
    }

    pub fn heading_color(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.heading_color))
            .unwrap_or(DEFAULT_HEADING_COLOR)
    }
}

// ---------------------------------------------------------------------------
// Hot-reload system
// ---------------------------------------------------------------------------

/// Logs when `config/ui/screen/level_up.ron` is loaded or hot-reloaded.
///
/// Because the level-up screen is transient (spawned on enter, despawned on
/// exit), live entity updates are not required — the next time the overlay
/// opens it will read the new values via [`LevelUpScreenParams`].
pub fn hot_reload_level_up_screen(mut events: MessageReader<AssetEvent<LevelUpScreenConfig>>) {
    for event in events.read() {
        match event {
            AssetEvent::Added { .. } => {
                info!("✅ Level-up screen config loaded");
            }
            AssetEvent::Modified { .. } => {
                info!("🔥 Level-up screen config hot-reloaded");
            }
            AssetEvent::Removed { .. } => {
                warn!("⚠️ Level-up screen config removed");
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
    fn level_up_screen_config_deserialization() {
        let ron_data = r#"
LevelUpScreenConfig(
    overlay_color: (r: 0.02, g: 0.02, b: 0.06, a: 0.92),
    heading_color: (r: 1.0,  g: 0.85, b: 0.20),
)
"#;
        let cfg: LevelUpScreenConfig = ron::de::from_str(ron_data).expect("RON parse must succeed");

        assert!(
            (cfg.overlay_color.a - 0.92).abs() < 1e-6,
            "alpha must be 0.92"
        );
        assert!((cfg.heading_color.r - 1.0).abs() < 1e-6);
    }
}
