//! Game-over screen — shown when the player's HP reaches zero.
//!
//! Spawns a full-screen layout containing a "GAME OVER" heading, run
//! statistics (survival time, level reached, enemies defeated, gold earned),
//! and two buttons: "もう一度" (retry → CharacterSelect) and "タイトルへ"
//! (title).  All entities are tagged with
//! [`DespawnOnExit`]`(`[`AppState::GameOver`]`)` so Bevy automatically
//! despawns them when the state transitions away.
//!
//! - Background color: [`UiStyleParams`]
//! - Heading layout: [`ScreenHeadingHudParams`]
//! - Button dimensions and colors: [`MenuButtonHudParams`]

use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;
use vs_core::resources::{GameData, GameSettings};
use vs_core::states::AppState;

use crate::components::ButtonAction;
use crate::config::{MenuButtonHudParams, ScreenHeadingHudParams, UiStyleParams};
use crate::hud::gameplay::timer::format_elapsed;
use crate::hud::menu_button::spawn_large_menu_button;
use crate::hud::screen_heading::spawn_screen_heading;
use crate::i18n::{font_for_lang, t};
use crate::styles::DEFAULT_BG_COLOR;

// ---------------------------------------------------------------------------
// Fallback constants
// ---------------------------------------------------------------------------

/// Red tone used for the "GAME OVER" heading.
const GAME_OVER_COLOR: Color = Color::srgb(0.8, 0.2, 0.2);
/// Muted white for stat text lines.
const DEFAULT_STAT_COLOR: Color = Color::srgb(0.85, 0.85, 0.85);
/// Font size for stat lines.
const DEFAULT_STAT_FONT_SIZE: f32 = 24.0;
/// Top margin between heading and stats container (pixels).
const DEFAULT_STATS_MARGIN_TOP: f32 = 16.0;
/// Top margin between stats and buttons (pixels).
const DEFAULT_BUTTON_MARGIN_TOP: f32 = 48.0;
/// Vertical gap between individual stat lines (pixels).
const DEFAULT_ROW_GAP: f32 = 8.0;

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

/// Marks the root background node of the game-over screen.
#[derive(Component)]
pub struct GameOverScreenBg;

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns the game-over screen UI when entering [`AppState::GameOver`].
///
/// Reads [`GameData`] to display run statistics (survival time, level
/// reached, enemies defeated, gold earned).  Visual tunables fall back to
/// the private `DEFAULT_*` constants defined above.
pub fn setup_game_over_screen(
    mut commands: Commands,
    ui_style: UiStyleParams,
    heading_cfg: ScreenHeadingHudParams,
    btn_cfg: MenuButtonHudParams,
    game_data: Res<GameData>,
    asset_server: Option<Res<AssetServer>>,
    settings: Option<Res<GameSettings>>,
) {
    let lang = settings.as_deref().map(|s| s.language).unwrap_or_default();
    let font: Handle<Font> = asset_server
        .map(|s| s.load(font_for_lang(lang)))
        .unwrap_or_default();
    let bg_color = ui_style
        .get()
        .map(|c| Color::from(&c.bg_color))
        .unwrap_or(DEFAULT_BG_COLOR);

    let clear_time = format_elapsed(game_data.elapsed_time as u32);
    let level = game_data.current_level;
    let kills = game_data.kill_count;
    let gold = game_data.gold_earned;

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(bg_color),
            DespawnOnExit(AppState::GameOver),
            GameOverScreenBg,
        ))
        .with_children(|parent| {
            // "GAME OVER" heading — red.
            spawn_screen_heading(
                parent,
                t("game_over_title", lang),
                GAME_OVER_COLOR,
                heading_cfg.get(),
                font.clone(),
            );

            // Run statistics.
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(DEFAULT_STATS_MARGIN_TOP)),
                    row_gap: Val::Px(DEFAULT_ROW_GAP),
                    ..default()
                })
                .with_children(|stats| {
                    for line in [
                        format!("{} {clear_time}", t("stat_clear_time", lang)),
                        format!("{} {level}", t("stat_level_reached", lang)),
                        format!("{} {kills}", t("stat_enemies_defeated", lang)),
                        format!("{} {gold}", t("stat_gold_earned", lang)),
                    ] {
                        stats.spawn((
                            Text::new(line),
                            TextFont {
                                font: font.clone(),
                                font_size: DEFAULT_STAT_FONT_SIZE,
                                ..default()
                            },
                            TextColor(DEFAULT_STAT_COLOR),
                        ));
                    }
                });

            // Button row.
            parent.spawn(Node {
                margin: UiRect::top(Val::Px(DEFAULT_BUTTON_MARGIN_TOP)),
                ..default()
            });

            // "もう一度" → CharacterSelect (restart run).
            spawn_large_menu_button(
                parent,
                t("btn_retry", lang),
                ButtonAction::GoToCharacterSelect,
                btn_cfg.get(),
                font.clone(),
                Some("btn_retry"),
            );

            // "タイトルへ" → Title.
            spawn_large_menu_button(
                parent,
                t("btn_to_title", lang),
                ButtonAction::GoToTitle,
                btn_cfg.get(),
                font.clone(),
                Some("btn_to_title"),
            );
        });
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::components::MenuButton;
    use crate::hud::menu_button::LargeMenuButtonHud;
    use crate::hud::screen_heading::ScreenHeadingHud;

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<AppState>();
        app.insert_resource(GameData::default());
        app
    }

    fn enter_game_over(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::GameOver);
        app.update();
        app.update();
    }

    #[test]
    fn setup_game_over_screen_spawns_nodes() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::GameOver), setup_game_over_screen);
        enter_game_over(&mut app);

        let mut q = app.world_mut().query_filtered::<Entity, With<Node>>();
        assert!(
            q.iter(app.world()).count() > 0,
            "game-over screen must spawn at least one Node"
        );
    }

    #[test]
    fn setup_game_over_screen_has_two_buttons() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::GameOver), setup_game_over_screen);
        enter_game_over(&mut app);

        let mut q = app.world_mut().query_filtered::<Entity, With<Button>>();
        assert_eq!(
            q.iter(app.world()).count(),
            2,
            "game-over screen should have exactly two buttons"
        );
    }

    #[test]
    fn game_over_screen_has_retry_and_title_buttons() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::GameOver), setup_game_over_screen);
        enter_game_over(&mut app);

        let mut q = app.world_mut().query::<&MenuButton>();
        let actions: Vec<ButtonAction> = q.iter(app.world()).map(|b| b.action).collect();
        assert!(
            actions.contains(&ButtonAction::GoToCharacterSelect),
            "game-over screen must have a GoToCharacterSelect (retry) button"
        );
        assert!(
            actions.contains(&ButtonAction::GoToTitle),
            "game-over screen must have a GoToTitle button"
        );
    }

    #[test]
    fn game_over_screen_bg_has_marker_component() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::GameOver), setup_game_over_screen);
        enter_game_over(&mut app);

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<GameOverScreenBg>>();
        assert_eq!(
            q.iter(app.world()).count(),
            1,
            "exactly one GameOverScreenBg expected"
        );
    }

    #[test]
    fn game_over_screen_despawns_on_exit() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::GameOver), setup_game_over_screen);
        enter_game_over(&mut app);

        {
            let mut q = app
                .world_mut()
                .query_filtered::<Entity, With<GameOverScreenBg>>();
            assert_eq!(q.iter(app.world()).count(), 1);
        }

        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Title);
        app.update();
        app.update();

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<GameOverScreenBg>>();
        assert_eq!(
            q.iter(app.world()).count(),
            0,
            "game-over nodes must despawn after leaving GameOver state"
        );
    }

    #[test]
    fn game_over_screen_has_two_hud_buttons() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::GameOver), setup_game_over_screen);
        enter_game_over(&mut app);

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<LargeMenuButtonHud>>();
        assert_eq!(
            q.iter(app.world()).count(),
            2,
            "exactly two LargeMenuButtonHud expected"
        );
    }

    #[test]
    fn game_over_screen_heading_has_hud_marker() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::GameOver), setup_game_over_screen);
        enter_game_over(&mut app);

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<ScreenHeadingHud>>();
        assert_eq!(
            q.iter(app.world()).count(),
            1,
            "exactly one ScreenHeadingHud expected"
        );
    }

    #[test]
    fn stats_reflect_game_data() {
        let mut app = build_app();
        {
            let mut gd = app.world_mut().resource_mut::<GameData>();
            gd.elapsed_time = 1865.0; // 31:05
            gd.current_level = 42;
            gd.kill_count = 777;
            gd.gold_earned = 999;
        }
        app.add_systems(OnEnter(AppState::GameOver), setup_game_over_screen);
        enter_game_over(&mut app);

        let mut q = app.world_mut().query::<&Text>();
        let texts: Vec<String> = q.iter(app.world()).map(|t| t.0.clone()).collect();

        assert!(
            texts.iter().any(|t| t.contains("31:05")),
            "survival time '31:05' should appear; got: {texts:?}"
        );
        assert!(
            texts.iter().any(|t| t.contains("42")),
            "level 42 should appear; got: {texts:?}"
        );
        assert!(
            texts.iter().any(|t| t.contains("777")),
            "kill count 777 should appear; got: {texts:?}"
        );
        assert!(
            texts.iter().any(|t| t.contains("999")),
            "gold 999 should appear; got: {texts:?}"
        );
    }
}
