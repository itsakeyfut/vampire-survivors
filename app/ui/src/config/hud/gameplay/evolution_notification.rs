//! Evolution notification HUD configuration.
//!
//! Loaded from `assets/config/ui/hud/gameplay/evolution_notification.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

use crate::config::SrgbColor;

// ---------------------------------------------------------------------------
// Fallback constants (used while evolution_notification.ron is still loading)
// ---------------------------------------------------------------------------

const DEFAULT_DISPLAY_DURATION: f32 = 3.0;
const DEFAULT_FADE_START: f32 = 1.5;
const DEFAULT_FONT_SIZE: f32 = 40.0;
const DEFAULT_TOP_PERCENT: f32 = 38.0;
const DEFAULT_TEXT_COLOR: Color = Color::srgb(1.0, 0.85, 0.2);

/// Deserialization mirror of [`EvolutionNotificationHudConfig`] — every field is `Option<T>` so
/// RON files with missing fields still load and emit a `warn!` instead of failing.
#[derive(Deserialize, Default)]
#[serde(default, rename = "EvolutionNotificationHudConfig")]
pub(crate) struct EvolutionNotificationHudConfigPartial {
    pub display_duration: Option<f32>,
    pub fade_start: Option<f32>,
    pub font_size: Option<f32>,
    pub top_percent: Option<f32>,
    pub text_color: Option<SrgbColor>,
}

/// Evolution notification HUD config loaded from
/// `config/ui/hud/gameplay/evolution_notification.ron`.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct EvolutionNotificationHudConfig {
    /// Total display duration before the notification is despawned (seconds).
    pub display_duration: f32,
    /// Time after which the alpha fade-out begins (seconds).
    pub fade_start: f32,
    /// Font size of the notification text in points.
    pub font_size: f32,
    /// Vertical position as a percentage of the screen height.
    pub top_percent: f32,
    /// Notification text color.
    pub text_color: SrgbColor,
}

impl From<EvolutionNotificationHudConfigPartial> for EvolutionNotificationHudConfig {
    fn from(p: EvolutionNotificationHudConfigPartial) -> Self {
        EvolutionNotificationHudConfig {
            display_duration: p.display_duration.unwrap_or_else(|| {
                warn!(
                    "evolution_notification.ron: `display_duration` missing → using default {DEFAULT_DISPLAY_DURATION}"
                );
                DEFAULT_DISPLAY_DURATION
            }),
            fade_start: p.fade_start.unwrap_or_else(|| {
                warn!(
                    "evolution_notification.ron: `fade_start` missing → using default {DEFAULT_FADE_START}"
                );
                DEFAULT_FADE_START
            }),
            font_size: p.font_size.unwrap_or_else(|| {
                warn!(
                    "evolution_notification.ron: `font_size` missing → using default {DEFAULT_FONT_SIZE}"
                );
                DEFAULT_FONT_SIZE
            }),
            top_percent: p.top_percent.unwrap_or_else(|| {
                warn!(
                    "evolution_notification.ron: `top_percent` missing → using default {DEFAULT_TOP_PERCENT}"
                );
                DEFAULT_TOP_PERCENT
            }),
            text_color: p.text_color.unwrap_or_else(|| {
                warn!("evolution_notification.ron: `text_color` missing → using default");
                SrgbColor { r: 1.0, g: 0.85, b: 0.2 }
            }),
        }
    }
}

/// Resource holding the handle to the loaded [`EvolutionNotificationHudConfig`].
#[derive(Resource)]
pub struct EvolutionNotificationHudConfigHandle(pub Handle<EvolutionNotificationHudConfig>);

/// SystemParam for accessing [`EvolutionNotificationHudConfig`].
///
/// Returns `None` while the asset is loading or the plugin is absent.
#[derive(SystemParam)]
pub struct EvolutionNotificationHudParams<'w> {
    handle: Option<Res<'w, EvolutionNotificationHudConfigHandle>>,
    assets: Option<Res<'w, Assets<EvolutionNotificationHudConfig>>>,
}

impl<'w> EvolutionNotificationHudParams<'w> {
    /// Returns the currently loaded config, or `None`.
    pub fn get(&self) -> Option<&EvolutionNotificationHudConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }

    pub fn display_duration(&self) -> f32 {
        self.get()
            .map(|c| c.display_duration)
            .unwrap_or(DEFAULT_DISPLAY_DURATION)
    }

    pub fn fade_start(&self) -> f32 {
        self.get()
            .map(|c| c.fade_start)
            .unwrap_or(DEFAULT_FADE_START)
    }

    pub fn font_size(&self) -> f32 {
        self.get().map(|c| c.font_size).unwrap_or(DEFAULT_FONT_SIZE)
    }

    pub fn top_percent(&self) -> f32 {
        self.get()
            .map(|c| c.top_percent)
            .unwrap_or(DEFAULT_TOP_PERCENT)
    }

    pub fn text_color(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.text_color))
            .unwrap_or(DEFAULT_TEXT_COLOR)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const RON: &str = r#"
EvolutionNotificationHudConfig(
    display_duration: 3.0,
    fade_start:       1.5,
    font_size:        40.0,
    top_percent:      38.0,
    text_color:       (r: 1.0, g: 0.85, b: 0.2),
)
"#;

    #[test]
    fn evolution_notification_hud_config_deserialization() {
        let partial: EvolutionNotificationHudConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(RON)
            .expect("RON parse must succeed");
        let cfg = EvolutionNotificationHudConfig::from(partial);
        assert_eq!(cfg.display_duration, 3.0);
        assert_eq!(cfg.fade_start, 1.5);
        assert_eq!(cfg.font_size, 40.0);
        assert_eq!(cfg.top_percent, 38.0);
        assert!((cfg.text_color.r - 1.0).abs() < 1e-6);
        assert!((cfg.text_color.g - 0.85).abs() < 1e-6);
        assert!((cfg.text_color.b - 0.2).abs() < 1e-6);
    }

    #[test]
    fn display_duration_is_positive() {
        let partial: EvolutionNotificationHudConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(RON)
            .unwrap();
        let cfg = EvolutionNotificationHudConfig::from(partial);
        assert!(cfg.display_duration > 0.0);
    }

    #[test]
    fn fade_start_is_before_display_duration() {
        let partial: EvolutionNotificationHudConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(RON)
            .unwrap();
        let cfg = EvolutionNotificationHudConfig::from(partial);
        assert!(cfg.fade_start < cfg.display_duration);
    }
}
