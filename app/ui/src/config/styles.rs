//! UI style configuration — colors, font sizes, and button dimensions.
//!
//! Loaded from `assets/config/ui/styles.ron`.  Systems read the current
//! values via [`UiStyleParams`] and fall back to the `DEFAULT_*` constants
//! in [`crate::styles`] when the config is not yet loaded.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// SrgbColor — RON-deserialisable colour triplet
// ---------------------------------------------------------------------------

/// Linear sRGB colour triplet for RON serialisation.
///
/// Bevy's [`Color`] does not implement [`serde::Deserialize`], so we parse
/// colour values via this thin wrapper and convert with
/// [`From<SrgbColor> for Color`].
#[derive(Deserialize, Debug, Clone)]
pub struct SrgbColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl From<SrgbColor> for Color {
    fn from(c: SrgbColor) -> Self {
        Color::srgb(c.r, c.g, c.b)
    }
}

impl From<&SrgbColor> for Color {
    fn from(c: &SrgbColor) -> Self {
        Color::srgb(c.r, c.g, c.b)
    }
}

// ---------------------------------------------------------------------------
// SrgbaColor — RON-deserialisable colour with alpha
// ---------------------------------------------------------------------------

/// Linear sRGB colour with alpha channel for RON serialisation.
///
/// Used for semi-transparent UI overlays where [`SrgbColor`] (opaque) is not
/// sufficient.
#[derive(Deserialize, Debug, Clone)]
pub struct SrgbaColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl From<SrgbaColor> for Color {
    fn from(c: SrgbaColor) -> Self {
        Color::srgba(c.r, c.g, c.b, c.a)
    }
}

impl From<&SrgbaColor> for Color {
    fn from(c: &SrgbaColor) -> Self {
        Color::srgba(c.r, c.g, c.b, c.a)
    }
}

// ---------------------------------------------------------------------------
// Fallback constants (used while styles.ron is still loading)
// ---------------------------------------------------------------------------

const DEFAULT_BG_COLOR: Color = Color::srgb(0.05, 0.05, 0.08);
const DEFAULT_TITLE_COLOR: Color = Color::srgb(0.85, 0.15, 0.15);

// ---------------------------------------------------------------------------
// UiStyleConfig
// ---------------------------------------------------------------------------

/// Deserialization mirror of [`UiStyleConfig`] — every field is `Option<T>` so
/// RON files with missing fields still load and emit a `warn!` instead of failing.
#[derive(Deserialize, Default)]
#[serde(default, rename = "UiStyleConfig")]
pub(super) struct UiStyleConfigPartial {
    pub bg_color: Option<SrgbColor>,
    pub title_color: Option<SrgbColor>,
}

/// UI style configuration loaded from `config/ui/styles.ron`.
///
/// Covers screen-level appearance that is shared across states: background
/// color and title text color.  Per-widget styles (buttons, headings, cards)
/// live in the HUD config files under `config/ui/hud/`.
/// Systems that read via [`UiStyleParams`] pick up hot-reloaded values
/// automatically on the next frame.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct UiStyleConfig {
    /// Background color for full-screen overlays (title, game-over, etc.).
    pub bg_color: SrgbColor,
    /// Color of the game title heading on the title screen.
    pub title_color: SrgbColor,
}

impl From<UiStyleConfigPartial> for UiStyleConfig {
    fn from(p: UiStyleConfigPartial) -> Self {
        UiStyleConfig {
            bg_color: p.bg_color.unwrap_or_else(|| {
                warn!("styles.ron: `bg_color` missing → using default");
                SrgbColor {
                    r: 0.05,
                    g: 0.05,
                    b: 0.08,
                }
            }),
            title_color: p.title_color.unwrap_or_else(|| {
                warn!("styles.ron: `title_color` missing → using default");
                SrgbColor {
                    r: 0.85,
                    g: 0.15,
                    b: 0.15,
                }
            }),
        }
    }
}

/// Resource holding the handle to the loaded [`UiStyleConfig`].
#[derive(Resource)]
pub struct UiStyleConfigHandle(pub Handle<UiStyleConfig>);

// ---------------------------------------------------------------------------
// SystemParam bundle
// ---------------------------------------------------------------------------

/// SystemParam bundle for accessing [`UiStyleConfig`].
///
/// Returns `None` while the asset is still loading or when
/// [`super::UiConfigPlugin`] has not been registered (e.g. in unit tests).
/// Call `.get()` to obtain `Option<&UiStyleConfig>`.
#[derive(SystemParam)]
pub struct UiStyleParams<'w> {
    handle: Option<Res<'w, UiStyleConfigHandle>>,
    assets: Option<Res<'w, Assets<UiStyleConfig>>>,
}

impl<'w> UiStyleParams<'w> {
    /// Returns the currently loaded [`UiStyleConfig`], or `None` while loading.
    pub fn get(&self) -> Option<&UiStyleConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }

    pub fn bg_color(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.bg_color))
            .unwrap_or(DEFAULT_BG_COLOR)
    }

    pub fn title_color(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.title_color))
            .unwrap_or(DEFAULT_TITLE_COLOR)
    }
}

// ---------------------------------------------------------------------------
// Marker components (hot-reload targets on the title screen)
// ---------------------------------------------------------------------------

/// Marker attached to the title-screen background [`Node`].
///
/// The hot-reload system queries this to update [`BackgroundColor`]
/// when `config/ui/styles.ron` changes.
#[derive(Component)]
pub struct TitleScreenBg;

/// Marker attached to the title-screen heading [`Text`].
///
/// The hot-reload system queries this to update [`TextColor`]
/// when `config/ui/styles.ron` changes.
#[derive(Component)]
pub struct TitleHeadingText;

// ---------------------------------------------------------------------------
// Hot-reload system
// ---------------------------------------------------------------------------

/// Updates live title-screen entities when `config/ui/styles.ron` changes.
///
/// Reacts to both [`AssetEvent::Added`] (first load) and
/// [`AssetEvent::Modified`] (hot-reload) so entities spawned before the
/// config finishes loading also receive the configured values.
///
/// Updates background color ([`TitleScreenBg`]) and title heading color
/// ([`TitleHeadingText`]).  Heading font size is handled by
/// [`crate::hud::screen_heading::hot_reload_screen_heading_hud`].
/// Button colors and dimensions are handled by
/// [`crate::hud::menu_button::hot_reload_menu_button_hud`].
pub fn hot_reload_ui_style(
    mut events: MessageReader<AssetEvent<UiStyleConfig>>,
    config_assets: Res<Assets<UiStyleConfig>>,
    config_handle: Option<Res<UiStyleConfigHandle>>,
    mut bg_q: Query<&mut BackgroundColor, With<TitleScreenBg>>,
    mut heading_q: Query<&mut TextColor, With<TitleHeadingText>>,
) {
    let Some(config_handle) = config_handle else {
        return;
    };

    let mut needs_apply = false;
    for event in events.read() {
        match event {
            AssetEvent::Added { .. } => {
                info!("✅ UI style config loaded");
                needs_apply = true;
            }
            AssetEvent::Modified { .. } => {
                info!("🔥 UI style config hot-reloaded");
                needs_apply = true;
            }
            AssetEvent::Removed { .. } => {
                warn!("⚠️ UI style config removed");
            }
            _ => {}
        }
    }

    if needs_apply && let Some(cfg) = config_assets.get(&config_handle.0) {
        if let Ok(mut bg) = bg_q.single_mut() {
            *bg = BackgroundColor(Color::from(&cfg.bg_color));
        }
        if let Ok(mut tc) = heading_q.single_mut() {
            *tc = TextColor(Color::from(&cfg.title_color));
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
    fn ui_style_config_deserialization() {
        let ron_data = r#"
UiStyleConfig(
    bg_color:    (r: 0.05, g: 0.05, b: 0.08),
    title_color: (r: 0.85, g: 0.15, b: 0.15),
)
"#;
        let partial: UiStyleConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(ron_data)
            .expect("RON parse must succeed");
        let cfg = UiStyleConfig::from(partial);
        assert!((cfg.bg_color.r - 0.05).abs() < 1e-6);
        assert!((cfg.title_color.r - 0.85).abs() < 1e-6);
    }

    #[test]
    fn srgb_color_converts_to_bevy_color() {
        let c = SrgbColor {
            r: 1.0,
            g: 0.5,
            b: 0.0,
        };
        let color = Color::from(c);
        let srgba = color.to_srgba();
        assert!((srgba.red - 1.0).abs() < 1e-6);
        assert!((srgba.green - 0.5).abs() < 1e-6);
        assert!((srgba.blue - 0.0).abs() < 1e-6);
    }

    #[test]
    fn srgba_color_converts_to_bevy_color_with_alpha() {
        let c = SrgbaColor {
            r: 0.02,
            g: 0.02,
            b: 0.06,
            a: 0.92,
        };
        let color = Color::from(&c);
        let srgba = color.to_srgba();
        assert!((srgba.red - 0.02).abs() < 1e-6);
        assert!((srgba.alpha - 0.92).abs() < 1e-6);
    }

    #[test]
    fn srgb_color_ref_converts_to_bevy_color() {
        let c = SrgbColor {
            r: 0.3,
            g: 0.6,
            b: 0.9,
        };
        let color = Color::from(&c);
        let srgba = color.to_srgba();
        assert!((srgba.red - 0.3).abs() < 1e-6);
    }
}
