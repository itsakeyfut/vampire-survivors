//! Game-over screen — shown when the player's HP reaches zero.
//!
//! Spawns a full-screen layout containing a screen heading and a large menu
//! button, composed from HUD widget functions.  All entities are tagged with
//! [`DespawnOnExit`]`(`[`AppState::GameOver`]`)` so Bevy automatically
//! despawns them when the state transitions away.
//!
//! - Background color: [`UiStyleParams`]
//! - Heading layout: [`ScreenHeadingHudParams`]
//! - Button dimensions and colors: [`MenuButtonHudParams`]

use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;
use vs_core::states::AppState;

use crate::components::ButtonAction;
use crate::config::{MenuButtonHudParams, ScreenHeadingHudParams, UiStyleParams};
use crate::hud::menu_button::spawn_large_menu_button;
use crate::hud::screen_heading::spawn_screen_heading;
use crate::styles::DEFAULT_BG_COLOR;

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

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns the game-over screen UI when entering [`AppState::GameOver`].
pub fn setup_game_over_screen(
    mut commands: Commands,
    ui_style: UiStyleParams,
    heading_cfg: ScreenHeadingHudParams,
    btn_cfg: MenuButtonHudParams,
) {
    let bg_color = ui_style
        .get()
        .map(|c| Color::from(&c.bg_color))
        .unwrap_or(DEFAULT_BG_COLOR);

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
            // "GAME OVER" heading — red; color is screen-specific.
            spawn_screen_heading(parent, "GAME OVER", GAME_OVER_COLOR, heading_cfg.get());

            // Title button.
            spawn_large_menu_button(parent, "Title", ButtonAction::GoToTitle, btn_cfg.get());
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
    fn game_over_screen_button_has_hud_marker() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::GameOver), setup_game_over_screen);
        enter_game_over(&mut app);

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
}
