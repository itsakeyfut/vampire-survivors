//! Level display widget.
//!
//! Shows the player's current level as `"Lv. N"`.
//!
//! # Usage
//!
//! ```ignore
//! anchor.with_children(|p| level::spawn_level(p, cfg.get()));
//! app.add_systems(Update, level::update_level_text.run_if(in_state(AppState::Playing)));
//! ```

use bevy::prelude::*;
use vs_core::resources::GameData;

use crate::config::hud::gameplay::level::{LevelHudConfig, LevelHudConfigHandle};

// ---------------------------------------------------------------------------
// Fallback constants
// ---------------------------------------------------------------------------

const DEFAULT_FONT_SIZE: f32 = 22.0;
const DEFAULT_TEXT_COLOR: Color = Color::srgb(0.95, 0.90, 0.85);

// ---------------------------------------------------------------------------
// Marker component
// ---------------------------------------------------------------------------

/// Marks the [`Text`] node showing the current player level.
///
/// [`update_level_text`] queries this marker to update the text each frame.
#[derive(Component, Debug)]
pub struct HudLevel;

// ---------------------------------------------------------------------------
// Spawn
// ---------------------------------------------------------------------------

/// Spawns the level text node as a child of `parent`.
///
/// Initial text is `"Lv. 1"`.
///
/// `cfg` is `None` while the RON asset is loading; fallback constants are used
/// in that case.
pub fn spawn_level(
    parent: &mut ChildSpawnerCommands,
    cfg: Option<&LevelHudConfig>,
    font: Handle<Font>,
) {
    let font_size = cfg.map(|c| c.font_size).unwrap_or(DEFAULT_FONT_SIZE);
    let text_color = cfg
        .map(|c| Color::from(&c.text_color))
        .unwrap_or(DEFAULT_TEXT_COLOR);

    parent.spawn((
        Text::new("Lv. 1"),
        TextFont {
            font,
            font_size,
            ..default()
        },
        TextColor(text_color),
        HudLevel,
    ));
}

// ---------------------------------------------------------------------------
// Update system
// ---------------------------------------------------------------------------

/// Writes `"Lv. N"` to [`HudLevel`] from [`GameData::current_level`].
pub fn update_level_text(game_data: Res<GameData>, mut level_q: Query<&mut Text, With<HudLevel>>) {
    let Ok(mut text) = level_q.single_mut() else {
        return;
    };
    text.0 = format!("Lv. {}", game_data.current_level);
}

// ---------------------------------------------------------------------------
// Hot-reload system
// ---------------------------------------------------------------------------

/// Updates level label appearance when `config/ui/hud/gameplay/level.ron` is
/// loaded or modified.
pub fn hot_reload_level_hud(
    mut events: MessageReader<AssetEvent<LevelHudConfig>>,
    cfg_assets: Res<Assets<LevelHudConfig>>,
    cfg_handle: Option<Res<LevelHudConfigHandle>>,
    mut level_q: Query<(&mut TextColor, &mut TextFont), With<HudLevel>>,
) {
    let Some(cfg_handle) = cfg_handle else {
        return;
    };

    let mut needs_apply = false;
    for event in events.read() {
        match event {
            AssetEvent::Added { .. } => {
                info!("✅ Level HUD config loaded");
                needs_apply = true;
            }
            AssetEvent::Modified { .. } => {
                info!("🔥 Level HUD config hot-reloaded");
                needs_apply = true;
            }
            AssetEvent::Removed { .. } => {
                warn!("⚠️ Level HUD config removed");
            }
            _ => {}
        }
    }

    if needs_apply && let Some(cfg) = cfg_assets.get(&cfg_handle.0) {
        for (mut tc, mut font) in level_q.iter_mut() {
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
    use bevy::ecs::system::RunSystemOnce as _;
    use bevy::state::app::StatesPlugin;

    use super::*;

    fn build_app_with_level() -> (App, Entity) {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        let e = app.world_mut().spawn((Text::new("?"), HudLevel)).id();
        (app, e)
    }

    #[test]
    fn shows_level_one_at_start() {
        let (mut app, e) = build_app_with_level();
        app.insert_resource(GameData::default());
        app.world_mut().run_system_once(update_level_text).unwrap();
        assert_eq!(app.world().get::<Text>(e).unwrap().0, "Lv. 1");
    }

    #[test]
    fn updates_when_level_increases() {
        let (mut app, e) = build_app_with_level();
        let mut gd = GameData::default();
        gd.current_level = 7;
        app.insert_resource(gd);
        app.world_mut().run_system_once(update_level_text).unwrap();
        assert_eq!(app.world().get::<Text>(e).unwrap().0, "Lv. 7");
    }
}
