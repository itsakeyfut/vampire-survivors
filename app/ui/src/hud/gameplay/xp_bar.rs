//! XP bar widget.
//!
//! Displays XP progress toward the next level as a full-viewport-width
//! horizontal fill bar pinned to the bottom of the screen.
//!
//! ```text
//! ░░░░████████░░░░░░░░░░░░░░░░░░  (full viewport width × 10 px)
//! ```
//!
//! # Usage
//!
//! ```ignore
//! anchor.with_children(|p| xp_bar::spawn_xp_bar(p, cfg.get()));
//! app.add_systems(Update, xp_bar::update_xp_bar.run_if(in_state(AppState::Playing)));
//! ```

use bevy::prelude::*;
use vs_core::resources::GameData;

use crate::config::hud::gameplay::xp_bar::{XpBarHudConfig, XpBarHudConfigHandle};

// ---------------------------------------------------------------------------
// Fallback constants
// ---------------------------------------------------------------------------

const DEFAULT_BAR_HEIGHT: f32 = 10.0;
const DEFAULT_FILL_COLOR: Color = Color::srgb(0.25, 0.65, 1.00);
const DEFAULT_TRACK_COLOR: Color = Color::srgb(0.05, 0.08, 0.20);

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

/// Marks the inner fill [`Node`] of the XP bar.
///
/// [`update_xp_bar`] queries this marker to set `node.width` each frame.
#[derive(Component, Debug)]
pub struct HudXpBar;

/// Marks the background track [`Node`] of the XP bar.
///
/// [`hot_reload_xp_bar_hud`] uses this to update track height and color.
#[derive(Component, Debug)]
pub struct HudXpBarTrack;

// ---------------------------------------------------------------------------
// Spawn
// ---------------------------------------------------------------------------

/// Spawns the XP bar (track + fill) as a child of `parent`.
///
/// The track occupies 100 % of the parent's width; the fill starts at 0 %.
/// The caller is responsible for positioning the parent (typically pinned to
/// the bottom edge of the screen).
///
/// `cfg` is `None` while the RON asset is loading; fallback constants are used
/// in that case.
pub fn spawn_xp_bar(parent: &mut ChildSpawnerCommands, cfg: Option<&XpBarHudConfig>) {
    let bar_height = cfg.map(|c| c.bar_height).unwrap_or(DEFAULT_BAR_HEIGHT);
    let fill_color = cfg
        .map(|c| Color::from(&c.fill_color))
        .unwrap_or(DEFAULT_FILL_COLOR);
    let track_color = cfg
        .map(|c| Color::from(&c.track_color))
        .unwrap_or(DEFAULT_TRACK_COLOR);

    // background track
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(bar_height),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(track_color),
            HudXpBarTrack,
        ))
        .with_children(|track| {
            // fill bar — width is updated by update_xp_bar
            track.spawn((
                Node {
                    width: Val::Percent(0.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(fill_color),
                HudXpBar,
            ));
        });
}

// ---------------------------------------------------------------------------
// Update system
// ---------------------------------------------------------------------------

/// Sets the XP bar fill width from [`GameData`].
///
/// Width is `(current_xp / xp_to_next_level) * 100 %`, clamped to `[0, 100]`.
/// When `xp_to_next_level` is 0 (shouldn't happen in normal play), the bar
/// shows 100 % as a safe default.
pub fn update_xp_bar(game_data: Res<GameData>, mut bar_q: Query<&mut Node, With<HudXpBar>>) {
    let Ok(mut node) = bar_q.single_mut() else {
        return;
    };
    let pct = if game_data.xp_to_next_level > 0 {
        (game_data.current_xp as f32 / game_data.xp_to_next_level as f32).clamp(0.0, 1.0) * 100.0
    } else {
        100.0
    };
    node.width = Val::Percent(pct);
}

// ---------------------------------------------------------------------------
// Hot-reload system
// ---------------------------------------------------------------------------

/// Updates XP bar appearance when `config/ui/hud/gameplay/xp_bar.ron` is
/// loaded or modified.
pub fn hot_reload_xp_bar_hud(
    mut events: MessageReader<AssetEvent<XpBarHudConfig>>,
    cfg_assets: Res<Assets<XpBarHudConfig>>,
    cfg_handle: Option<Res<XpBarHudConfigHandle>>,
    mut track_q: Query<(&mut BackgroundColor, &mut Node), With<HudXpBarTrack>>,
    mut fill_q: Query<&mut BackgroundColor, (With<HudXpBar>, Without<HudXpBarTrack>)>,
) {
    let Some(cfg_handle) = cfg_handle else {
        return;
    };

    let mut needs_apply = false;
    for event in events.read() {
        match event {
            AssetEvent::Added { .. } => {
                info!("✅ XP bar HUD config loaded");
                needs_apply = true;
            }
            AssetEvent::Modified { .. } => {
                info!("🔥 XP bar HUD config hot-reloaded");
                needs_apply = true;
            }
            AssetEvent::Removed { .. } => {
                warn!("⚠️ XP bar HUD config removed");
            }
            _ => {}
        }
    }

    if needs_apply && let Some(cfg) = cfg_assets.get(&cfg_handle.0) {
        for (mut bg, mut node) in track_q.iter_mut() {
            *bg = BackgroundColor(Color::from(&cfg.track_color));
            node.height = Val::Px(cfg.bar_height);
        }
        for mut bg in fill_q.iter_mut() {
            *bg = BackgroundColor(Color::from(&cfg.fill_color));
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

    fn build_app_with_bar() -> (App, Entity) {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        let bar = app
            .world_mut()
            .spawn((
                Node {
                    width: Val::Percent(50.0),
                    ..default()
                },
                HudXpBar,
            ))
            .id();
        (app, bar)
    }

    #[test]
    fn zero_xp_fills_bar_to_0_percent() {
        let (mut app, bar) = build_app_with_bar();
        app.insert_resource(GameData::default()); // current_xp = 0
        app.world_mut().run_system_once(update_xp_bar).unwrap();
        assert_eq!(
            app.world().get::<Node>(bar).unwrap().width,
            Val::Percent(0.0)
        );
    }

    #[test]
    fn half_xp_fills_bar_to_50_percent() {
        let (mut app, bar) = build_app_with_bar();
        let mut gd = GameData::default();
        gd.current_xp = 10;
        gd.xp_to_next_level = 20;
        app.insert_resource(gd);
        app.world_mut().run_system_once(update_xp_bar).unwrap();
        assert_eq!(
            app.world().get::<Node>(bar).unwrap().width,
            Val::Percent(50.0)
        );
    }

    #[test]
    fn full_xp_fills_bar_to_100_percent() {
        let (mut app, bar) = build_app_with_bar();
        let mut gd = GameData::default();
        gd.current_xp = 20;
        gd.xp_to_next_level = 20;
        app.insert_resource(gd);
        app.world_mut().run_system_once(update_xp_bar).unwrap();
        assert_eq!(
            app.world().get::<Node>(bar).unwrap().width,
            Val::Percent(100.0)
        );
    }

    #[test]
    fn xp_over_threshold_is_clamped_to_100_percent() {
        let (mut app, bar) = build_app_with_bar();
        let mut gd = GameData::default();
        gd.current_xp = 30;
        gd.xp_to_next_level = 20;
        app.insert_resource(gd);
        app.world_mut().run_system_once(update_xp_bar).unwrap();
        assert_eq!(
            app.world().get::<Node>(bar).unwrap().width,
            Val::Percent(100.0)
        );
    }
}
