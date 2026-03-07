//! Kill count HUD widget.
//!
//! Displays the total number of enemies defeated this run in the bottom-right
//! corner, above the XP bar.
//!
//! ```text
//! Kills: 0        (updates to 42 when 42 enemies have died)
//! ```

use bevy::prelude::*;
use vs_core::resources::GameData;

use crate::config::hud::gameplay::KillCountHudConfig;
use crate::config::hud::gameplay::kill_count::KillCountHudConfigHandle;

// ---------------------------------------------------------------------------
// Fallback constants
// ---------------------------------------------------------------------------

const DEFAULT_FONT_SIZE: f32 = 14.0;
const DEFAULT_TEXT_COLOR: Color = Color::srgb(0.95, 0.90, 0.85);

// ---------------------------------------------------------------------------
// Marker component
// ---------------------------------------------------------------------------

/// Marks the kill count [`Text`] node.
///
/// [`update_kill_count`] queries this marker to update the displayed count
/// each frame.
#[derive(Component, Debug)]
pub struct HudKillCount;

// ---------------------------------------------------------------------------
// Spawn
// ---------------------------------------------------------------------------

/// Spawns the kill count label as a child of `parent`.
///
/// `cfg` is `None` while the RON asset is loading; fallback constants are used
/// in that case.
pub fn spawn_kill_count(parent: &mut ChildSpawnerCommands, cfg: Option<&KillCountHudConfig>) {
    let font_size = cfg.map(|c| c.font_size).unwrap_or(DEFAULT_FONT_SIZE);
    let text_color = cfg
        .map(|c| Color::from(&c.text_color))
        .unwrap_or(DEFAULT_TEXT_COLOR);

    parent.spawn((
        Text::new("Kills: 0"),
        TextFont {
            font_size,
            ..default()
        },
        TextColor(text_color),
        HudKillCount,
    ));
}

// ---------------------------------------------------------------------------
// Update system
// ---------------------------------------------------------------------------

/// Updates the kill count label to reflect [`GameData::kill_count`].
pub fn update_kill_count(
    game_data: Res<GameData>,
    mut text_q: Query<&mut Text, With<HudKillCount>>,
) {
    let Ok(mut text) = text_q.single_mut() else {
        return;
    };
    *text = Text::new(format!("Kills: {}", game_data.kill_count));
}

// ---------------------------------------------------------------------------
// Hot-reload system
// ---------------------------------------------------------------------------

/// Updates kill count label appearance when `config/ui/hud/gameplay/kill_count.ron`
/// is loaded or modified.
pub fn hot_reload_kill_count_hud(
    mut events: MessageReader<AssetEvent<KillCountHudConfig>>,
    cfg_assets: Res<Assets<KillCountHudConfig>>,
    cfg_handle: Option<Res<KillCountHudConfigHandle>>,
    mut label_q: Query<(&mut TextColor, &mut TextFont), With<HudKillCount>>,
) {
    let Some(cfg_handle) = cfg_handle else {
        return;
    };

    let mut needs_apply = false;
    for event in events.read() {
        match event {
            AssetEvent::Added { .. } => {
                info!("Kill count HUD config loaded");
                needs_apply = true;
            }
            AssetEvent::Modified { .. } => {
                info!("Kill count HUD config hot-reloaded");
                needs_apply = true;
            }
            AssetEvent::Removed { .. } => {
                warn!("Kill count HUD config removed");
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
            .spawn((Text::new("Kills: 0"), HudKillCount))
            .id()
    }

    #[test]
    fn zero_kills_shows_zero() {
        let mut app = build_app();
        let label = spawn_label(&mut app);

        app.world_mut().run_system_once(update_kill_count).unwrap();

        let text = app.world().get::<Text>(label).unwrap();
        assert!(text.0.contains('0'), "expected '0' in '{}'", text.0);
    }

    #[test]
    fn kill_count_reflects_game_data() {
        let mut app = build_app();
        app.world_mut().resource_mut::<GameData>().kill_count = 42;
        let label = spawn_label(&mut app);

        app.world_mut().run_system_once(update_kill_count).unwrap();

        let text = app.world().get::<Text>(label).unwrap();
        assert!(text.0.contains("42"), "expected '42' in '{}'", text.0);
    }
}
