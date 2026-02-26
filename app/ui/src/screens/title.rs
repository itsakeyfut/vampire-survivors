//! Title screen — the first screen the player sees when the game starts.
//!
//! Spawns a full-screen layout containing the game title and a Start button.
//! All entities are tagged with [`DespawnOnExit`]`(AppState::Title)` so Bevy
//! automatically despawns them when the state transitions away from `Title`.
//!
//! Visual properties are read from [`UiStyleParams`]; the `DEFAULT_*`
//! constants in [`crate::styles`] are used as fallbacks while the RON config
//! is still loading.

use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;
use vs_core::states::AppState;

use crate::components::{ButtonAction, MenuButton};
use crate::config::{
    TitleButtonLabel, TitleHeadingText, TitleScreenBg, TitleStartButton, UiStyleParams,
};
use crate::styles::{
    DEFAULT_BG_COLOR, DEFAULT_BUTTON_LARGE_HEIGHT, DEFAULT_BUTTON_LARGE_WIDTH,
    DEFAULT_BUTTON_NORMAL, DEFAULT_FONT_SIZE_HUGE, DEFAULT_FONT_SIZE_LARGE, DEFAULT_TEXT_COLOR,
    DEFAULT_TITLE_COLOR,
};

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns the title screen UI when entering [`AppState::Title`].
pub fn setup_title_screen(mut commands: Commands, ui_style: UiStyleParams) {
    let bg_color = ui_style
        .get()
        .map(|c| Color::from(&c.bg_color))
        .unwrap_or(DEFAULT_BG_COLOR);
    let title_color = ui_style
        .get()
        .map(|c| Color::from(&c.title_color))
        .unwrap_or(DEFAULT_TITLE_COLOR);
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
            DespawnOnExit(AppState::Title),
            TitleScreenBg,
        ))
        .with_children(|parent| {
            // Game title
            parent.spawn((
                Text::new("Vampire Survivors"),
                TextFont {
                    font_size: font_size_huge,
                    ..default()
                },
                TextColor(title_color),
                Node {
                    margin: UiRect::bottom(Val::Px(80.0)),
                    ..default()
                },
                TitleHeadingText,
            ));

            // Start button
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
                        action: ButtonAction::StartGame,
                    },
                    TitleStartButton,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Start"),
                        TextFont {
                            font_size: font_size_large,
                            ..default()
                        },
                        TextColor(text_color),
                        TitleButtonLabel,
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

    #[test]
    fn setup_title_screen_spawns_nodes() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Title), setup_title_screen);
        // Title is the default state. Transition away first so we can
        // re-enter Title and trigger OnEnter(Title).
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Playing);
        app.update(); // Title → Playing
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Title);
        app.update(); // Playing → Title (fires OnEnter(Title))
        app.update(); // let spawned nodes settle

        let mut node_q = app.world_mut().query_filtered::<Entity, With<Node>>();
        let node_count = node_q.iter(app.world()).count();
        assert!(
            node_count > 0,
            "title screen should spawn at least one Node"
        );
    }

    #[test]
    fn setup_title_screen_has_exactly_one_button() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Title), setup_title_screen);
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Playing);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Title);
        app.update();
        app.update();

        let mut button_q = app.world_mut().query_filtered::<Entity, With<Button>>();
        let button_count = button_q.iter(app.world()).count();
        assert_eq!(
            button_count, 1,
            "title screen should have exactly one button"
        );
    }

    #[test]
    fn start_button_has_start_game_action() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Title), setup_title_screen);
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Playing);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Title);
        app.update();
        app.update();

        let mut q = app.world_mut().query::<&MenuButton>();
        let btn = q.single(app.world()).expect("MenuButton should exist");
        assert_eq!(btn.action, ButtonAction::StartGame);
    }

    #[test]
    fn title_screen_bg_has_marker_component() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Title), setup_title_screen);
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Playing);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Title);
        app.update();
        app.update();

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<TitleScreenBg>>();
        assert_eq!(
            q.iter(app.world()).count(),
            1,
            "exactly one TitleScreenBg entity expected"
        );
    }

    #[test]
    fn title_screen_button_has_marker_component() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Title), setup_title_screen);
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Playing);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Title);
        app.update();
        app.update();

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<TitleStartButton>>();
        assert_eq!(
            q.iter(app.world()).count(),
            1,
            "exactly one TitleStartButton entity expected"
        );
    }
}
