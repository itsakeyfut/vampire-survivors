//! Title screen — the first screen the player sees when the game starts.
//!
//! Spawns a full-screen layout containing the game title and a Start button.
//! All entities are tagged with [`DespawnOnExit`]`(AppState::Title)` so Bevy
//! automatically despawns them when the state transitions away from `Title`.

use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;
use vs_core::states::AppState;

use crate::components::{ButtonAction, MenuButton};
use crate::styles::{
    BG_COLOR, BUTTON_LARGE_HEIGHT, BUTTON_LARGE_WIDTH, BUTTON_NORMAL, FONT_SIZE_HUGE,
    FONT_SIZE_LARGE, TEXT_COLOR, TITLE_COLOR,
};

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns the title screen UI when entering [`AppState::Title`].
pub fn setup_title_screen(mut commands: Commands) {
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
            BackgroundColor(BG_COLOR),
            DespawnOnExit(AppState::Title),
        ))
        .with_children(|parent| {
            // Game title
            parent.spawn((
                Text::new("Vampire Survivors"),
                TextFont {
                    font_size: FONT_SIZE_HUGE,
                    ..default()
                },
                TextColor(TITLE_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(80.0)),
                    ..default()
                },
            ));

            // Start button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(BUTTON_LARGE_WIDTH),
                        height: Val::Px(BUTTON_LARGE_HEIGHT),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(BUTTON_NORMAL),
                    MenuButton {
                        action: ButtonAction::StartGame,
                    },
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Start"),
                        TextFont {
                            font_size: FONT_SIZE_LARGE,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
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
}
