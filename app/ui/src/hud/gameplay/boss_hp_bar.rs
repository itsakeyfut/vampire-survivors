//! Boss HP bar widget.
//!
//! Displays Boss Death's current HP as a centered horizontal bar near the
//! bottom of the screen.  The widget is spawned as part of the gameplay HUD
//! but stays invisible until a boss entity exists in the world.
//!
//! ```text
//!        ── BOSS ──
//!  ████████████████████████████░░░░░   (400 × 20 px bar)
//! ```
//!
//! The bar is hidden (`Visibility::Hidden`) while no [`BossPhase`] entity
//! exists and made visible (`Visibility::Visible`) by [`update_boss_hp_bar`]
//! as soon as the boss spawns.

use bevy::prelude::*;
use vs_core::components::Enemy;
use vs_core::types::BossPhase;

// ---------------------------------------------------------------------------
// Fallback constants
// ---------------------------------------------------------------------------

const DEFAULT_BAR_WIDTH: f32 = 400.0;
const DEFAULT_BAR_HEIGHT: f32 = 20.0;
const DEFAULT_BAR_RADIUS: f32 = 4.0;
const DEFAULT_LABEL_FONT_SIZE: f32 = 14.0;
const DEFAULT_LABEL_GAP: f32 = 4.0;
/// Fill color: dark purple — distinct from the player's red HP bar.
const DEFAULT_FILL_COLOR: Color = Color::srgb(0.65, 0.10, 0.85);
const DEFAULT_TRACK_COLOR: Color = Color::srgb(0.15, 0.05, 0.20);
const DEFAULT_TEXT_COLOR: Color = Color::srgb(0.95, 0.90, 0.85);

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

/// Marks the column root of the boss HP bar widget.
///
/// [`update_boss_hp_bar`] toggles `Visibility` on this node to show or hide
/// the entire widget.
#[derive(Component, Debug)]
pub struct HudBossHpBarRoot;

/// Marks the inner fill [`Node`] of the boss HP bar.
///
/// [`update_boss_hp_bar`] queries this marker to set `node.width` each frame.
#[derive(Component, Debug)]
pub struct HudBossHpBar;

// ---------------------------------------------------------------------------
// Spawn
// ---------------------------------------------------------------------------

/// Spawns the boss HP bar (label + track + fill) as a child of `parent`.
///
/// The root node starts with [`Visibility::Hidden`]; `update_boss_hp_bar`
/// makes it visible when the boss entity exists.
pub fn spawn_boss_hp_bar(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(DEFAULT_LABEL_GAP),
                ..default()
            },
            Visibility::Hidden,
            HudBossHpBarRoot,
        ))
        .with_children(|col| {
            // "BOSS" label
            col.spawn((
                Text::new("BOSS"),
                TextFont {
                    font_size: DEFAULT_LABEL_FONT_SIZE,
                    ..default()
                },
                TextColor(DEFAULT_TEXT_COLOR),
            ));

            // Background track
            col.spawn((
                Node {
                    width: Val::Px(DEFAULT_BAR_WIDTH),
                    height: Val::Px(DEFAULT_BAR_HEIGHT),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(DEFAULT_TRACK_COLOR),
                BorderRadius::all(Val::Px(DEFAULT_BAR_RADIUS)),
            ))
            .with_children(|track| {
                // Fill bar — width updated by update_boss_hp_bar each frame.
                track.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(DEFAULT_FILL_COLOR),
                    BorderRadius::all(Val::Px(DEFAULT_BAR_RADIUS)),
                    HudBossHpBar,
                ));
            });
        });
}

// ---------------------------------------------------------------------------
// Update system
// ---------------------------------------------------------------------------

/// Shows the boss HP bar and updates its fill while Boss Death is alive.
///
/// - When a [`BossPhase`] entity exists: sets `Visibility::Visible` on the
///   root and updates the fill bar width proportional to `current_hp / max_hp`.
/// - When no boss entity exists: keeps the root hidden.
pub fn update_boss_hp_bar(
    boss_q: Query<&Enemy, With<BossPhase>>,
    mut root_q: Query<&mut Visibility, With<HudBossHpBarRoot>>,
    mut bar_q: Query<&mut Node, With<HudBossHpBar>>,
) {
    let Ok(mut visibility) = root_q.single_mut() else {
        return;
    };

    match boss_q.single() {
        Ok(enemy) => {
            *visibility = Visibility::Visible;
            if let Ok(mut node) = bar_q.single_mut() {
                let pct = if enemy.max_hp > 0.0 {
                    (enemy.current_hp / enemy.max_hp).clamp(0.0, 1.0) * 100.0
                } else {
                    0.0
                };
                node.width = Val::Percent(pct);
            }
        }
        Err(_) => {
            *visibility = Visibility::Hidden;
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
    use vs_core::types::EnemyType;

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app
    }

    fn spawn_bar(app: &mut App) -> (Entity, Entity) {
        let bar = app
            .world_mut()
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    ..default()
                },
                HudBossHpBar,
            ))
            .id();
        let root = app
            .world_mut()
            .spawn((Visibility::Hidden, HudBossHpBarRoot))
            .id();
        (root, bar)
    }

    fn spawn_boss(app: &mut App, current_hp: f32, max_hp: f32) -> Entity {
        let mut enemy = Enemy::from_type(EnemyType::BossDeath, 1.0);
        enemy.current_hp = current_hp;
        enemy.max_hp = max_hp;
        app.world_mut().spawn((enemy, BossPhase::Phase1)).id()
    }

    /// Bar is visible and at 100% when boss is at full HP.
    #[test]
    fn full_hp_shows_bar_at_100_percent() {
        let mut app = build_app();
        let (root, bar) = spawn_bar(&mut app);
        spawn_boss(&mut app, 5000.0, 5000.0);

        app.world_mut().run_system_once(update_boss_hp_bar).unwrap();

        assert_eq!(
            *app.world().get::<Visibility>(root).unwrap(),
            Visibility::Visible
        );
        assert_eq!(
            app.world().get::<Node>(bar).unwrap().width,
            Val::Percent(100.0)
        );
    }

    /// Bar fill reflects partial HP correctly.
    #[test]
    fn partial_hp_fills_bar_proportionally() {
        let mut app = build_app();
        let (_, bar) = spawn_bar(&mut app);
        spawn_boss(&mut app, 2500.0, 5000.0); // 50%

        app.world_mut().run_system_once(update_boss_hp_bar).unwrap();

        assert_eq!(
            app.world().get::<Node>(bar).unwrap().width,
            Val::Percent(50.0)
        );
    }

    /// Bar is hidden when no boss entity exists.
    #[test]
    fn no_boss_hides_bar() {
        let mut app = build_app();
        let (root, _) = spawn_bar(&mut app);

        app.world_mut().run_system_once(update_boss_hp_bar).unwrap();

        assert_eq!(
            *app.world().get::<Visibility>(root).unwrap(),
            Visibility::Hidden
        );
    }

    /// Overheal is clamped to 100%.
    #[test]
    fn hp_over_max_clamped_to_100_percent() {
        let mut app = build_app();
        let (_, bar) = spawn_bar(&mut app);
        spawn_boss(&mut app, 6000.0, 5000.0); // 120% — should clamp

        app.world_mut().run_system_once(update_boss_hp_bar).unwrap();

        assert_eq!(
            app.world().get::<Node>(bar).unwrap().width,
            Val::Percent(100.0)
        );
    }

    /// Zero HP shows 0% fill.
    #[test]
    fn zero_hp_shows_empty_bar() {
        let mut app = build_app();
        let (_, bar) = spawn_bar(&mut app);
        spawn_boss(&mut app, 0.0, 5000.0);

        app.world_mut().run_system_once(update_boss_hp_bar).unwrap();

        assert_eq!(
            app.world().get::<Node>(bar).unwrap().width,
            Val::Percent(0.0)
        );
    }
}
