//! HP bar widget.
//!
//! Displays the player's current HP as a horizontal fill bar.
//!
//! ```text
//! HP
//! ████████░░  (200 × 16 px track; fill width ∝ current_hp / max_hp)
//! ```
//!
//! # Usage
//!
//! ```ignore
//! anchor.with_children(|p| hp_bar::spawn_hp_bar(p, cfg.get()));
//! app.add_systems(Update, hp_bar::update_hp_bar.run_if(in_state(AppState::Playing)));
//! ```

use bevy::prelude::*;
use vs_core::components::{Player, PlayerStats};

use crate::config::hud::gameplay::hp_bar::{HpBarHudConfig, HpBarHudConfigHandle};

// ---------------------------------------------------------------------------
// Fallback constants
// ---------------------------------------------------------------------------

const DEFAULT_BAR_WIDTH: f32 = 200.0;
const DEFAULT_BAR_HEIGHT: f32 = 16.0;
const DEFAULT_BAR_RADIUS: f32 = 4.0;
const DEFAULT_LABEL_FONT_SIZE: f32 = 14.0;
const DEFAULT_LABEL_GAP: f32 = 4.0;
const DEFAULT_FILL_COLOR: Color = Color::srgb(0.85, 0.20, 0.20);
const DEFAULT_TRACK_COLOR: Color = Color::srgb(0.20, 0.05, 0.05);
const DEFAULT_TEXT_COLOR: Color = Color::srgb(0.95, 0.90, 0.85);

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

/// Marks the inner fill [`Node`] of the HP bar.
///
/// [`update_hp_bar`] queries this marker to set `node.width` each frame.
#[derive(Component, Debug)]
pub struct HudHpBar;

/// Marks the background track [`Node`] of the HP bar.
///
/// [`hot_reload_hp_bar_hud`] uses this to update track dimensions and color.
#[derive(Component, Debug)]
pub struct HudHpBarTrack;

/// Marks the "HP" label [`Text`] node.
///
/// [`hot_reload_hp_bar_hud`] uses this to update font size and color.
#[derive(Component, Debug)]
pub struct HudHpBarLabel;

/// Marks the column container node of the HP bar widget.
///
/// [`hot_reload_hp_bar_hud`] uses this to update `row_gap` (label gap).
#[derive(Component, Debug)]
pub struct HudHpBarRoot;

// ---------------------------------------------------------------------------
// Spawn
// ---------------------------------------------------------------------------

/// Spawns the HP bar (label + track + fill) as a child of `parent`.
///
/// `cfg` is `None` while the RON asset is loading; fallback constants are used
/// in that case.
pub fn spawn_hp_bar(parent: &mut ChildSpawnerCommands, cfg: Option<&HpBarHudConfig>) {
    let bar_width = cfg.map(|c| c.bar_width).unwrap_or(DEFAULT_BAR_WIDTH);
    let bar_height = cfg.map(|c| c.bar_height).unwrap_or(DEFAULT_BAR_HEIGHT);
    let bar_radius = cfg.map(|c| c.bar_radius).unwrap_or(DEFAULT_BAR_RADIUS);
    let label_font_size = cfg
        .map(|c| c.label_font_size)
        .unwrap_or(DEFAULT_LABEL_FONT_SIZE);
    let label_gap = cfg.map(|c| c.label_gap).unwrap_or(DEFAULT_LABEL_GAP);
    let fill_color = cfg
        .map(|c| Color::from(&c.fill_color))
        .unwrap_or(DEFAULT_FILL_COLOR);
    let track_color = cfg
        .map(|c| Color::from(&c.track_color))
        .unwrap_or(DEFAULT_TRACK_COLOR);
    let text_color = cfg
        .map(|c| Color::from(&c.text_color))
        .unwrap_or(DEFAULT_TEXT_COLOR);

    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(label_gap),
                ..default()
            },
            HudHpBarRoot,
        ))
        .with_children(|col| {
            // "HP" label
            col.spawn((
                Text::new("HP"),
                TextFont {
                    font_size: label_font_size,
                    ..default()
                },
                TextColor(text_color),
                HudHpBarLabel,
            ));

            // background track
            col.spawn((
                Node {
                    width: Val::Px(bar_width),
                    height: Val::Px(bar_height),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(track_color),
                BorderRadius::all(Val::Px(bar_radius)),
                HudHpBarTrack,
            ))
            .with_children(|track| {
                // fill bar — width is updated by update_hp_bar
                track.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(fill_color),
                    BorderRadius::all(Val::Px(bar_radius)),
                    HudHpBar,
                ));
            });
        });
}

// ---------------------------------------------------------------------------
// Update system
// ---------------------------------------------------------------------------

/// Sets the HP bar fill width from [`PlayerStats`].
///
/// Width is `(current_hp / max_hp) * 100 %`, clamped to `[0, 100]`.
pub fn update_hp_bar(
    player_q: Query<&PlayerStats, With<Player>>,
    mut bar_q: Query<&mut Node, With<HudHpBar>>,
) {
    let Ok(stats) = player_q.single() else {
        return;
    };
    let Ok(mut node) = bar_q.single_mut() else {
        return;
    };
    let pct = if stats.max_hp > 0.0 {
        (stats.current_hp / stats.max_hp).clamp(0.0, 1.0) * 100.0
    } else {
        0.0
    };
    node.width = Val::Percent(pct);
}

// ---------------------------------------------------------------------------
// Hot-reload system
// ---------------------------------------------------------------------------

/// Updates HP bar appearance when `config/ui/hud/gameplay/hp_bar.ron` is
/// loaded or modified.
#[allow(clippy::type_complexity)]
pub fn hot_reload_hp_bar_hud(
    mut events: MessageReader<AssetEvent<HpBarHudConfig>>,
    cfg_assets: Res<Assets<HpBarHudConfig>>,
    cfg_handle: Option<Res<HpBarHudConfigHandle>>,
    mut root_q: Query<&mut Node, With<HudHpBarRoot>>,
    mut track_q: Query<
        (&mut BackgroundColor, &mut Node, &mut BorderRadius),
        (With<HudHpBarTrack>, Without<HudHpBarRoot>),
    >,
    mut fill_q: Query<
        (&mut BackgroundColor, &mut BorderRadius),
        (With<HudHpBar>, Without<HudHpBarTrack>),
    >,
    mut label_q: Query<(&mut TextColor, &mut TextFont), With<HudHpBarLabel>>,
) {
    let Some(cfg_handle) = cfg_handle else {
        return;
    };

    let mut needs_apply = false;
    for event in events.read() {
        match event {
            AssetEvent::Added { .. } => {
                info!("✅ HP bar HUD config loaded");
                needs_apply = true;
            }
            AssetEvent::Modified { .. } => {
                info!("🔥 HP bar HUD config hot-reloaded");
                needs_apply = true;
            }
            AssetEvent::Removed { .. } => {
                warn!("⚠️ HP bar HUD config removed");
            }
            _ => {}
        }
    }

    if needs_apply && let Some(cfg) = cfg_assets.get(&cfg_handle.0) {
        for mut node in root_q.iter_mut() {
            node.row_gap = Val::Px(cfg.label_gap);
        }
        let radius = BorderRadius::all(Val::Px(cfg.bar_radius));
        for (mut bg, mut node, mut br) in track_q.iter_mut() {
            *bg = BackgroundColor(Color::from(&cfg.track_color));
            node.width = Val::Px(cfg.bar_width);
            node.height = Val::Px(cfg.bar_height);
            *br = radius;
        }
        for (mut bg, mut br) in fill_q.iter_mut() {
            *bg = BackgroundColor(Color::from(&cfg.fill_color));
            *br = radius;
        }
        for (mut tc, mut font) in label_q.iter_mut() {
            *tc = TextColor(Color::from(&cfg.text_color));
            font.font_size = cfg.label_font_size;
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
                    width: Val::Percent(100.0),
                    ..default()
                },
                HudHpBar,
            ))
            .id();
        (app, bar)
    }

    fn insert_player(app: &mut App, current_hp: f32, max_hp: f32) {
        app.world_mut().spawn((
            Player,
            PlayerStats {
                current_hp,
                max_hp,
                ..default()
            },
        ));
    }

    #[test]
    fn full_hp_fills_bar_to_100_percent() {
        let (mut app, bar) = build_app_with_bar();
        insert_player(&mut app, 100.0, 100.0);
        app.world_mut().run_system_once(update_hp_bar).unwrap();
        assert_eq!(
            app.world().get::<Node>(bar).unwrap().width,
            Val::Percent(100.0)
        );
    }

    #[test]
    fn half_hp_fills_bar_to_50_percent() {
        let (mut app, bar) = build_app_with_bar();
        insert_player(&mut app, 50.0, 100.0);
        app.world_mut().run_system_once(update_hp_bar).unwrap();
        assert_eq!(
            app.world().get::<Node>(bar).unwrap().width,
            Val::Percent(50.0)
        );
    }

    #[test]
    fn zero_hp_fills_bar_to_0_percent() {
        let (mut app, bar) = build_app_with_bar();
        insert_player(&mut app, 0.0, 100.0);
        app.world_mut().run_system_once(update_hp_bar).unwrap();
        assert_eq!(
            app.world().get::<Node>(bar).unwrap().width,
            Val::Percent(0.0)
        );
    }

    #[test]
    fn hp_over_max_is_clamped_to_100_percent() {
        let (mut app, bar) = build_app_with_bar();
        insert_player(&mut app, 150.0, 100.0);
        app.world_mut().run_system_once(update_hp_bar).unwrap();
        assert_eq!(
            app.world().get::<Node>(bar).unwrap().width,
            Val::Percent(100.0)
        );
    }

    #[test]
    fn no_player_leaves_bar_unchanged() {
        let (mut app, bar) = build_app_with_bar();
        // No player spawned — system should be a no-op.
        app.world_mut().run_system_once(update_hp_bar).unwrap();
        // Width stays at the initial value set in build_app_with_bar.
        assert_eq!(
            app.world().get::<Node>(bar).unwrap().width,
            Val::Percent(100.0)
        );
    }
}
