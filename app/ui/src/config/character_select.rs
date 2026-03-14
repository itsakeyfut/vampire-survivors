//! Character-select screen configuration.
//!
//! Loaded from `assets/config/ui/screen/character_select.ron`.
//! Systems read the current values via [`CharacterSelectScreenParams`] and fall
//! back to private `DEFAULT_*` constants defined in each consumer module.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

use super::SrgbColor;

// ---------------------------------------------------------------------------
// Fallback constants (used while character_select.ron is still loading)
// ---------------------------------------------------------------------------

const DEFAULT_CARD_WIDTH: f32 = 160.0;
const DEFAULT_CARD_HEIGHT: f32 = 140.0;
const DEFAULT_CARD_GAP: f32 = 16.0;
const DEFAULT_CARD_NAME_FONT_SIZE: f32 = 18.0;
const DEFAULT_CARD_COLOR_UNLOCKED: Color = Color::srgb(0.133, 0.200, 0.400);
const DEFAULT_CARD_COLOR_SELECTED: Color = Color::srgb(0.300, 0.500, 0.900);
const DEFAULT_CARD_COLOR_HOVER: Color = Color::srgb(0.200, 0.350, 0.650);
const DEFAULT_CARD_COLOR_PRESSED: Color = Color::srgb(0.086, 0.133, 0.267);
const DEFAULT_CARD_COLOR_LOCKED: Color = Color::srgb(0.100, 0.100, 0.150);
const DEFAULT_CARD_COLOR_LOCKED_HOVER: Color = Color::srgb(0.130, 0.130, 0.180);
const DEFAULT_CARD_TEXT_COLOR: Color = Color::srgb(1.000, 1.000, 1.000);
const DEFAULT_CARD_TEXT_LOCKED_COLOR: Color = Color::srgb(0.500, 0.500, 0.550);
const DEFAULT_DETAIL_BG_COLOR: Color = Color::srgb(0.080, 0.040, 0.160);
const DEFAULT_DETAIL_TEXT_COLOR: Color = Color::srgb(0.900, 0.900, 0.900);
const DEFAULT_DETAIL_LOCKED_COLOR: Color = Color::srgb(0.500, 0.500, 0.550);
const DEFAULT_DETAIL_FONT_SIZE: f32 = 20.0;
const DEFAULT_DETAIL_PANEL_WIDTH: f32 = 580.0;

// ---------------------------------------------------------------------------
// Config asset
// ---------------------------------------------------------------------------

/// Deserialization mirror of [`CharacterSelectScreenConfig`] — every field is `Option<T>` so
/// RON files with missing fields still load and emit a `warn!` instead of failing.
#[derive(Deserialize, Default)]
#[serde(default, rename = "CharacterSelectScreenConfig")]
pub(super) struct CharacterSelectScreenConfigPartial {
    pub card_width: Option<f32>,
    pub card_height: Option<f32>,
    pub card_gap: Option<f32>,
    pub card_name_font_size: Option<f32>,
    pub card_color_unlocked: Option<SrgbColor>,
    pub card_color_selected: Option<SrgbColor>,
    pub card_color_hover: Option<SrgbColor>,
    pub card_color_pressed: Option<SrgbColor>,
    pub card_color_locked: Option<SrgbColor>,
    pub card_color_locked_hover: Option<SrgbColor>,
    pub card_text_color: Option<SrgbColor>,
    pub card_text_locked_color: Option<SrgbColor>,
    pub detail_bg_color: Option<SrgbColor>,
    pub detail_text_color: Option<SrgbColor>,
    pub detail_locked_color: Option<SrgbColor>,
    pub detail_font_size: Option<f32>,
    pub detail_panel_width: Option<f32>,
}

/// Character-select screen style config loaded from
/// `config/ui/screen/character_select.ron`.
///
/// Controls card dimensions, card and detail panel colors, and text sizes for
/// the character-select screen.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct CharacterSelectScreenConfig {
    /// Width of each character card in pixels.
    pub card_width: f32,
    /// Height of each character card in pixels.
    pub card_height: f32,
    /// Horizontal gap between cards in pixels.
    pub card_gap: f32,
    /// Font size for the character name label inside each card.
    pub card_name_font_size: f32,
    /// Card background color when unlocked and not selected.
    pub card_color_unlocked: SrgbColor,
    /// Card background color when unlocked and selected.
    pub card_color_selected: SrgbColor,
    /// Card background color on hover (unlocked).
    pub card_color_hover: SrgbColor,
    /// Card background color when pressed.
    pub card_color_pressed: SrgbColor,
    /// Card background color for locked characters.
    pub card_color_locked: SrgbColor,
    /// Card background color on hover (locked).
    pub card_color_locked_hover: SrgbColor,
    /// Card name text color for unlocked characters.
    pub card_text_color: SrgbColor,
    /// Card name text color for locked characters.
    pub card_text_locked_color: SrgbColor,
    /// Detail panel background color.
    pub detail_bg_color: SrgbColor,
    /// Detail panel text color for unlocked characters.
    pub detail_text_color: SrgbColor,
    /// Detail panel text color when a locked character is shown.
    pub detail_locked_color: SrgbColor,
    /// Font size for the detail panel text.
    pub detail_font_size: f32,
    /// Width of the detail panel in pixels.
    pub detail_panel_width: f32,
}

impl From<CharacterSelectScreenConfigPartial> for CharacterSelectScreenConfig {
    fn from(p: CharacterSelectScreenConfigPartial) -> Self {
        CharacterSelectScreenConfig {
            card_width: p.card_width.unwrap_or_else(|| {
                warn!(
                    "character_select.ron: `card_width` missing → using default {DEFAULT_CARD_WIDTH}"
                );
                DEFAULT_CARD_WIDTH
            }),
            card_height: p.card_height.unwrap_or_else(|| {
                warn!(
                    "character_select.ron: `card_height` missing → using default {DEFAULT_CARD_HEIGHT}"
                );
                DEFAULT_CARD_HEIGHT
            }),
            card_gap: p.card_gap.unwrap_or_else(|| {
                warn!(
                    "character_select.ron: `card_gap` missing → using default {DEFAULT_CARD_GAP}"
                );
                DEFAULT_CARD_GAP
            }),
            card_name_font_size: p.card_name_font_size.unwrap_or_else(|| {
                warn!(
                    "character_select.ron: `card_name_font_size` missing → using default {DEFAULT_CARD_NAME_FONT_SIZE}"
                );
                DEFAULT_CARD_NAME_FONT_SIZE
            }),
            card_color_unlocked: p.card_color_unlocked.unwrap_or_else(|| {
                warn!("character_select.ron: `card_color_unlocked` missing → using default");
                SrgbColor { r: 0.133, g: 0.200, b: 0.400 }
            }),
            card_color_selected: p.card_color_selected.unwrap_or_else(|| {
                warn!("character_select.ron: `card_color_selected` missing → using default");
                SrgbColor { r: 0.300, g: 0.500, b: 0.900 }
            }),
            card_color_hover: p.card_color_hover.unwrap_or_else(|| {
                warn!("character_select.ron: `card_color_hover` missing → using default");
                SrgbColor { r: 0.200, g: 0.350, b: 0.650 }
            }),
            card_color_pressed: p.card_color_pressed.unwrap_or_else(|| {
                warn!("character_select.ron: `card_color_pressed` missing → using default");
                SrgbColor { r: 0.086, g: 0.133, b: 0.267 }
            }),
            card_color_locked: p.card_color_locked.unwrap_or_else(|| {
                warn!("character_select.ron: `card_color_locked` missing → using default");
                SrgbColor { r: 0.100, g: 0.100, b: 0.150 }
            }),
            card_color_locked_hover: p.card_color_locked_hover.unwrap_or_else(|| {
                warn!("character_select.ron: `card_color_locked_hover` missing → using default");
                SrgbColor { r: 0.130, g: 0.130, b: 0.180 }
            }),
            card_text_color: p.card_text_color.unwrap_or_else(|| {
                warn!("character_select.ron: `card_text_color` missing → using default");
                SrgbColor { r: 1.000, g: 1.000, b: 1.000 }
            }),
            card_text_locked_color: p.card_text_locked_color.unwrap_or_else(|| {
                warn!("character_select.ron: `card_text_locked_color` missing → using default");
                SrgbColor { r: 0.500, g: 0.500, b: 0.550 }
            }),
            detail_bg_color: p.detail_bg_color.unwrap_or_else(|| {
                warn!("character_select.ron: `detail_bg_color` missing → using default");
                SrgbColor { r: 0.080, g: 0.040, b: 0.160 }
            }),
            detail_text_color: p.detail_text_color.unwrap_or_else(|| {
                warn!("character_select.ron: `detail_text_color` missing → using default");
                SrgbColor { r: 0.900, g: 0.900, b: 0.900 }
            }),
            detail_locked_color: p.detail_locked_color.unwrap_or_else(|| {
                warn!("character_select.ron: `detail_locked_color` missing → using default");
                SrgbColor { r: 0.500, g: 0.500, b: 0.550 }
            }),
            detail_font_size: p.detail_font_size.unwrap_or_else(|| {
                warn!(
                    "character_select.ron: `detail_font_size` missing → using default {DEFAULT_DETAIL_FONT_SIZE}"
                );
                DEFAULT_DETAIL_FONT_SIZE
            }),
            detail_panel_width: p.detail_panel_width.unwrap_or_else(|| {
                warn!(
                    "character_select.ron: `detail_panel_width` missing → using default {DEFAULT_DETAIL_PANEL_WIDTH}"
                );
                DEFAULT_DETAIL_PANEL_WIDTH
            }),
        }
    }
}

/// Resource holding the handle to the loaded [`CharacterSelectScreenConfig`].
#[derive(Resource)]
pub struct CharacterSelectScreenConfigHandle(pub Handle<CharacterSelectScreenConfig>);

// ---------------------------------------------------------------------------
// SystemParam bundle
// ---------------------------------------------------------------------------

/// SystemParam bundle for accessing [`CharacterSelectScreenConfig`].
///
/// Returns `None` while the asset is still loading or when
/// [`super::UiConfigPlugin`] has not been registered (e.g. in unit tests).
/// Call `.get()` to obtain `Option<&CharacterSelectScreenConfig>`.
#[derive(SystemParam)]
pub struct CharacterSelectScreenParams<'w> {
    handle: Option<Res<'w, CharacterSelectScreenConfigHandle>>,
    assets: Option<Res<'w, Assets<CharacterSelectScreenConfig>>>,
}

impl<'w> CharacterSelectScreenParams<'w> {
    /// Returns the currently loaded [`CharacterSelectScreenConfig`], or `None`.
    pub fn get(&self) -> Option<&CharacterSelectScreenConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }

    pub fn card_width(&self) -> f32 {
        self.get()
            .map(|c| c.card_width)
            .unwrap_or(DEFAULT_CARD_WIDTH)
    }

    pub fn card_height(&self) -> f32 {
        self.get()
            .map(|c| c.card_height)
            .unwrap_or(DEFAULT_CARD_HEIGHT)
    }

    pub fn card_gap(&self) -> f32 {
        self.get().map(|c| c.card_gap).unwrap_or(DEFAULT_CARD_GAP)
    }

    pub fn card_name_font_size(&self) -> f32 {
        self.get()
            .map(|c| c.card_name_font_size)
            .unwrap_or(DEFAULT_CARD_NAME_FONT_SIZE)
    }

    pub fn card_color_unlocked(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.card_color_unlocked))
            .unwrap_or(DEFAULT_CARD_COLOR_UNLOCKED)
    }

    pub fn card_color_selected(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.card_color_selected))
            .unwrap_or(DEFAULT_CARD_COLOR_SELECTED)
    }

    pub fn card_color_hover(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.card_color_hover))
            .unwrap_or(DEFAULT_CARD_COLOR_HOVER)
    }

    pub fn card_color_pressed(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.card_color_pressed))
            .unwrap_or(DEFAULT_CARD_COLOR_PRESSED)
    }

    pub fn card_color_locked(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.card_color_locked))
            .unwrap_or(DEFAULT_CARD_COLOR_LOCKED)
    }

    pub fn card_color_locked_hover(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.card_color_locked_hover))
            .unwrap_or(DEFAULT_CARD_COLOR_LOCKED_HOVER)
    }

    pub fn card_text_color(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.card_text_color))
            .unwrap_or(DEFAULT_CARD_TEXT_COLOR)
    }

    pub fn card_text_locked_color(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.card_text_locked_color))
            .unwrap_or(DEFAULT_CARD_TEXT_LOCKED_COLOR)
    }

    pub fn detail_bg_color(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.detail_bg_color))
            .unwrap_or(DEFAULT_DETAIL_BG_COLOR)
    }

    pub fn detail_text_color(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.detail_text_color))
            .unwrap_or(DEFAULT_DETAIL_TEXT_COLOR)
    }

    pub fn detail_locked_color(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.detail_locked_color))
            .unwrap_or(DEFAULT_DETAIL_LOCKED_COLOR)
    }

    pub fn detail_font_size(&self) -> f32 {
        self.get()
            .map(|c| c.detail_font_size)
            .unwrap_or(DEFAULT_DETAIL_FONT_SIZE)
    }

    pub fn detail_panel_width(&self) -> f32 {
        self.get()
            .map(|c| c.detail_panel_width)
            .unwrap_or(DEFAULT_DETAIL_PANEL_WIDTH)
    }
}

// ---------------------------------------------------------------------------
// Hot-reload system
// ---------------------------------------------------------------------------

/// Logs when `config/ui/screen/character_select.ron` is loaded or hot-reloaded.
///
/// Because the character-select screen is transient (spawned on enter, despawned
/// on exit), live entity updates are not required — the next time the screen
/// opens it will read the new values via [`CharacterSelectScreenParams`].
pub fn hot_reload_character_select_screen(
    mut events: MessageReader<AssetEvent<CharacterSelectScreenConfig>>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Added { .. } => {
                info!("✅ Character select screen config loaded");
            }
            AssetEvent::Modified { .. } => {
                info!("🔥 Character select screen config hot-reloaded");
            }
            AssetEvent::Removed { .. } => {
                warn!("⚠️ Character select screen config removed");
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
    fn character_select_screen_config_deserialization() {
        let ron_data = r#"
CharacterSelectScreenConfig(
    card_width:              160.0,
    card_height:             140.0,
    card_gap:                16.0,
    card_name_font_size:     18.0,
    card_color_unlocked:     (r: 0.133, g: 0.200, b: 0.400),
    card_color_selected:     (r: 0.300, g: 0.500, b: 0.900),
    card_color_hover:        (r: 0.200, g: 0.350, b: 0.650),
    card_color_pressed:      (r: 0.086, g: 0.133, b: 0.267),
    card_color_locked:       (r: 0.100, g: 0.100, b: 0.150),
    card_color_locked_hover: (r: 0.130, g: 0.130, b: 0.180),
    card_text_color:         (r: 1.000, g: 1.000, b: 1.000),
    card_text_locked_color:  (r: 0.500, g: 0.500, b: 0.550),
    detail_bg_color:         (r: 0.080, g: 0.040, b: 0.160),
    detail_text_color:       (r: 0.900, g: 0.900, b: 0.900),
    detail_locked_color:     (r: 0.500, g: 0.500, b: 0.550),
    detail_font_size:        20.0,
    detail_panel_width:      580.0,
)
"#;
        let partial: CharacterSelectScreenConfigPartial =
            ron::Options::default().with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME).from_str(ron_data).expect("RON parse must succeed");
        let cfg = CharacterSelectScreenConfig::from(partial);

        assert_eq!(cfg.card_width, 160.0);
        assert_eq!(cfg.card_height, 140.0);
        assert_eq!(cfg.card_gap, 16.0);
        assert_eq!(cfg.card_name_font_size, 18.0);
        assert!((cfg.card_color_selected.r - 0.300).abs() < 1e-6);
        assert_eq!(cfg.detail_font_size, 20.0);
        assert_eq!(cfg.detail_panel_width, 580.0);
    }

    #[test]
    fn character_select_screen_config_dimensions_are_positive() {
        let cfg = CharacterSelectScreenConfig {
            card_width: 160.0,
            card_height: 140.0,
            card_gap: 16.0,
            card_name_font_size: 18.0,
            card_color_unlocked: SrgbColor {
                r: 0.133,
                g: 0.200,
                b: 0.400,
            },
            card_color_selected: SrgbColor {
                r: 0.300,
                g: 0.500,
                b: 0.900,
            },
            card_color_hover: SrgbColor {
                r: 0.200,
                g: 0.350,
                b: 0.650,
            },
            card_color_pressed: SrgbColor {
                r: 0.086,
                g: 0.133,
                b: 0.267,
            },
            card_color_locked: SrgbColor {
                r: 0.100,
                g: 0.100,
                b: 0.150,
            },
            card_color_locked_hover: SrgbColor {
                r: 0.130,
                g: 0.130,
                b: 0.180,
            },
            card_text_color: SrgbColor {
                r: 1.0,
                g: 1.0,
                b: 1.0,
            },
            card_text_locked_color: SrgbColor {
                r: 0.5,
                g: 0.5,
                b: 0.55,
            },
            detail_bg_color: SrgbColor {
                r: 0.08,
                g: 0.04,
                b: 0.16,
            },
            detail_text_color: SrgbColor {
                r: 0.90,
                g: 0.90,
                b: 0.90,
            },
            detail_locked_color: SrgbColor {
                r: 0.50,
                g: 0.50,
                b: 0.55,
            },
            detail_font_size: 20.0,
            detail_panel_width: 580.0,
        };
        assert!(cfg.card_width > 0.0);
        assert!(cfg.card_height > 0.0);
        assert!(cfg.card_gap > 0.0);
        assert!(cfg.card_name_font_size > 0.0);
        assert!(cfg.detail_font_size > 0.0);
        assert!(cfg.detail_panel_width > 0.0);
    }
}
