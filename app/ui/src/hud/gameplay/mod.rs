//! Gameplay HUD — in-game overlay widgets.
//!
//! [`setup_gameplay_hud`] runs on [`OnEnter(AppState::Playing)`] and spawns a
//! transparent full-screen root node.  Each widget is positioned in its own
//! absolute-positioned anchor so layouts stay independent.
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │  HP 100/100          Lv. 1             0:00     │
//! │  ████████████░░░░                               │
//! │                                                 │
//! │  [ Wh ][ MW ][    ][    ][    ][    ]  Kills: 0 │
//! │░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░│ ← XP bar
//! └─────────────────────────────────────────────────┘
//! ```
//!
//! The boss HP bar is **not** part of this overlay — it is rendered in world
//! space as child sprites of the boss entity (see [`boss_hp_bar`]).
//!
//! | Module                    | Widget                   | Spawn                       | Update system                     |
//! |---------------------------|--------------------------|-----------------------------|-----------------------------------|
//! | [`hp_bar`]                | HP bar (color + number)  | `spawn_hp_bar`              | `update_hp_bar`                   |
//! | [`xp_bar`]                | XP bar                   | `spawn_xp_bar`              | `update_xp_bar`                   |
//! | [`timer`]                 | Elapsed timer            | `spawn_timer`               | `update_timer`                    |
//! | [`level`]                 | Level label              | `spawn_level`               | `update_level_text`               |
//! | [`evolution_notification`]| Evolution toast          | `on_weapon_evolved`         | `update_evolution_notification`   |
//! | [`weapon_slots`]          | 6 weapon slots           | `spawn_weapon_slots`        | `update_weapon_slots`             |
//! | [`kill_count`]            | Kill counter             | `spawn_kill_count`          | `update_kill_count`               |
//! | [`gold`]                  | Gold earned label        | `spawn_gold`                | `update_gold`                     |
//! | [`boss_hp_bar`]           | Boss HP bar (world-space)| `maybe_spawn_boss_hp_bar`   | `update_boss_hp_bar_world`        |

pub mod boss_hp_bar;
pub mod boss_warning;
pub mod evolution_notification;
pub mod gold;
pub mod hp_bar;
pub mod kill_count;
pub mod level;
pub mod timer;
pub mod weapon_slots;
pub mod xp_bar;

use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;
use vs_core::resources::GameSettings;
use vs_core::states::AppState;

use crate::config::hud::gameplay::{
    GameplayHudLayoutConfig, GameplayHudLayoutConfigHandle, GameplayHudLayoutParams, GoldHudParams,
    HpBarHudParams, KillCountHudParams, LevelHudParams, TimerHudParams, WeaponSlotsHudParams,
    XpBarHudParams,
};
use crate::i18n::font_for_lang;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Gap between the top of the XP bar and the widgets anchored just above it.
const BOTTOM_WIDGET_GAP: f32 = 8.0;

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

/// Marks the bottom-center anchor node (holds weapon slots).
#[derive(Component, Debug)]
pub struct HudWeaponSlotsAnchor;

/// Marks the bottom-right anchor node (holds the kill count).
#[derive(Component, Debug)]
pub struct HudKillCountAnchor;

/// Marks the bottom-right anchor node (holds the gold label, above kill count).
#[derive(Component, Debug)]
pub struct HudGoldAnchor;

// ---------------------------------------------------------------------------
// Setup system
// ---------------------------------------------------------------------------

/// Spawns the full gameplay HUD overlay.
///
/// Runs on [`OnEnter(AppState::Playing)`].  The root node carries
/// [`DespawnOnExit`]`(`[`AppState::Playing`]`)` so all HUD entities are
/// automatically cleaned up on every state transition away from `Playing`.
#[allow(clippy::too_many_arguments)]
pub fn setup_gameplay_hud(
    mut commands: Commands,
    layout_cfg: GameplayHudLayoutParams,
    hp_bar_cfg: HpBarHudParams,
    xp_bar_cfg: XpBarHudParams,
    timer_cfg: TimerHudParams,
    level_cfg: LevelHudParams,
    weapon_slots_cfg: WeaponSlotsHudParams,
    kill_count_cfg: KillCountHudParams,
    gold_cfg: GoldHudParams,
    asset_server: Option<Res<AssetServer>>,
    settings: Option<Res<GameSettings>>,
) {
    let lang = settings.as_deref().map(|s| s.language).unwrap_or_default();
    let font: Handle<Font> = asset_server
        .map(|s| s.load(font_for_lang(lang)))
        .unwrap_or_default();

    let edge = layout_cfg.edge_margin();
    let bottom_widget_offset = xp_bar_cfg.bar_height() + BOTTOM_WIDGET_GAP;

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
                hp_bar::spawn_hp_bar(anchor, hp_bar_cfg.get(), font.clone());
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
                level::spawn_level(anchor, level_cfg.get(), font.clone());
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
                timer::spawn_timer(anchor, timer_cfg.get(), font.clone());
            });

            // ------------------------------------------------------------------
            // Bottom-center: Weapon slots (above XP bar)
            // ------------------------------------------------------------------
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(bottom_widget_offset),
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                HudWeaponSlotsAnchor,
            ))
            .with_children(|anchor| {
                weapon_slots::spawn_weapon_slots(anchor, weapon_slots_cfg.get(), font.clone());
            });

            // ------------------------------------------------------------------
            // Bottom-right: Gold label (above kill count, above XP bar)
            // ------------------------------------------------------------------
            let gold_extra = gold_cfg.vertical_offset();
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(bottom_widget_offset + gold_extra),
                    right: Val::Px(edge),
                    ..default()
                },
                HudGoldAnchor,
            ))
            .with_children(|anchor| {
                gold::spawn_gold(anchor, gold_cfg.get(), font.clone());
            });

            // ------------------------------------------------------------------
            // Bottom-right: Kill count (above XP bar)
            // ------------------------------------------------------------------
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(bottom_widget_offset),
                    right: Val::Px(edge),
                    ..default()
                },
                HudKillCountAnchor,
            ))
            .with_children(|anchor| {
                kill_count::spawn_kill_count(anchor, kill_count_cfg.get(), font.clone());
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
#[allow(clippy::too_many_arguments)]
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
    mut gold_q: Query<
        &mut Node,
        (
            With<HudGoldAnchor>,
            Without<HudHpBarAnchor>,
            Without<HudLevelAnchor>,
            Without<HudTimerAnchor>,
        ),
    >,
    mut kill_q: Query<
        &mut Node,
        (
            With<HudKillCountAnchor>,
            Without<HudHpBarAnchor>,
            Without<HudLevelAnchor>,
            Without<HudTimerAnchor>,
            Without<HudGoldAnchor>,
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
                info!("Gameplay HUD layout config loaded");
                needs_apply = true;
            }
            AssetEvent::Modified { .. } => {
                info!("Gameplay HUD layout config hot-reloaded");
                needs_apply = true;
            }
            AssetEvent::Removed { .. } => {
                warn!("Gameplay HUD layout config removed");
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
        if let Ok(mut node) = gold_q.single_mut() {
            node.right = Val::Px(edge);
        }
        if let Ok(mut node) = kill_q.single_mut() {
            node.right = Val::Px(edge);
        }
    }
}
