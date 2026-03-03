//! Gameplay HUD — in-game overlay widgets.
//!
//! [`setup_gameplay_hud`] runs on [`OnEnter(AppState::Playing)`] and spawns a
//! transparent full-screen root node.  Each widget is positioned in its own
//! absolute-positioned anchor so layouts stay independent.
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │  HP                  Lv. 1              0:00    │
//! │  ██████████░░░░░░                               │
//! │                                                 │
//! │░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░│ ← XP bar
//! └─────────────────────────────────────────────────┘
//! ```
//!
//! | Module      | Widget         | Spawn fn          | Update system           |
//! |-------------|----------------|-------------------|-------------------------|
//! | [`hp_bar`]  | HP bar         | `spawn_hp_bar`    | `update_hp_bar`         |
//! | [`xp_bar`]  | XP bar         | `spawn_xp_bar`    | `update_xp_bar`         |
//! | [`timer`]   | Elapsed timer  | `spawn_timer`     | `update_timer`          |
//! | [`level`]   | Level label    | `spawn_level`     | `update_level_text`     |

pub mod hp_bar;
pub mod level;
pub mod timer;
pub mod xp_bar;

use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;
use vs_core::states::AppState;

use crate::config::hud::gameplay::{
    GameplayHudLayoutConfig, GameplayHudLayoutConfigHandle, GameplayHudLayoutParams,
    HpBarHudParams, LevelHudParams, TimerHudParams, XpBarHudParams,
};

// ---------------------------------------------------------------------------
// Fallback constant
// ---------------------------------------------------------------------------

const DEFAULT_EDGE_MARGIN: f32 = 12.0;

// ---------------------------------------------------------------------------
// Anchor marker components
// ---------------------------------------------------------------------------

/// Marks the top-left anchor node (holds the HP bar).
#[derive(Component, Debug)]
pub struct HudHpBarAnchor;

/// Marks the top-center anchor node (holds the level label).
#[derive(Component, Debug)]
pub struct HudLevelAnchor;

/// Marks the top-right anchor node (holds the timer).
#[derive(Component, Debug)]
pub struct HudTimerAnchor;

/// Marks the bottom anchor node (holds the XP bar).
#[derive(Component, Debug)]
pub struct HudXpBarAnchor;

// ---------------------------------------------------------------------------
// Setup system
// ---------------------------------------------------------------------------

/// Spawns the full gameplay HUD overlay.
///
/// Runs on [`OnEnter(AppState::Playing)`].  The root node carries
/// [`DespawnOnExit`]`(`[`AppState::Playing`]`)` so all HUD entities are
/// automatically cleaned up on every state transition away from `Playing`.
pub fn setup_gameplay_hud(
    mut commands: Commands,
    layout_cfg: GameplayHudLayoutParams,
    hp_bar_cfg: HpBarHudParams,
    xp_bar_cfg: XpBarHudParams,
    timer_cfg: TimerHudParams,
    level_cfg: LevelHudParams,
) {
    let edge = layout_cfg
        .get()
        .map(|c| c.edge_margin)
        .unwrap_or(DEFAULT_EDGE_MARGIN);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            DespawnOnExit(AppState::Playing),
        ))
        .with_children(|root| {
            // ------------------------------------------------------------------
            // Top-left: HP bar
            // ------------------------------------------------------------------
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(edge),
                    left: Val::Px(edge),
                    ..default()
                },
                HudHpBarAnchor,
            ))
            .with_children(|anchor| {
                hp_bar::spawn_hp_bar(anchor, hp_bar_cfg.get());
            });

            // ------------------------------------------------------------------
            // Top-center: Level label
            // ------------------------------------------------------------------
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(edge),
                    left: Val::Percent(50.0),
                    ..default()
                },
                HudLevelAnchor,
            ))
            .with_children(|anchor| {
                level::spawn_level(anchor, level_cfg.get());
            });

            // ------------------------------------------------------------------
            // Top-right: Timer
            // ------------------------------------------------------------------
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(edge),
                    right: Val::Px(edge),
                    ..default()
                },
                HudTimerAnchor,
            ))
            .with_children(|anchor| {
                timer::spawn_timer(anchor, timer_cfg.get());
            });

            // ------------------------------------------------------------------
            // Bottom: XP bar (full viewport width)
            // ------------------------------------------------------------------
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(0.0),
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    ..default()
                },
                HudXpBarAnchor,
            ))
            .with_children(|anchor| {
                xp_bar::spawn_xp_bar(anchor, xp_bar_cfg.get());
            });
        });
}

// ---------------------------------------------------------------------------
// Layout hot-reload system
// ---------------------------------------------------------------------------

/// Repositions HUD anchor nodes when `config/ui/hud/gameplay/layout.ron` is
/// loaded or modified.
#[allow(clippy::type_complexity)]
pub fn hot_reload_gameplay_layout(
    mut events: MessageReader<AssetEvent<GameplayHudLayoutConfig>>,
    cfg_assets: Res<Assets<GameplayHudLayoutConfig>>,
    cfg_handle: Option<Res<GameplayHudLayoutConfigHandle>>,
    mut hp_q: Query<&mut Node, With<HudHpBarAnchor>>,
    mut level_q: Query<&mut Node, (With<HudLevelAnchor>, Without<HudHpBarAnchor>)>,
    mut timer_q: Query<
        &mut Node,
        (
            With<HudTimerAnchor>,
            Without<HudHpBarAnchor>,
            Without<HudLevelAnchor>,
        ),
    >,
) {
    let Some(cfg_handle) = cfg_handle else {
        return;
    };

    let mut needs_apply = false;
    for event in events.read() {
        match event {
            AssetEvent::Added { .. } => {
                info!("✅ Gameplay HUD layout config loaded");
                needs_apply = true;
            }
            AssetEvent::Modified { .. } => {
                info!("🔥 Gameplay HUD layout config hot-reloaded");
                needs_apply = true;
            }
            AssetEvent::Removed { .. } => {
                warn!("⚠️ Gameplay HUD layout config removed");
            }
            _ => {}
        }
    }

    if needs_apply && let Some(cfg) = cfg_assets.get(&cfg_handle.0) {
        let edge = cfg.edge_margin;
        if let Ok(mut node) = hp_q.single_mut() {
            node.top = Val::Px(edge);
            node.left = Val::Px(edge);
        }
        if let Ok(mut node) = level_q.single_mut() {
            node.top = Val::Px(edge);
        }
        if let Ok(mut node) = timer_q.single_mut() {
            node.top = Val::Px(edge);
            node.right = Val::Px(edge);
        }
        // XP bar is pinned to the bottom edge with no margin — not updated here.
    }
}
