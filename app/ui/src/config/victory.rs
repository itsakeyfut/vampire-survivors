//! Victory screen configuration.
//!
//! Loaded from `assets/config/ui/screen/victory.ron`.
//! Systems read the current values via [`VictoryScreenParams`] and fall back
//! to private `DEFAULT_*` constants defined in each consumer module.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

use super::SrgbColor;

// ---------------------------------------------------------------------------
// Fallback constants (used while victory.ron is still loading)
// ---------------------------------------------------------------------------

const DEFAULT_VICTORY_COLOR: Color = Color::srgb(1.0, 0.85, 0.10);
const DEFAULT_STAT_COLOR: Color = Color::srgb(0.85, 0.85, 0.85);
const DEFAULT_STAT_FONT_SIZE: f32 = 24.0;
const DEFAULT_STATS_MARGIN_TOP: f32 = 16.0;
const DEFAULT_BUTTON_MARGIN_TOP: f32 = 48.0;
const DEFAULT_ROW_GAP: f32 = 8.0;

// ---------------------------------------------------------------------------
// Config asset
// ---------------------------------------------------------------------------

/// Victory screen style config loaded from `config/ui/screen/victory.ron`.
///
/// Controls the visual appearance of the victory screen: heading color, stat
/// text color and size, and spacing between sections.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct VictoryScreenConfig {
    /// Color of the "YOU WIN!" heading text.
    pub victory_color: SrgbColor,
    /// Color of the run-statistics text lines.
    pub stat_color: SrgbColor,
    /// Font size of the run-statistics text lines.
    pub stat_font_size: f32,
    /// Top margin between the heading and the stats container (pixels).
    pub stats_margin_top: f32,
    /// Top margin between the stats container and the title button (pixels).
    pub button_margin_top: f32,
    /// Vertical gap between individual stat lines (pixels).
    pub row_gap: f32,
}

/// Resource holding the handle to the loaded [`VictoryScreenConfig`].
#[derive(Resource)]
pub struct VictoryScreenConfigHandle(pub Handle<VictoryScreenConfig>);

// ---------------------------------------------------------------------------
// SystemParam bundle
// ---------------------------------------------------------------------------

/// SystemParam bundle for accessing [`VictoryScreenConfig`].
///
/// Returns `None` while the asset is still loading or when
/// [`super::UiConfigPlugin`] has not been registered (e.g. in unit tests).
/// Call `.get()` to obtain `Option<&VictoryScreenConfig>`.
#[derive(SystemParam)]
pub struct VictoryScreenParams<'w> {
    handle: Option<Res<'w, VictoryScreenConfigHandle>>,
    assets: Option<Res<'w, Assets<VictoryScreenConfig>>>,
}

impl<'w> VictoryScreenParams<'w> {
    /// Returns the currently loaded [`VictoryScreenConfig`], or `None`.
    pub fn get(&self) -> Option<&VictoryScreenConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }

    pub fn victory_color(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.victory_color))
            .unwrap_or(DEFAULT_VICTORY_COLOR)
    }

    pub fn stat_color(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.stat_color))
            .unwrap_or(DEFAULT_STAT_COLOR)
    }

    pub fn stat_font_size(&self) -> f32 {
        self.get()
            .map(|c| c.stat_font_size)
            .unwrap_or(DEFAULT_STAT_FONT_SIZE)
    }

    pub fn stats_margin_top(&self) -> f32 {
        self.get()
            .map(|c| c.stats_margin_top)
            .unwrap_or(DEFAULT_STATS_MARGIN_TOP)
    }

    pub fn button_margin_top(&self) -> f32 {
        self.get()
            .map(|c| c.button_margin_top)
            .unwrap_or(DEFAULT_BUTTON_MARGIN_TOP)
    }

    pub fn row_gap(&self) -> f32 {
        self.get().map(|c| c.row_gap).unwrap_or(DEFAULT_ROW_GAP)
    }
}

// ---------------------------------------------------------------------------
// Hot-reload system
// ---------------------------------------------------------------------------

/// Logs when `config/ui/screen/victory.ron` is loaded or hot-reloaded.
///
/// Because the victory screen is transient (spawned on enter, despawned on
/// exit), live entity updates are not required — the next time the screen
/// opens it will read the new values via [`VictoryScreenParams`].
pub fn hot_reload_victory_screen(mut events: MessageReader<AssetEvent<VictoryScreenConfig>>) {
    for event in events.read() {
        match event {
            AssetEvent::Added { .. } => {
                info!("✅ Victory screen config loaded");
            }
            AssetEvent::Modified { .. } => {
                info!("🔥 Victory screen config hot-reloaded");
            }
            AssetEvent::Removed { .. } => {
                warn!("⚠️ Victory screen config removed");
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
    fn victory_screen_config_deserialization() {
        let ron_data = r#"
VictoryScreenConfig(
    victory_color:    (r: 1.0,  g: 0.85, b: 0.10),
    stat_color:       (r: 0.85, g: 0.85, b: 0.85),
    stat_font_size:   24.0,
    stats_margin_top: 16.0,
    button_margin_top: 48.0,
    row_gap:          8.0,
)
"#;
        let cfg: VictoryScreenConfig = ron::de::from_str(ron_data).expect("RON parse must succeed");

        assert!((cfg.victory_color.r - 1.0).abs() < 1e-6);
        assert!((cfg.victory_color.g - 0.85).abs() < 1e-6);
        assert_eq!(cfg.stat_font_size, 24.0);
        assert_eq!(cfg.stats_margin_top, 16.0);
        assert_eq!(cfg.button_margin_top, 48.0);
        assert_eq!(cfg.row_gap, 8.0);
    }

    #[test]
    fn victory_screen_config_values_are_positive() {
        let cfg = VictoryScreenConfig {
            victory_color: SrgbColor {
                r: 1.0,
                g: 0.85,
                b: 0.1,
            },
            stat_color: SrgbColor {
                r: 0.85,
                g: 0.85,
                b: 0.85,
            },
            stat_font_size: 24.0,
            stats_margin_top: 16.0,
            button_margin_top: 48.0,
            row_gap: 8.0,
        };
        assert!(cfg.stat_font_size > 0.0);
        assert!(cfg.stats_margin_top >= 0.0);
        assert!(cfg.button_margin_top >= 0.0);
        assert!(cfg.row_gap >= 0.0);
    }
}
