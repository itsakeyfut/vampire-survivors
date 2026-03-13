//! Gold HUD widget.
//!
//! Displays the gold earned in the current run in the bottom-right corner,
//! above the kill count.
//!
//! ```text
//! Gold: 0 G       (updates to "Gold: 50 G" when a chest is opened)
//! Kills: 0
//! ```

use bevy::prelude::*;
use vs_core::resources::GameData;

use crate::config::hud::gameplay::GoldHudConfig;
use crate::config::hud::gameplay::gold::GoldHudConfigHandle;

// ---------------------------------------------------------------------------
// Fallback constants
// ---------------------------------------------------------------------------

const DEFAULT_FONT_SIZE: f32 = 14.0;
const DEFAULT_TEXT_COLOR: Color = Color::srgb(1.0, 0.85, 0.2);

// ---------------------------------------------------------------------------
// Marker component
// ---------------------------------------------------------------------------

/// Marks the gold [`Text`] node.
///
/// [`update_gold`] queries this marker to update the displayed amount each frame.
#[derive(Component, Debug)]
pub struct HudGold;

// ---------------------------------------------------------------------------
// Spawn
// ---------------------------------------------------------------------------

/// Spawns the gold label as a child of `parent`.
///
/// `cfg` is `None` while the RON asset is loading; fallback constants are used
/// in that case.
pub fn spawn_gold(
    parent: &mut ChildSpawnerCommands,
    cfg: Option<&GoldHudConfig>,
    font: Handle<Font>,
) {
    let font_size = cfg.map(|c| c.font_size).unwrap_or(DEFAULT_FONT_SIZE);
    let text_color = cfg
        .map(|c| Color::from(&c.text_color))
        .unwrap_or(DEFAULT_TEXT_COLOR);

    parent.spawn((
        Text::new("Gold: 0 G"),
        TextFont {
            font,
            font_size,
            ..default()
        },
        TextColor(text_color),
        HudGold,
    ));
}

// ---------------------------------------------------------------------------
// Update system
// ---------------------------------------------------------------------------

/// Updates the gold label to reflect [`GameData::gold_earned`].
pub fn update_gold(game_data: Res<GameData>, mut text_q: Query<&mut Text, With<HudGold>>) {
    let Ok(mut text) = text_q.single_mut() else {
        return;
    };
    *text = Text::new(format!("Gold: {} G", game_data.gold_earned));
}

// ---------------------------------------------------------------------------
// Hot-reload system
// ---------------------------------------------------------------------------

/// Updates gold label appearance when `config/ui/hud/gameplay/gold.ron`
/// is loaded or modified.
pub fn hot_reload_gold_hud(
    mut events: MessageReader<AssetEvent<GoldHudConfig>>,
    cfg_assets: Res<Assets<GoldHudConfig>>,
    cfg_handle: Option<Res<GoldHudConfigHandle>>,
    mut label_q: Query<(&mut TextColor, &mut TextFont), With<HudGold>>,
) {
    let Some(cfg_handle) = cfg_handle else {
        return;
    };

    let mut needs_apply = false;
    for event in events.read() {
        match event {
            AssetEvent::Added { .. } => {
                info!("Gold HUD config loaded");
                needs_apply = true;
            }
            AssetEvent::Modified { .. } => {
                info!("Gold HUD config hot-reloaded");
                needs_apply = true;
            }
            AssetEvent::Removed { .. } => {
                warn!("Gold HUD config removed");
            }
            _ => {}
        }
    }

    if needs_apply && let Some(cfg) = cfg_assets.get(&cfg_handle.0) {
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
    use bevy::ecs::system::RunSystemOnce as _;

    use super::*;

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(GameData::default());
        app
    }

    fn spawn_label(app: &mut App) -> Entity {
        app.world_mut()
            .spawn((Text::new("Gold: 0 G"), HudGold))
            .id()
    }

    #[test]
    fn zero_gold_shows_zero() {
        let mut app = build_app();
        let label = spawn_label(&mut app);

        app.world_mut().run_system_once(update_gold).unwrap();

        let text = app.world().get::<Text>(label).unwrap();
        assert!(text.0.contains('0'), "expected '0' in '{}'", text.0);
    }

    #[test]
    fn gold_reflects_game_data() {
        let mut app = build_app();
        app.world_mut().resource_mut::<GameData>().gold_earned = 150;
        let label = spawn_label(&mut app);

        app.world_mut().run_system_once(update_gold).unwrap();

        let text = app.world().get::<Text>(label).unwrap();
        assert!(text.0.contains("150"), "expected '150' in '{}'", text.0);
    }

    #[test]
    fn gold_label_includes_unit() {
        let mut app = build_app();
        app.world_mut().resource_mut::<GameData>().gold_earned = 50;
        let label = spawn_label(&mut app);

        app.world_mut().run_system_once(update_gold).unwrap();

        let text = app.world().get::<Text>(label).unwrap();
        assert!(text.0.contains('G'), "expected 'G' unit in '{}'", text.0);
    }
}
