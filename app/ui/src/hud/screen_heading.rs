//! Screen heading HUD widget.
//!
//! Spawns a single large text node used as the main heading of any screen.
//! Text content and colour are screen-specific parameters; layout
//! (font size, bottom margin) is read from [`ScreenHeadingHudConfig`].

use bevy::prelude::*;

use crate::config::hud::screen_heading::{ScreenHeadingHudConfig, ScreenHeadingHudConfigHandle};

// ---------------------------------------------------------------------------
// Fallback constants
// ---------------------------------------------------------------------------

const DEFAULT_FONT_SIZE: f32 = 72.0;
const DEFAULT_MARGIN_BOTTOM: f32 = 80.0;

// ---------------------------------------------------------------------------
// Marker component
// ---------------------------------------------------------------------------

/// Marker component placed on every screen heading text entity.
///
/// Used by [`hot_reload_screen_heading_hud`] to find and update live entities
/// when `config/ui/hud/screen_heading.ron` changes.
#[derive(Component)]
pub struct ScreenHeadingHud;

// ---------------------------------------------------------------------------
// Spawn function
// ---------------------------------------------------------------------------

/// Spawns a screen heading text node as a child of `parent`.
///
/// - `text`  — the heading string displayed to the player.
/// - `color` — screen-specific text colour (e.g. gold for level-up, red for game-over).
/// - `cfg`   — layout config; pass `heading_params.get()`. Falls back to `DEFAULT_*`
///   constants when the asset is not yet loaded.
pub fn spawn_screen_heading(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    color: Color,
    cfg: Option<&ScreenHeadingHudConfig>,
) {
    let font_size = cfg.map(|c| c.font_size).unwrap_or(DEFAULT_FONT_SIZE);
    let margin_bottom = cfg
        .map(|c| c.margin_bottom)
        .unwrap_or(DEFAULT_MARGIN_BOTTOM);

    parent.spawn((
        Text::new(text),
        TextFont {
            font_size,
            ..default()
        },
        TextColor(color),
        Node {
            margin: UiRect::bottom(Val::Px(margin_bottom)),
            ..default()
        },
        ScreenHeadingHud,
    ));
}

// ---------------------------------------------------------------------------
// Hot-reload system
// ---------------------------------------------------------------------------

/// Updates font size and bottom margin of all [`ScreenHeadingHud`] entities
/// when `config/ui/hud/screen_heading.ron` is loaded or modified.
///
/// Text colours are screen-specific and are **not** updated here; each screen
/// manages its own colour via [`crate::config::UiStyleParams`] or a local constant.
pub fn hot_reload_screen_heading_hud(
    mut events: MessageReader<AssetEvent<ScreenHeadingHudConfig>>,
    cfg_assets: Res<Assets<ScreenHeadingHudConfig>>,
    cfg_handle: Option<Res<ScreenHeadingHudConfigHandle>>,
    mut heading_q: Query<(&mut TextFont, &mut Node), With<ScreenHeadingHud>>,
) {
    let Some(cfg_handle) = cfg_handle else {
        return;
    };

    let mut needs_apply = false;
    for event in events.read() {
        match event {
            AssetEvent::Added { .. } => {
                info!("✅ Screen heading HUD config loaded");
                needs_apply = true;
            }
            AssetEvent::Modified { .. } => {
                info!("🔥 Screen heading HUD config hot-reloaded");
                needs_apply = true;
            }
            AssetEvent::Removed { .. } => {
                warn!("⚠️ Screen heading HUD config removed");
            }
            _ => {}
        }
    }

    if needs_apply && let Some(cfg) = cfg_assets.get(&cfg_handle.0) {
        for (mut font, mut node) in heading_q.iter_mut() {
            font.font_size = cfg.font_size;
            node.margin = UiRect::bottom(Val::Px(cfg.margin_bottom));
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
    fn spawn_screen_heading_uses_fallbacks_when_cfg_is_none() {
        use bevy::state::app::StatesPlugin;
        use vs_core::states::AppState;

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<AppState>();

        let mut cmds = app.world_mut().commands();
        cmds.spawn(Node::default()).with_children(|parent| {
            spawn_screen_heading(parent, "TEST", Color::WHITE, None);
        });
        app.world_mut().flush();

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<ScreenHeadingHud>>();
        assert_eq!(
            q.iter(app.world()).count(),
            1,
            "spawn_screen_heading must produce one ScreenHeadingHud entity"
        );
    }
}
