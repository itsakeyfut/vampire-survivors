//! Title screen — the first screen the player sees when the game starts.
//!
//! Spawns a full-screen layout containing a screen heading and a large menu
//! button, composed from HUD widget functions.  All entities are tagged with
//! [`DespawnOnExit`]`(AppState::Title)` so Bevy automatically despawns them
//! when the state transitions away from `Title`.
//!
//! - Background and title text colour: [`UiStyleParams`]
//! - Heading font size / margin: [`ScreenHeadingHudParams`]
//! - Button dimensions and colors: [`MenuButtonHudParams`]

use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;
use vs_core::states::AppState;

use crate::components::ButtonAction;
use crate::config::{
    MenuButtonHudParams, ScreenHeadingHudParams, TitleHeadingText, TitleScreenBg, UiStyleParams,
};
use crate::hud::menu_button::spawn_large_menu_button;
use crate::hud::screen_heading::ScreenHeadingHud;
use crate::styles::{DEFAULT_BG_COLOR, DEFAULT_TITLE_COLOR};

// ---------------------------------------------------------------------------
// Fallback constants
// ---------------------------------------------------------------------------

const DEFAULT_FONT_SIZE: f32 = 72.0;
const DEFAULT_MARGIN_BOTTOM: f32 = 80.0;

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns the title screen UI when entering [`AppState::Title`].
pub fn setup_title_screen(
    mut commands: Commands,
    ui_style: UiStyleParams,
    heading_cfg: ScreenHeadingHudParams,
    btn_cfg: MenuButtonHudParams,
) {
    let bg_color = ui_style
        .get()
        .map(|c| Color::from(&c.bg_color))
        .unwrap_or(DEFAULT_BG_COLOR);
    let title_color = ui_style
        .get()
        .map(|c| Color::from(&c.title_color))
        .unwrap_or(DEFAULT_TITLE_COLOR);
    let heading_font_size = heading_cfg
        .get()
        .map(|c| c.font_size)
        .unwrap_or(DEFAULT_FONT_SIZE);
    let heading_margin = heading_cfg
        .get()
        .map(|c| c.margin_bottom)
        .unwrap_or(DEFAULT_MARGIN_BOTTOM);

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
            // Game title heading.
            // Carries both the generic HUD marker (ScreenHeadingHud) for
            // hot_reload_screen_heading_hud and the screen-specific TitleHeadingText
            // marker so hot_reload_ui_style can update the title color.
            parent.spawn((
                Text::new("Vampire Survivors"),
                TextFont {
                    font_size: heading_font_size,
                    ..default()
                },
                TextColor(title_color),
                Node {
                    margin: UiRect::bottom(Val::Px(heading_margin)),
                    ..default()
                },
                ScreenHeadingHud,
                TitleHeadingText,
            ));

            // Start button.
            spawn_large_menu_button(parent, "Start", ButtonAction::StartGame, btn_cfg.get());
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

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<AppState>();
        app
    }

    fn enter_title(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Playing);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Title);
        app.update();
        app.update();
    }

    #[test]
    fn setup_title_screen_spawns_nodes() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Title), setup_title_screen);
        enter_title(&mut app);

        let mut q = app.world_mut().query_filtered::<Entity, With<Node>>();
        assert!(
            q.iter(app.world()).count() > 0,
            "title screen must spawn at least one Node"
        );
    }

    #[test]
    fn setup_title_screen_has_exactly_one_button() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Title), setup_title_screen);
        enter_title(&mut app);

        let mut q = app.world_mut().query_filtered::<Entity, With<Button>>();
        assert_eq!(
            q.iter(app.world()).count(),
            1,
            "title screen should have exactly one button"
        );
    }

    #[test]
    fn start_button_has_start_game_action() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Title), setup_title_screen);
        enter_title(&mut app);

        let mut q = app.world_mut().query::<&MenuButton>();
        let btn = q.single(app.world()).expect("MenuButton should exist");
        assert_eq!(btn.action, ButtonAction::StartGame);
    }

    #[test]
    fn title_screen_bg_has_marker_component() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Title), setup_title_screen);
        enter_title(&mut app);

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<TitleScreenBg>>();
        assert_eq!(
            q.iter(app.world()).count(),
            1,
            "exactly one TitleScreenBg expected"
        );
    }

    #[test]
    fn title_screen_button_has_hud_marker() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Title), setup_title_screen);
        enter_title(&mut app);

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
    fn title_screen_heading_has_hud_marker() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Title), setup_title_screen);
        enter_title(&mut app);

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
