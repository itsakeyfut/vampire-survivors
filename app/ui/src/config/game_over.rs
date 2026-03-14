//! Game-over screen configuration.
//!
//! Loaded from `assets/config/ui/screen/game_over.ron`.
//! Systems read the current values via [`GameOverScreenParams`] and fall back
//! to private `DEFAULT_*` constants defined in each consumer module.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

use super::SrgbColor;

// ---------------------------------------------------------------------------
// Fallback constants (used while game_over.ron is still loading)
// ---------------------------------------------------------------------------

const DEFAULT_HEADING_COLOR: Color = Color::srgb(0.8, 0.2, 0.2);
const DEFAULT_STAT_COLOR: Color = Color::srgb(0.85, 0.85, 0.85);
const DEFAULT_STAT_FONT_SIZE: f32 = 24.0;
const DEFAULT_STATS_MARGIN_TOP: f32 = 16.0;
const DEFAULT_BUTTON_MARGIN_TOP: f32 = 48.0;
const DEFAULT_ROW_GAP: f32 = 8.0;

// ---------------------------------------------------------------------------
// Config asset
// ---------------------------------------------------------------------------

/// Deserialization mirror of [`GameOverScreenConfig`] — every field is `Option<T>` so
/// RON files with missing fields still load and emit a `warn!` instead of failing.
#[derive(Deserialize, Default)]
#[serde(default, rename = "GameOverScreenConfig")]
pub(super) struct GameOverScreenConfigPartial {
    pub heading_color: Option<SrgbColor>,
    pub stat_color: Option<SrgbColor>,
    pub stat_font_size: Option<f32>,
    pub stats_margin_top: Option<f32>,
    pub button_margin_top: Option<f32>,
    pub row_gap: Option<f32>,
}

/// Game-over screen style config loaded from `config/ui/screen/game_over.ron`.
///
/// Controls the visual appearance of the game-over screen: heading color,
/// stat text color and size, and spacing between sections.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct GameOverScreenConfig {
    /// Color of the "GAME OVER" heading text.
    pub heading_color: SrgbColor,
    /// Color of the run-statistics text lines.
    pub stat_color: SrgbColor,
    /// Font size of the run-statistics text lines.
    pub stat_font_size: f32,
    /// Top margin between the heading and the stats container (pixels).
    pub stats_margin_top: f32,
    /// Top margin between the stats container and the buttons (pixels).
    pub button_margin_top: f32,
    /// Vertical gap between individual stat lines and between buttons (pixels).
    pub row_gap: f32,
}

impl From<GameOverScreenConfigPartial> for GameOverScreenConfig {
    fn from(p: GameOverScreenConfigPartial) -> Self {
        GameOverScreenConfig {
            heading_color: p.heading_color.unwrap_or_else(|| {
                warn!("game_over.ron: `heading_color` missing → using default");
                SrgbColor { r: 0.8, g: 0.2, b: 0.2 }
            }),
            stat_color: p.stat_color.unwrap_or_else(|| {
                warn!("game_over.ron: `stat_color` missing → using default");
                SrgbColor { r: 0.85, g: 0.85, b: 0.85 }
            }),
            stat_font_size: p.stat_font_size.unwrap_or_else(|| {
                warn!(
                    "game_over.ron: `stat_font_size` missing → using default {DEFAULT_STAT_FONT_SIZE}"
                );
                DEFAULT_STAT_FONT_SIZE
            }),
            stats_margin_top: p.stats_margin_top.unwrap_or_else(|| {
                warn!(
                    "game_over.ron: `stats_margin_top` missing → using default {DEFAULT_STATS_MARGIN_TOP}"
                );
                DEFAULT_STATS_MARGIN_TOP
            }),
            button_margin_top: p.button_margin_top.unwrap_or_else(|| {
                warn!(
                    "game_over.ron: `button_margin_top` missing → using default {DEFAULT_BUTTON_MARGIN_TOP}"
                );
                DEFAULT_BUTTON_MARGIN_TOP
            }),
            row_gap: p.row_gap.unwrap_or_else(|| {
                warn!("game_over.ron: `row_gap` missing → using default {DEFAULT_ROW_GAP}");
                DEFAULT_ROW_GAP
            }),
        }
    }
}

/// Resource holding the handle to the loaded [`GameOverScreenConfig`].
#[derive(Resource)]
pub struct GameOverScreenConfigHandle(pub Handle<GameOverScreenConfig>);

// ---------------------------------------------------------------------------
// SystemParam bundle
// ---------------------------------------------------------------------------

/// SystemParam bundle for accessing [`GameOverScreenConfig`].
///
/// Returns `None` while the asset is still loading or when
/// [`super::UiConfigPlugin`] has not been registered (e.g. in unit tests).
/// Call `.get()` to obtain `Option<&GameOverScreenConfig>`.
#[derive(SystemParam)]
pub struct GameOverScreenParams<'w> {
    handle: Option<Res<'w, GameOverScreenConfigHandle>>,
    assets: Option<Res<'w, Assets<GameOverScreenConfig>>>,
}

impl<'w> GameOverScreenParams<'w> {
    /// Returns the currently loaded [`GameOverScreenConfig`], or `None`.
    pub fn get(&self) -> Option<&GameOverScreenConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }

    pub fn heading_color(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.heading_color))
            .unwrap_or(DEFAULT_HEADING_COLOR)
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

/// Logs when `config/ui/screen/game_over.ron` is loaded or hot-reloaded.
///
/// Because the game-over screen is transient (spawned on enter, despawned on
/// exit), live entity updates are not required — the next time the screen
/// opens it will read the new values via [`GameOverScreenParams`].
pub fn hot_reload_game_over_screen(mut events: MessageReader<AssetEvent<GameOverScreenConfig>>) {
    for event in events.read() {
        match event {
            AssetEvent::Added { .. } => {
                info!("✅ Game-over screen config loaded");
            }
            AssetEvent::Modified { .. } => {
                info!("🔥 Game-over screen config hot-reloaded");
            }
            AssetEvent::Removed { .. } => {
                warn!("⚠️ Game-over screen config removed");
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
    fn game_over_screen_config_deserialization() {
        let ron_data = r#"
GameOverScreenConfig(
    heading_color:     (r: 0.8,  g: 0.2,  b: 0.2),
    stat_color:        (r: 0.85, g: 0.85, b: 0.85),
    stat_font_size:    24.0,
    stats_margin_top:  16.0,
    button_margin_top: 48.0,
    row_gap:           8.0,
)
"#;
        let partial: GameOverScreenConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(ron_data)
            .expect("RON parse must succeed");
        let cfg = GameOverScreenConfig::from(partial);

        assert!((cfg.heading_color.r - 0.8).abs() < 1e-6);
        assert!((cfg.stat_color.r - 0.85).abs() < 1e-6);
        assert_eq!(cfg.stat_font_size, 24.0);
        assert_eq!(cfg.stats_margin_top, 16.0);
        assert_eq!(cfg.button_margin_top, 48.0);
        assert_eq!(cfg.row_gap, 8.0);
    }

    #[test]
    fn game_over_screen_config_values_are_positive() {
        let cfg = GameOverScreenConfig {
            heading_color: SrgbColor {
                r: 0.8,
                g: 0.2,
                b: 0.2,
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
