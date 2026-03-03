//! Timer widget.
//!
//! Displays the elapsed gameplay time in `M:SS` format.
//!
//! # Usage
//!
//! ```ignore
//! anchor.with_children(|p| timer::spawn_timer(p, cfg.get()));
//! app.add_systems(Update, timer::update_timer.run_if(in_state(AppState::Playing)));
//! ```

use bevy::prelude::*;
use vs_core::resources::GameData;

use crate::config::hud::gameplay::timer::{TimerHudConfig, TimerHudConfigHandle};

// ---------------------------------------------------------------------------
// Fallback constants
// ---------------------------------------------------------------------------

const DEFAULT_FONT_SIZE: f32 = 28.0;
const DEFAULT_TEXT_COLOR: Color = Color::srgb(0.95, 0.90, 0.85);

// ---------------------------------------------------------------------------
// Marker component
// ---------------------------------------------------------------------------

/// Marks the [`Text`] node showing elapsed gameplay time.
///
/// [`update_timer`] queries this marker to update the text each frame.
#[derive(Component, Debug)]
pub struct HudTimer;

// ---------------------------------------------------------------------------
// Spawn
// ---------------------------------------------------------------------------

/// Spawns the timer text node as a child of `parent`.
///
/// Initial text is `"0:00"`.
///
/// `cfg` is `None` while the RON asset is loading; fallback constants are used
/// in that case.
pub fn spawn_timer(parent: &mut ChildSpawnerCommands, cfg: Option<&TimerHudConfig>) {
    let font_size = cfg.map(|c| c.font_size).unwrap_or(DEFAULT_FONT_SIZE);
    let text_color = cfg
        .map(|c| Color::from(&c.text_color))
        .unwrap_or(DEFAULT_TEXT_COLOR);

    parent.spawn((
        Text::new("0:00"),
        TextFont {
            font_size,
            ..default()
        },
        TextColor(text_color),
        HudTimer,
    ));
}

// ---------------------------------------------------------------------------
// Update system
// ---------------------------------------------------------------------------

/// Formats [`GameData::elapsed_time`] as `M:SS` and writes it to [`HudTimer`].
pub fn update_timer(game_data: Res<GameData>, mut timer_q: Query<&mut Text, With<HudTimer>>) {
    let Ok(mut text) = timer_q.single_mut() else {
        return;
    };
    text.0 = format_elapsed(game_data.elapsed_time as u32);
}

// ---------------------------------------------------------------------------
// Hot-reload system
// ---------------------------------------------------------------------------

/// Updates timer appearance when `config/ui/hud/gameplay/timer.ron` is
/// loaded or modified.
pub fn hot_reload_timer_hud(
    mut events: MessageReader<AssetEvent<TimerHudConfig>>,
    cfg_assets: Res<Assets<TimerHudConfig>>,
    cfg_handle: Option<Res<TimerHudConfigHandle>>,
    mut timer_q: Query<(&mut TextColor, &mut TextFont), With<HudTimer>>,
) {
    let Some(cfg_handle) = cfg_handle else {
        return;
    };

    let mut needs_apply = false;
    for event in events.read() {
        match event {
            AssetEvent::Added { .. } => {
                info!("✅ Timer HUD config loaded");
                needs_apply = true;
            }
            AssetEvent::Modified { .. } => {
                info!("🔥 Timer HUD config hot-reloaded");
                needs_apply = true;
            }
            AssetEvent::Removed { .. } => {
                warn!("⚠️ Timer HUD config removed");
            }
            _ => {}
        }
    }

    if needs_apply && let Some(cfg) = cfg_assets.get(&cfg_handle.0) {
        for (mut tc, mut font) in timer_q.iter_mut() {
            *tc = TextColor(Color::from(&cfg.text_color));
            font.font_size = cfg.font_size;
        }
    }
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

/// Formats elapsed seconds as `M:SS`.
///
/// Minutes are not zero-padded; seconds always use two digits.
///
/// # Examples
///
/// ```
/// # use vs_ui::hud::gameplay::timer::format_elapsed;
/// assert_eq!(format_elapsed(0),    "0:00");
/// assert_eq!(format_elapsed(59),   "0:59");
/// assert_eq!(format_elapsed(60),   "1:00");
/// assert_eq!(format_elapsed(65),   "1:05");
/// assert_eq!(format_elapsed(3661), "61:01");
/// ```
pub fn format_elapsed(total_secs: u32) -> String {
    format!("{}:{:02}", total_secs / 60, total_secs % 60)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce as _;
    use bevy::state::app::StatesPlugin;

    use super::*;

    // --- format_elapsed ---

    #[test]
    fn format_zero() {
        assert_eq!(format_elapsed(0), "0:00");
    }

    #[test]
    fn format_under_one_minute() {
        assert_eq!(format_elapsed(5), "0:05");
        assert_eq!(format_elapsed(59), "0:59");
    }

    #[test]
    fn format_exactly_one_minute() {
        assert_eq!(format_elapsed(60), "1:00");
    }

    #[test]
    fn format_one_minute_five_seconds() {
        assert_eq!(format_elapsed(65), "1:05");
    }

    #[test]
    fn format_large_value() {
        assert_eq!(format_elapsed(3661), "61:01");
    }

    // --- update_timer ---

    fn build_app_with_timer() -> (App, Entity) {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        let e = app.world_mut().spawn((Text::new("?"), HudTimer)).id();
        (app, e)
    }

    #[test]
    fn timer_shows_zero_at_start() {
        let (mut app, e) = build_app_with_timer();
        app.insert_resource(GameData::default());
        app.world_mut().run_system_once(update_timer).unwrap();
        assert_eq!(app.world().get::<Text>(e).unwrap().0, "0:00");
    }

    #[test]
    fn timer_formats_two_minutes_five_seconds() {
        let (mut app, e) = build_app_with_timer();
        let mut gd = GameData::default();
        gd.elapsed_time = 125.0;
        app.insert_resource(gd);
        app.world_mut().run_system_once(update_timer).unwrap();
        assert_eq!(app.world().get::<Text>(e).unwrap().0, "2:05");
    }

    #[test]
    fn timer_truncates_fractional_seconds() {
        let (mut app, e) = build_app_with_timer();
        let mut gd = GameData::default();
        gd.elapsed_time = 61.9; // still "1:01", not "1:02"
        app.insert_resource(gd);
        app.world_mut().run_system_once(update_timer).unwrap();
        assert_eq!(app.world().get::<Text>(e).unwrap().0, "1:01");
    }
}
