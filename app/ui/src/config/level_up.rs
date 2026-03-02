//! Level-up card selection screen configuration.
//!
//! Loaded from `assets/config/ui/screen/level_up.ron`.
//! Systems read the current values via [`LevelUpScreenParams`] and fall back
//! to the `DEFAULT_*` constants defined here when the asset is not yet loaded.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

use super::{SrgbColor, SrgbaColor};

// ---------------------------------------------------------------------------
// Fallback constants
// ---------------------------------------------------------------------------

/// Semi-transparent dark overlay behind the upgrade cards.
pub(crate) const DEFAULT_OVERLAY_COLOR: Color = Color::srgba(0.02, 0.02, 0.06, 0.92);

/// "LEVEL UP!" heading color — gold.
pub(crate) const DEFAULT_HEADING_COLOR: Color = Color::srgb(1.0, 0.85, 0.20);

/// Card background color (resting state).
pub(crate) const DEFAULT_CARD_NORMAL: Color = Color::srgb(0.12, 0.08, 0.28);

/// Card background color on hover.
pub(crate) const DEFAULT_CARD_HOVER: Color = Color::srgb(0.22, 0.14, 0.48);

/// Card background color while pressed.
pub(crate) const DEFAULT_CARD_PRESSED: Color = Color::srgb(0.08, 0.05, 0.18);

/// Upgrade type subtitle text color — dim gold.
pub(crate) const DEFAULT_SUBTITLE_COLOR: Color = Color::srgb(0.85, 0.70, 0.30);

/// Width of each upgrade card in pixels.
pub(crate) const DEFAULT_CARD_WIDTH: f32 = 260.0;

/// Height of each upgrade card in pixels.
pub(crate) const DEFAULT_CARD_HEIGHT: f32 = 320.0;

/// Horizontal gap between adjacent cards in pixels.
pub(crate) const DEFAULT_CARD_GAP: f32 = 30.0;

// ---------------------------------------------------------------------------
// Config asset
// ---------------------------------------------------------------------------

/// Level-up screen style config loaded from
/// `config/ui/screen/level_up.ron`.
///
/// Controls the visual appearance of the card selection overlay: overlay
/// tint, heading and card colors, and card layout dimensions.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct LevelUpScreenConfig {
    /// Semi-transparent overlay background color (supports alpha).
    pub overlay_color: SrgbaColor,
    /// "LEVEL UP!" heading text color.
    pub heading_color: SrgbColor,
    /// Upgrade card background color (resting state).
    pub card_normal: SrgbColor,
    /// Upgrade card background color on hover.
    pub card_hover: SrgbColor,
    /// Upgrade card background color while pressed.
    pub card_pressed: SrgbColor,
    /// Upgrade type subtitle text color.
    pub subtitle_color: SrgbColor,
    /// Width of each upgrade card in pixels.
    pub card_width: f32,
    /// Height of each upgrade card in pixels.
    pub card_height: f32,
    /// Horizontal gap between adjacent cards in pixels.
    pub card_gap: f32,
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
    overlay_color:  (r: 0.02, g: 0.02, b: 0.06, a: 0.92),
    heading_color:  (r: 1.0,  g: 0.85, b: 0.20),
    card_normal:    (r: 0.12, g: 0.08, b: 0.28),
    card_hover:     (r: 0.22, g: 0.14, b: 0.48),
    card_pressed:   (r: 0.08, g: 0.05, b: 0.18),
    subtitle_color: (r: 0.85, g: 0.70, b: 0.30),
    card_width:     260.0,
    card_height:    320.0,
    card_gap:        30.0,
)
"#;
        let cfg: LevelUpScreenConfig = ron::de::from_str(ron_data).expect("RON parse must succeed");

        assert!(
            (cfg.overlay_color.a - 0.92).abs() < 1e-6,
            "alpha must be 0.92"
        );
        assert!((cfg.heading_color.r - 1.0).abs() < 1e-6);
        assert!((cfg.card_normal.r - 0.12).abs() < 1e-6);
        assert!((cfg.card_hover.r - 0.22).abs() < 1e-6);
        assert!((cfg.card_pressed.r - 0.08).abs() < 1e-6);
        assert!((cfg.subtitle_color.r - 0.85).abs() < 1e-6);
        assert_eq!(cfg.card_width, 260.0);
        assert_eq!(cfg.card_height, 320.0);
        assert_eq!(cfg.card_gap, 30.0);
    }

    #[test]
    fn default_card_dimensions_are_positive() {
        assert!(DEFAULT_CARD_WIDTH > 0.0);
        assert!(DEFAULT_CARD_HEIGHT > 0.0);
        assert!(DEFAULT_CARD_GAP > 0.0);
    }
}
