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
// UiStyleConfig
// ---------------------------------------------------------------------------

/// UI style configuration loaded from `config/ui/styles.ron`.
///
/// Covers the full visual palette for all screens: colors, font sizes, and
/// button dimensions.  Systems that read via [`UiStyleParams`] pick up
/// hot-reloaded values automatically on the next frame.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct UiStyleConfig {
    // Colors
    pub bg_color: SrgbColor,
    pub title_color: SrgbColor,
    pub text_color: SrgbColor,
    pub button_normal: SrgbColor,
    pub button_hover: SrgbColor,
    pub button_pressed: SrgbColor,
    // Font sizes
    pub font_size_huge: f32,
    pub font_size_large: f32,
    pub font_size_medium: f32,
    pub font_size_small: f32,
    // Button dimensions
    pub button_large_width: f32,
    pub button_large_height: f32,
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

/// Marker attached to the Start button [`Node`].
///
/// The hot-reload system queries this to update [`BackgroundColor`]
/// (resting state) when `config/ui/styles.ron` changes.
#[derive(Component)]
pub struct TitleStartButton;

/// Marker attached to the Start button label [`Text`].
///
/// The hot-reload system queries this to update [`TextColor`]
/// when `config/ui/styles.ron` changes.
#[derive(Component)]
pub struct TitleButtonLabel;

// ---------------------------------------------------------------------------
// Hot-reload system
// ---------------------------------------------------------------------------

/// Updates live title-screen entities when `config/ui/styles.ron` changes.
///
/// Reacts to both [`AssetEvent::Added`] (first load) and
/// [`AssetEvent::Modified`] (hot-reload) so entities spawned before the
/// config finishes loading also receive the configured values.
/// Updates colors, font sizes, and button dimensions.
/// Button hover/press colors are handled frame-by-frame by
/// [`crate::components::handle_button_interaction`].
#[allow(clippy::too_many_arguments)]
pub fn hot_reload_ui_style(
    mut events: MessageReader<AssetEvent<UiStyleConfig>>,
    config_assets: Res<Assets<UiStyleConfig>>,
    config_handle: Option<Res<UiStyleConfigHandle>>,
    mut bg_q: Query<&mut BackgroundColor, With<TitleScreenBg>>,
    mut heading_q: Query<&mut TextColor, With<TitleHeadingText>>,
    mut btn_q: Query<&mut BackgroundColor, (With<TitleStartButton>, Without<TitleScreenBg>)>,
    mut btn_label_q: Query<&mut TextColor, (With<TitleButtonLabel>, Without<TitleHeadingText>)>,
    mut heading_font_q: Query<&mut TextFont, With<TitleHeadingText>>,
    mut btn_label_font_q: Query<&mut TextFont, (With<TitleButtonLabel>, Without<TitleHeadingText>)>,
    mut btn_node_q: Query<&mut Node, (With<TitleStartButton>, Without<TitleScreenBg>)>,
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
        if let Ok(mut font) = heading_font_q.single_mut() {
            font.font_size = cfg.font_size_huge;
        }
        if let Ok(mut bg) = btn_q.single_mut() {
            *bg = BackgroundColor(Color::from(&cfg.button_normal));
        }
        if let Ok(mut node) = btn_node_q.single_mut() {
            node.width = Val::Px(cfg.button_large_width);
            node.height = Val::Px(cfg.button_large_height);
        }
        if let Ok(mut tc) = btn_label_q.single_mut() {
            *tc = TextColor(Color::from(&cfg.text_color));
        }
        if let Ok(mut font) = btn_label_font_q.single_mut() {
            font.font_size = cfg.font_size_large;
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
    bg_color:      (r: 0.05, g: 0.05, b: 0.08),
    title_color:   (r: 0.85, g: 0.15, b: 0.15),
    text_color:    (r: 0.95, g: 0.90, b: 0.85),
    button_normal:  (r: 0.30, g: 0.05, b: 0.05),
    button_hover:   (r: 0.60, g: 0.10, b: 0.10),
    button_pressed: (r: 0.20, g: 0.02, b: 0.02),
    font_size_huge:   72.0,
    font_size_large:  48.0,
    font_size_medium: 32.0,
    font_size_small:  24.0,
    button_large_width:  280.0,
    button_large_height:  80.0,
)
"#;
        let cfg: UiStyleConfig = ron::de::from_str(ron_data).expect("RON parse must succeed");
        assert!((cfg.bg_color.r - 0.05).abs() < 1e-6);
        assert!((cfg.title_color.r - 0.85).abs() < 1e-6);
        assert!((cfg.text_color.r - 0.95).abs() < 1e-6);
        assert!((cfg.button_normal.r - 0.30).abs() < 1e-6);
        assert!((cfg.button_hover.r - 0.60).abs() < 1e-6);
        assert!((cfg.button_pressed.r - 0.20).abs() < 1e-6);
        assert_eq!(cfg.font_size_huge, 72.0);
        assert_eq!(cfg.font_size_large, 48.0);
        assert_eq!(cfg.font_size_medium, 32.0);
        assert_eq!(cfg.font_size_small, 24.0);
        assert_eq!(cfg.button_large_width, 280.0);
        assert_eq!(cfg.button_large_height, 80.0);
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

    #[test]
    fn font_sizes_are_ordered_in_full_config() {
        let ron_data = r#"
UiStyleConfig(
    bg_color: (r: 0.0, g: 0.0, b: 0.0),
    title_color: (r: 0.0, g: 0.0, b: 0.0),
    text_color: (r: 0.0, g: 0.0, b: 0.0),
    button_normal: (r: 0.0, g: 0.0, b: 0.0),
    button_hover: (r: 0.0, g: 0.0, b: 0.0),
    button_pressed: (r: 0.0, g: 0.0, b: 0.0),
    font_size_huge: 72.0,
    font_size_large: 48.0,
    font_size_medium: 32.0,
    font_size_small: 24.0,
    button_large_width: 280.0,
    button_large_height: 80.0,
)
"#;
        let cfg: UiStyleConfig = ron::de::from_str(ron_data).expect("RON parse must succeed");
        assert!(cfg.font_size_huge > cfg.font_size_large);
        assert!(cfg.font_size_large > cfg.font_size_medium);
        assert!(cfg.font_size_medium > cfg.font_size_small);
        assert!(cfg.font_size_small > 0.0);
    }
}
