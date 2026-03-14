//! World-space boss HP bar.
//!
//! Spawns a label + track + fill as child sprites of the boss entity, so the
//! bar moves with the boss automatically.  The bar is created the first frame a
//! [`BossPhase`] entity appears and is cleaned up automatically when the boss
//! entity is despawned.
//!
//! ```text
//!         DEATH          ← Text2d label (world space)
//!  ████████████░░░░░░    ← track sprite + fill sprite
//! ```
//!
//! ## System overview
//!
//! | System                    | Runs when                        | Effect                               |
//! |---------------------------|----------------------------------|--------------------------------------|
//! | [`maybe_spawn_boss_hp_bar`] | Every frame while `Playing`    | One-shot: spawns bar children once   |
//! | [`update_boss_hp_bar_world`] | Every frame while `Playing`   | Updates fill width to match HP %     |

use bevy::prelude::*;
use vs_core::components::Enemy;
use vs_core::types::BossPhase;

use crate::config::hud::gameplay::boss_hp_bar::BossHpBarHudConfigHandle;
use crate::config::hud::gameplay::BossHpBarHudConfig;

// ---------------------------------------------------------------------------
// Fallback constants
// ---------------------------------------------------------------------------

const DEFAULT_LABEL_TEXT: &str = "DEATH";
const DEFAULT_BAR_WIDTH: f32 = 160.0;
const DEFAULT_BAR_HEIGHT: f32 = 8.0;
const DEFAULT_LABEL_FONT_SIZE: f32 = 14.0;
const DEFAULT_LABEL_GAP: f32 = 4.0;
const DEFAULT_Y_OFFSET: f32 = -90.0;
const DEFAULT_FILL_COLOR: Color = Color::srgb(0.65, 0.10, 0.85);
const DEFAULT_TRACK_COLOR: Color = Color::srgb(0.15, 0.05, 0.20);
const DEFAULT_TEXT_COLOR: Color = Color::srgb(0.95, 0.90, 0.85);

/// Z offset of the bar track relative to the boss entity.
const Z_TRACK: f32 = 1.0;
/// Z offset of the fill sprite (above track).
const Z_FILL: f32 = 1.5;
/// Z offset of the label text (above both).
const Z_LABEL: f32 = 2.0;

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

/// Placed on the boss entity after the HP bar children have been spawned.
///
/// Used by [`maybe_spawn_boss_hp_bar`] to avoid re-spawning the bar on every frame.
#[derive(Component, Debug)]
pub struct BossHpBarAttached;

/// Marks the background track sprite child.
///
/// Used by [`hot_reload_boss_hp_bar_hud`] to update the track color.
#[derive(Component, Debug)]
pub struct BossHpBarTrack;

/// Marks the name label [`Text2d`] child.
///
/// Used by [`hot_reload_boss_hp_bar_hud`] to update the label color and font size.
#[derive(Component, Debug)]
pub struct BossHpBarLabel;

/// Placed on the fill sprite child.
///
/// Stores the full-width reference so [`update_boss_hp_bar_world`] can scale
/// the sprite proportionally to current HP.
#[derive(Component, Debug)]
pub struct BossHpBarFill {
    /// Full track width in world pixels (= 100% HP).
    pub max_width: f32,
    /// Track height in world pixels.
    pub height: f32,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns the HP bar (track + fill + label) as children of each new boss entity.
///
/// Runs every `Playing` frame, but is a no-op after the first frame because
/// [`BossHpBarAttached`] is inserted immediately on spawn.
pub fn maybe_spawn_boss_hp_bar(
    mut commands: Commands,
    boss_q: Query<Entity, (With<BossPhase>, Without<BossHpBarAttached>)>,
    cfg: crate::config::hud::gameplay::BossHpBarHudParams<'_>,
) {
    for boss_entity in boss_q.iter() {
        let c = cfg.get();
        let label_text = c
            .map(|c| c.label_text.as_str())
            .unwrap_or(DEFAULT_LABEL_TEXT);
        let bar_width = c.map(|c| c.bar_width).unwrap_or(DEFAULT_BAR_WIDTH);
        let bar_height = c.map(|c| c.bar_height).unwrap_or(DEFAULT_BAR_HEIGHT);
        let label_font_size = c
            .map(|c| c.label_font_size)
            .unwrap_or(DEFAULT_LABEL_FONT_SIZE);
        let label_gap = c.map(|c| c.label_gap).unwrap_or(DEFAULT_LABEL_GAP);
        let y_offset = c.map(|c| c.y_offset).unwrap_or(DEFAULT_Y_OFFSET);
        let fill_color: Color = c
            .map(|c| Color::from(&c.fill_color))
            .unwrap_or(DEFAULT_FILL_COLOR);
        let track_color: Color = c
            .map(|c| Color::from(&c.track_color))
            .unwrap_or(DEFAULT_TRACK_COLOR);
        let text_color: Color = c
            .map(|c| Color::from(&c.text_color))
            .unwrap_or(DEFAULT_TEXT_COLOR);

        // Label sits above the track: bottom of text ≈ top of track + label_gap.
        let label_y = y_offset + bar_height / 2.0 + label_gap + label_font_size * 0.6;

        commands
            .entity(boss_entity)
            .insert(BossHpBarAttached)
            .with_children(|parent| {
                // Background track.
                parent.spawn((
                    Sprite {
                        custom_size: Some(Vec2::new(bar_width, bar_height)),
                        color: track_color,
                        ..default()
                    },
                    Transform::from_xyz(0.0, y_offset, Z_TRACK),
                    BossHpBarTrack,
                ));

                // Fill sprite — starts at 100%, scaled down by update_boss_hp_bar_world.
                parent.spawn((
                    Sprite {
                        custom_size: Some(Vec2::new(bar_width, bar_height)),
                        color: fill_color,
                        ..default()
                    },
                    Transform::from_xyz(0.0, y_offset, Z_FILL),
                    BossHpBarFill {
                        max_width: bar_width,
                        height: bar_height,
                    },
                ));

                // Name label above the track.
                parent.spawn((
                    Text2d::new(label_text),
                    TextFont {
                        font_size: label_font_size,
                        ..default()
                    },
                    TextColor(text_color),
                    Transform::from_xyz(0.0, label_y, Z_LABEL),
                    BossHpBarLabel,
                ));
            });
    }
}

/// Updates the fill sprite width each frame to reflect the boss's current HP.
///
/// The fill is left-aligned within the track: `Transform.translation.x` is
/// shifted so the left edge stays fixed while the right edge shrinks.
pub fn update_boss_hp_bar_world(
    boss_q: Query<&Enemy, With<BossPhase>>,
    mut fill_q: Query<(&BossHpBarFill, &mut Sprite, &mut Transform)>,
) {
    let Ok(enemy) = boss_q.single() else {
        return;
    };

    let pct = if enemy.max_hp > 0.0 {
        (enemy.current_hp / enemy.max_hp).clamp(0.0, 1.0)
    } else {
        0.0
    };

    for (fill, mut sprite, mut tf) in fill_q.iter_mut() {
        let new_width = fill.max_width * pct;
        sprite.custom_size = Some(Vec2::new(new_width, fill.height));
        // Keep the left edge pinned to the track's left edge.
        tf.translation.x = -fill.max_width / 2.0 + new_width / 2.0;
    }
}

// ---------------------------------------------------------------------------
// Hot-reload system
// ---------------------------------------------------------------------------

/// Updates boss HP bar appearance when `config/ui/hud/gameplay/boss_hp_bar.ron`
/// is loaded or modified.
///
/// Applies new colors and font size to all existing bar children.  Geometry
/// (bar width / height / y-offset) is intentionally not updated at runtime
/// because the bar is world-space and resizing already-spawned sprites would
/// require re-computing child transforms.
pub fn hot_reload_boss_hp_bar_hud(
    mut events: MessageReader<AssetEvent<BossHpBarHudConfig>>,
    cfg_assets: Res<Assets<BossHpBarHudConfig>>,
    cfg_handle: Option<Res<BossHpBarHudConfigHandle>>,
    mut fill_q: Query<&mut Sprite, (With<BossHpBarFill>, Without<BossHpBarTrack>)>,
    mut track_q: Query<&mut Sprite, (With<BossHpBarTrack>, Without<BossHpBarFill>)>,
    mut label_q: Query<(&mut TextColor, &mut TextFont), With<BossHpBarLabel>>,
) {
    let Some(cfg_handle) = cfg_handle else {
        return;
    };

    let mut needs_apply = false;
    for event in events.read() {
        match event {
            AssetEvent::Added { .. } => {
                info!("Boss HP bar HUD config loaded");
                needs_apply = true;
            }
            AssetEvent::Modified { .. } => {
                info!("Boss HP bar HUD config hot-reloaded");
                needs_apply = true;
            }
            AssetEvent::Removed { .. } => {
                warn!("Boss HP bar HUD config removed");
            }
            _ => {}
        }
    }

    if needs_apply && let Some(cfg) = cfg_assets.get(&cfg_handle.0) {
        let fill_color: Color = Color::from(&cfg.fill_color);
        let track_color: Color = Color::from(&cfg.track_color);
        let text_color: Color = Color::from(&cfg.text_color);

        for mut sprite in fill_q.iter_mut() {
            sprite.color = fill_color;
        }
        for mut sprite in track_q.iter_mut() {
            sprite.color = track_color;
        }
        for (mut tc, mut font) in label_q.iter_mut() {
            *tc = TextColor(text_color);
            font.font_size = cfg.label_font_size;
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::ecs::system::RunSystemOnce as _;

    use super::*;
    use vs_core::types::EnemyType;

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app
    }

    fn spawn_fill(app: &mut App, max_width: f32) -> Entity {
        app.world_mut()
            .spawn((
                Sprite {
                    custom_size: Some(Vec2::new(max_width, DEFAULT_BAR_HEIGHT)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, Z_FILL),
                BossHpBarFill {
                    max_width,
                    height: DEFAULT_BAR_HEIGHT,
                },
            ))
            .id()
    }

    fn spawn_boss(app: &mut App, current_hp: f32, max_hp: f32) {
        let mut enemy = vs_core::components::Enemy::from_type(EnemyType::BossDeath, 1.0);
        enemy.current_hp = current_hp;
        enemy.max_hp = max_hp;
        app.world_mut().spawn((enemy, BossPhase::Phase1));
    }

    fn advance(app: &mut App, secs: f32) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(secs));
    }

    /// Fill is at 100% width when boss is at full HP.
    #[test]
    fn full_hp_fills_bar_100_percent() {
        let mut app = build_app();
        let fill = spawn_fill(&mut app, 160.0);
        spawn_boss(&mut app, 5000.0, 5000.0);
        advance(&mut app, 1.0 / 60.0);

        app.world_mut()
            .run_system_once(update_boss_hp_bar_world)
            .unwrap();

        let node = app.world().get::<Sprite>(fill).unwrap();
        assert_eq!(node.custom_size.unwrap().x, 160.0);
        let tf = app.world().get::<Transform>(fill).unwrap();
        assert!(
            (tf.translation.x).abs() < 1e-5,
            "fill x should be 0 at 100%"
        );
    }

    /// Fill reflects 50% HP correctly.
    #[test]
    fn half_hp_fills_bar_50_percent() {
        let mut app = build_app();
        let fill = spawn_fill(&mut app, 160.0);
        spawn_boss(&mut app, 2500.0, 5000.0);
        advance(&mut app, 1.0 / 60.0);

        app.world_mut()
            .run_system_once(update_boss_hp_bar_world)
            .unwrap();

        let sprite = app.world().get::<Sprite>(fill).unwrap();
        assert!((sprite.custom_size.unwrap().x - 80.0).abs() < 1e-5);
        // Left-align: center of 80px fill within 160px track → x = -40
        let tf = app.world().get::<Transform>(fill).unwrap();
        assert!((tf.translation.x - (-40.0)).abs() < 1e-5);
    }

    /// Fill is zero width when boss HP is 0.
    #[test]
    fn zero_hp_fills_bar_0_percent() {
        let mut app = build_app();
        let fill = spawn_fill(&mut app, 160.0);
        spawn_boss(&mut app, 0.0, 5000.0);
        advance(&mut app, 1.0 / 60.0);

        app.world_mut()
            .run_system_once(update_boss_hp_bar_world)
            .unwrap();

        let sprite = app.world().get::<Sprite>(fill).unwrap();
        assert!((sprite.custom_size.unwrap().x).abs() < 1e-5);
    }

    /// Overheal is clamped to 100%.
    #[test]
    fn overheal_clamped_to_100_percent() {
        let mut app = build_app();
        let fill = spawn_fill(&mut app, 160.0);
        spawn_boss(&mut app, 9999.0, 5000.0);
        advance(&mut app, 1.0 / 60.0);

        app.world_mut()
            .run_system_once(update_boss_hp_bar_world)
            .unwrap();

        let sprite = app.world().get::<Sprite>(fill).unwrap();
        assert!((sprite.custom_size.unwrap().x - 160.0).abs() < 1e-5);
    }

    /// No boss → update_boss_hp_bar_world returns early without panicking.
    #[test]
    fn no_boss_does_not_panic() {
        let mut app = build_app();
        spawn_fill(&mut app, 160.0);
        advance(&mut app, 1.0 / 60.0);

        // Must not panic.
        app.world_mut()
            .run_system_once(update_boss_hp_bar_world)
            .unwrap();
    }
}
