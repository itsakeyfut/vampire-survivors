//! Large menu button HUD widget.
//!
//! Spawns a full-width button with a centered text label.
//! Dimensions, font size, and colors are read from [`MenuButtonHudConfig`].
//! The button triggers an [`crate::components::ButtonAction`] on click via
//! [`crate::components::handle_button_interaction`].

use bevy::prelude::*;

use crate::components::{ButtonAction, MenuButton};
use crate::config::hud::menu_button::{MenuButtonHudConfig, MenuButtonHudConfigHandle};
use crate::i18n::TranslatableText;

// ---------------------------------------------------------------------------
// Fallback constants
// ---------------------------------------------------------------------------

const DEFAULT_WIDTH: f32 = 280.0;
const DEFAULT_HEIGHT: f32 = 60.0;
const DEFAULT_FONT_SIZE: f32 = 32.0;
// Button colors per docs/04_ui_ux.md: #223366 normal, white text.
const DEFAULT_COLOR_NORMAL: Color = Color::srgb(0.133, 0.200, 0.400);
const DEFAULT_TEXT_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

/// Marker component on the button node entity.
///
/// Used by [`hot_reload_menu_button_hud`] and by
/// [`crate::components::handle_button_interaction`] to find live button entities.
#[derive(Component)]
pub struct LargeMenuButtonHud;

/// Marker component on the button label text entity.
///
/// Used by [`hot_reload_menu_button_hud`] to update label color and font size.
#[derive(Component)]
pub struct LargeMenuButtonLabelHud;

// ---------------------------------------------------------------------------
// Spawn function
// ---------------------------------------------------------------------------

/// Spawns a large menu button with a centered text label as a child of `parent`.
///
/// - `label`    — button label text (e.g. "Start", "Title").
/// - `action`   — the [`ButtonAction`] triggered when the button is clicked.
/// - `cfg`      — layout/color config; pass `menu_button_params.get()`. Falls back
///   to `DEFAULT_*` constants when the asset is not yet loaded.
/// - `i18n_key` — when `Some`, attaches [`TranslatableText`] to the label so
///   [`crate::i18n::update_translatable_texts`] updates it live on language change.
pub fn spawn_large_menu_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    action: ButtonAction,
    cfg: Option<&MenuButtonHudConfig>,
    font: Handle<Font>,
    i18n_key: Option<&'static str>,
) {
    let width = cfg.map(|c| c.width).unwrap_or(DEFAULT_WIDTH);
    let height = cfg.map(|c| c.height).unwrap_or(DEFAULT_HEIGHT);
    let font_size = cfg.map(|c| c.font_size).unwrap_or(DEFAULT_FONT_SIZE);
    let color_normal = cfg
        .map(|c| Color::from(&c.color_normal))
        .unwrap_or(DEFAULT_COLOR_NORMAL);
    let text_color = cfg
        .map(|c| Color::from(&c.text_color))
        .unwrap_or(DEFAULT_TEXT_COLOR);

    parent
        .spawn((
            Button,
            Node {
                // Auto-width so Japanese labels (wider than English) never overflow.
                // min_width enforces the configured minimum; padding adds side breathing room.
                width: Val::Auto,
                min_width: Val::Px(width),
                height: Val::Px(height),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(24.0)),
                ..default()
            },
            BackgroundColor(color_normal),
            MenuButton { action },
            LargeMenuButtonHud,
        ))
        .with_children(|btn| {
            let mut label_cmd = btn.spawn((
                Text::new(label),
                TextFont {
                    font,
                    font_size,
                    ..default()
                },
                TextColor(text_color),
                TextLayout::new_with_linebreak(LineBreak::NoWrap),
                LargeMenuButtonLabelHud,
            ));
            if let Some(key) = i18n_key {
                label_cmd.insert(TranslatableText(key));
            }
        });
}

// ---------------------------------------------------------------------------
// Hot-reload system
// ---------------------------------------------------------------------------

/// Updates colors and dimensions of all [`LargeMenuButtonHud`] and
/// [`LargeMenuButtonLabelHud`] entities when `config/ui/hud/menu_button.ron`
/// is loaded or modified.
pub fn hot_reload_menu_button_hud(
    mut events: MessageReader<AssetEvent<MenuButtonHudConfig>>,
    cfg_assets: Res<Assets<MenuButtonHudConfig>>,
    cfg_handle: Option<Res<MenuButtonHudConfigHandle>>,
    mut btn_q: Query<(&mut BackgroundColor, &mut Node), With<LargeMenuButtonHud>>,
    mut label_q: Query<(&mut TextColor, &mut TextFont), With<LargeMenuButtonLabelHud>>,
) {
    let Some(cfg_handle) = cfg_handle else {
        return;
    };

    let mut needs_apply = false;
    for event in events.read() {
        match event {
            AssetEvent::Added { .. } => {
                info!("✅ Menu button HUD config loaded");
                needs_apply = true;
            }
            AssetEvent::Modified { .. } => {
                info!("🔥 Menu button HUD config hot-reloaded");
                needs_apply = true;
            }
            AssetEvent::Removed { .. } => {
                warn!("⚠️ Menu button HUD config removed");
            }
            _ => {}
        }
    }

    if needs_apply && let Some(cfg) = cfg_assets.get(&cfg_handle.0) {
        for (mut bg, mut node) in btn_q.iter_mut() {
            *bg = BackgroundColor(Color::from(&cfg.color_normal));
            node.min_width = Val::Px(cfg.width);
            node.height = Val::Px(cfg.height);
        }
        for (mut tc, mut font) in label_q.iter_mut() {
            *tc = TextColor(Color::from(&cfg.text_color));
            font.font_size = cfg.font_size;
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use vs_core::states::AppState;

    fn build_app() -> App {
        use bevy::state::app::StatesPlugin;
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<AppState>();
        app
    }

    #[test]
    fn spawn_large_menu_button_produces_button_entity() {
        let mut app = build_app();

        let mut cmds = app.world_mut().commands();
        cmds.spawn(Node::default()).with_children(|parent| {
            spawn_large_menu_button(
                parent,
                "Start",
                ButtonAction::StartGame,
                None,
                Handle::default(),
                None,
            );
        });
        app.world_mut().flush();

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, (With<Button>, With<LargeMenuButtonHud>)>();
        assert_eq!(
            q.iter(app.world()).count(),
            1,
            "exactly one LargeMenuButtonHud button expected"
        );
    }

    #[test]
    fn spawn_large_menu_button_label_has_marker() {
        let mut app = build_app();

        let mut cmds = app.world_mut().commands();
        cmds.spawn(Node::default()).with_children(|parent| {
            spawn_large_menu_button(
                parent,
                "Start",
                ButtonAction::StartGame,
                None,
                Handle::default(),
                None,
            );
        });
        app.world_mut().flush();

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<LargeMenuButtonLabelHud>>();
        assert_eq!(
            q.iter(app.world()).count(),
            1,
            "exactly one LargeMenuButtonLabelHud entity expected"
        );
    }
}
