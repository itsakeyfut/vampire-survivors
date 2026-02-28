//! Game-over screen — shown when the player's HP reaches zero.
//!
//! Spawns a full-screen layout containing:
//! - A **GAME OVER** heading in red
//! - A **Title** button that returns to [`AppState::Title`]
//!
//! All entities are tagged with [`DespawnOnExit`]`(`[`AppState::GameOver`]`)`
//! so Bevy automatically despawns them when the state transitions away.

use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;
use vs_core::states::AppState;

use crate::components::{ButtonAction, MenuButton};
use crate::config::UiStyleParams;
use crate::styles::{
    DEFAULT_BG_COLOR, DEFAULT_BUTTON_LARGE_HEIGHT, DEFAULT_BUTTON_LARGE_WIDTH,
    DEFAULT_BUTTON_NORMAL, DEFAULT_FONT_SIZE_HUGE, DEFAULT_FONT_SIZE_LARGE, DEFAULT_TEXT_COLOR,
};

// ---------------------------------------------------------------------------
// Color constants (local to this screen)
// ---------------------------------------------------------------------------

/// Red tone used for the "GAME OVER" heading.
const GAME_OVER_COLOR: Color = Color::srgb(0.8, 0.2, 0.2);

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

/// Marks the root background node of the game-over screen.
#[derive(Component)]
pub struct GameOverScreenBg;

/// Marks the "GAME OVER" heading text node.
#[derive(Component)]
pub struct GameOverHeadingText;

/// Marks the "Title" button on the game-over screen.
#[derive(Component)]
pub struct GameOverTitleButton;

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns the game-over screen UI when entering [`AppState::GameOver`].
pub fn setup_game_over_screen(mut commands: Commands, ui_style: UiStyleParams) {
    let bg_color = ui_style
        .get()
        .map(|c| Color::from(&c.bg_color))
        .unwrap_or(DEFAULT_BG_COLOR);
    let text_color = ui_style
        .get()
        .map(|c| Color::from(&c.text_color))
        .unwrap_or(DEFAULT_TEXT_COLOR);
    let button_normal = ui_style
        .get()
        .map(|c| Color::from(&c.button_normal))
        .unwrap_or(DEFAULT_BUTTON_NORMAL);
    let font_size_huge = ui_style
        .get()
        .map(|c| c.font_size_huge)
        .unwrap_or(DEFAULT_FONT_SIZE_HUGE);
    let font_size_large = ui_style
        .get()
        .map(|c| c.font_size_large)
        .unwrap_or(DEFAULT_FONT_SIZE_LARGE);
    let button_width = ui_style
        .get()
        .map(|c| c.button_large_width)
        .unwrap_or(DEFAULT_BUTTON_LARGE_WIDTH);
    let button_height = ui_style
        .get()
        .map(|c| c.button_large_height)
        .unwrap_or(DEFAULT_BUTTON_LARGE_HEIGHT);

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
            // "GAME OVER" heading
            parent.spawn((
                Text::new("GAME OVER"),
                TextFont {
                    font_size: font_size_huge,
                    ..default()
                },
                TextColor(GAME_OVER_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(80.0)),
                    ..default()
                },
                GameOverHeadingText,
            ));

            // Title button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(button_width),
                        height: Val::Px(button_height),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(button_normal),
                    MenuButton {
                        action: ButtonAction::GoToTitle,
                    },
                    GameOverTitleButton,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Title"),
                        TextFont {
                            font_size: font_size_large,
                            ..default()
                        },
                        TextColor(text_color),
                    ));
                });
        });
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;

    use super::*;

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<AppState>();
        app
    }

    fn enter_game_over(app: &mut App) {
        // GameOver is not the default state, so transition directly.
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::GameOver);
        app.update(); // Loading → GameOver (fires OnEnter(GameOver))
        app.update(); // let spawned nodes settle
    }

    #[test]
    fn setup_game_over_screen_spawns_nodes() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::GameOver), setup_game_over_screen);

        enter_game_over(&mut app);

        let mut node_q = app.world_mut().query_filtered::<Entity, With<Node>>();
        assert!(
            node_q.iter(app.world()).count() > 0,
            "game-over screen should spawn at least one Node"
        );
    }

    #[test]
    fn setup_game_over_screen_has_exactly_one_button() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::GameOver), setup_game_over_screen);

        enter_game_over(&mut app);

        let mut q = app.world_mut().query_filtered::<Entity, With<Button>>();
        assert_eq!(
            q.iter(app.world()).count(),
            1,
            "game-over screen should have exactly one button"
        );
    }

    #[test]
    fn title_button_has_go_to_title_action() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::GameOver), setup_game_over_screen);

        enter_game_over(&mut app);

        let mut q = app.world_mut().query::<&MenuButton>();
        let btn = q.single(app.world()).expect("MenuButton should exist");
        assert_eq!(btn.action, ButtonAction::GoToTitle);
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
            "exactly one GameOverScreenBg entity expected"
        );
    }

    #[test]
    fn game_over_screen_despawns_on_exit() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::GameOver), setup_game_over_screen);

        enter_game_over(&mut app);

        // Verify nodes are present in GameOver state.
        {
            let mut q = app
                .world_mut()
                .query_filtered::<Entity, With<GameOverScreenBg>>();
            assert_eq!(q.iter(app.world()).count(), 1);
        }

        // Transition to Title — DespawnOnExit(GameOver) should clean up.
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
            "game-over nodes should be despawned after leaving GameOver state"
        );
    }
}
