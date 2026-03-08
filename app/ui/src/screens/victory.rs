//! Victory screen — shown when the player defeats Boss Death.
//!
//! Spawns a full-screen layout containing a "YOU WIN!" heading, run statistics
//! (clear time, level reached, enemies defeated), and a button to return to the
//! title.  All entities are tagged with [`DespawnOnExit`]`(`[`AppState::Victory`]`)`
//! so Bevy automatically despawns them when the state transitions away.
//!
//! - Background color: [`UiStyleParams`]
//! - Heading layout: [`ScreenHeadingHudParams`]
//! - Button dimensions and colors: [`MenuButtonHudParams`]
//! - Victory-specific colors and spacing: [`VictoryScreenParams`]

use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;
use vs_core::resources::{GameData, GameSettings};
use vs_core::states::AppState;

use crate::components::ButtonAction;
use crate::config::{
    MenuButtonHudParams, ScreenHeadingHudParams, UiStyleParams, VictoryScreenParams,
};
use crate::hud::gameplay::timer::format_elapsed;
use crate::hud::menu_button::spawn_large_menu_button;
use crate::hud::screen_heading::spawn_screen_heading;
use crate::i18n::{font_for_lang, t};

// ---------------------------------------------------------------------------
// Fallback constants (used when VictoryScreenConfig / UiStyleConfig not loaded)
// ---------------------------------------------------------------------------

/// Near-black background, matching the shared screen palette.
const DEFAULT_BG_COLOR: Color = Color::srgb(0.05, 0.05, 0.08);
/// Gold tone for the "YOU WIN!" heading.
const DEFAULT_VICTORY_COLOR: Color = Color::srgb(1.0, 0.85, 0.1);
/// Muted white for stat text lines.
const DEFAULT_STAT_COLOR: Color = Color::srgb(0.85, 0.85, 0.85);
/// Font size for stat lines.
const DEFAULT_STAT_FONT_SIZE: f32 = 24.0;
/// Top margin between heading and stats container (pixels).
const DEFAULT_STATS_MARGIN_TOP: f32 = 16.0;
/// Top margin between stats and button (pixels).
const DEFAULT_BUTTON_MARGIN_TOP: f32 = 48.0;
/// Vertical gap between individual stat lines (pixels).
const DEFAULT_ROW_GAP: f32 = 8.0;

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

/// Marks the root background node of the victory screen.
#[derive(Component)]
pub struct VictoryScreenBg;

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns the victory screen UI when entering [`AppState::Victory`].
///
/// Reads [`GameData`] to display the run statistics (clear time, level
/// reached, enemies defeated).  Visual tunables are loaded from
/// `config/ui/screen/victory.ron` via [`VictoryScreenParams`]; the private
/// `DEFAULT_*` constants above serve as fallbacks while the file loads.
#[allow(clippy::too_many_arguments)]
pub fn setup_victory_screen(
    mut commands: Commands,
    ui_style: UiStyleParams,
    heading_cfg: ScreenHeadingHudParams,
    btn_cfg: MenuButtonHudParams,
    victory_cfg: VictoryScreenParams,
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
    let victory_color = victory_cfg
        .get()
        .map(|c| Color::from(&c.victory_color))
        .unwrap_or(DEFAULT_VICTORY_COLOR);
    let stat_color = victory_cfg
        .get()
        .map(|c| Color::from(&c.stat_color))
        .unwrap_or(DEFAULT_STAT_COLOR);
    let stat_font_size = victory_cfg
        .get()
        .map(|c| c.stat_font_size)
        .unwrap_or(DEFAULT_STAT_FONT_SIZE);
    let stats_margin_top = victory_cfg
        .get()
        .map(|c| c.stats_margin_top)
        .unwrap_or(DEFAULT_STATS_MARGIN_TOP);
    let button_margin_top = victory_cfg
        .get()
        .map(|c| c.button_margin_top)
        .unwrap_or(DEFAULT_BUTTON_MARGIN_TOP);
    let row_gap = victory_cfg
        .get()
        .map(|c| c.row_gap)
        .unwrap_or(DEFAULT_ROW_GAP);

    let clear_time = format_elapsed(game_data.elapsed_time as u32);
    let level = game_data.current_level;
    let kills = game_data.kill_count;

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
            DespawnOnExit(AppState::Victory),
            VictoryScreenBg,
        ))
        .with_children(|parent| {
            // "YOU WIN!" heading — gold.
            spawn_screen_heading(
                parent,
                t("victory_title", lang),
                victory_color,
                heading_cfg.get(),
                font.clone(),
            );

            // Run statistics.
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(stats_margin_top)),
                    row_gap: Val::Px(row_gap),
                    ..default()
                })
                .with_children(|stats| {
                    for line in [
                        format!("Clear Time: {clear_time}"),
                        format!("Level Reached: {level}"),
                        format!("Enemies Defeated: {kills}"),
                    ] {
                        stats.spawn((
                            Text::new(line),
                            TextFont {
                                font: font.clone(),
                                font_size: stat_font_size,
                                ..default()
                            },
                            TextColor(stat_color),
                        ));
                    }
                });

            // Title button.
            parent.spawn(Node {
                margin: UiRect::top(Val::Px(button_margin_top)),
                ..default()
            });
            spawn_large_menu_button(
                parent,
                t("btn_to_title", lang),
                ButtonAction::GoToTitle,
                btn_cfg.get(),
                font.clone(),
                None,
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

    fn enter_victory(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Victory);
        app.update();
        app.update();
    }

    #[test]
    fn setup_victory_screen_spawns_nodes() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Victory), setup_victory_screen);
        enter_victory(&mut app);

        let mut q = app.world_mut().query_filtered::<Entity, With<Node>>();
        assert!(
            q.iter(app.world()).count() > 0,
            "victory screen must spawn at least one Node"
        );
    }

    #[test]
    fn setup_victory_screen_has_exactly_one_button() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Victory), setup_victory_screen);
        enter_victory(&mut app);

        let mut q = app.world_mut().query_filtered::<Entity, With<Button>>();
        assert_eq!(
            q.iter(app.world()).count(),
            1,
            "victory screen should have exactly one button"
        );
    }

    #[test]
    fn title_button_has_go_to_title_action() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Victory), setup_victory_screen);
        enter_victory(&mut app);

        let mut q = app.world_mut().query::<&MenuButton>();
        let btn = q.single(app.world()).expect("MenuButton should exist");
        assert_eq!(btn.action, ButtonAction::GoToTitle);
    }

    #[test]
    fn victory_screen_bg_has_marker_component() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Victory), setup_victory_screen);
        enter_victory(&mut app);

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<VictoryScreenBg>>();
        assert_eq!(
            q.iter(app.world()).count(),
            1,
            "exactly one VictoryScreenBg expected"
        );
    }

    #[test]
    fn victory_screen_despawns_on_exit() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Victory), setup_victory_screen);
        enter_victory(&mut app);

        {
            let mut q = app
                .world_mut()
                .query_filtered::<Entity, With<VictoryScreenBg>>();
            assert_eq!(q.iter(app.world()).count(), 1);
        }

        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Title);
        app.update();
        app.update();

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<VictoryScreenBg>>();
        assert_eq!(
            q.iter(app.world()).count(),
            0,
            "victory nodes must despawn after leaving Victory state"
        );
    }

    #[test]
    fn victory_screen_heading_has_hud_marker() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Victory), setup_victory_screen);
        enter_victory(&mut app);

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
    fn victory_screen_button_has_hud_marker() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Victory), setup_victory_screen);
        enter_victory(&mut app);

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<LargeMenuButtonHud>>();
        assert_eq!(
            q.iter(app.world()).count(),
            1,
            "exactly one LargeMenuButtonHud expected"
        );
    }

    #[test]
    fn stats_reflect_game_data() {
        let mut app = build_app();
        // Set specific GameData values to verify they appear in text.
        {
            let mut gd = app.world_mut().resource_mut::<GameData>();
            gd.elapsed_time = 1865.0; // 31:05
            gd.current_level = 42;
            gd.kill_count = 777;
        }
        app.add_systems(OnEnter(AppState::Victory), setup_victory_screen);
        enter_victory(&mut app);

        // Collect all Text values and search for expected content.
        let mut q = app.world_mut().query::<&Text>();
        let texts: Vec<String> = q.iter(app.world()).map(|t| t.0.clone()).collect();

        assert!(
            texts.iter().any(|t| t.contains("31:05")),
            "clear time '31:05' should appear in a Text node; got: {texts:?}"
        );
        assert!(
            texts.iter().any(|t| t.contains("42")),
            "level 42 should appear in a Text node; got: {texts:?}"
        );
        assert!(
            texts.iter().any(|t| t.contains("777")),
            "kill count 777 should appear in a Text node; got: {texts:?}"
        );
    }
}
