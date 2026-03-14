//! Pause screen — shown when the player presses ESC during gameplay.
//!
//! The overlay is positioned absolutely so it sits on top of the game scene
//! without destroying HUD or gameplay entities.  All entities are tagged with
//! [`DespawnOnExit`]`(`[`AppState::Paused`]`)` so Bevy automatically
//! despawns them when the state transitions away.
//!
//! Systems:
//! - [`setup_pause_screen`]: spawns the overlay when entering [`AppState::Paused`]
//! - [`toggle_pause`]: handles ESC key to switch between Playing and Paused

use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;
use vs_core::resources::GameSettings;
use vs_core::states::AppState;

use crate::components::ButtonAction;
use crate::config::{MenuButtonHudParams, PauseScreenParams, ScreenHeadingHudParams};
use crate::hud::menu_button::spawn_large_menu_button;
use crate::hud::screen_heading::spawn_screen_heading;
use crate::i18n::{font_for_lang, t};

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

/// Marks the root overlay node of the pause screen.
#[derive(Component)]
pub struct PauseScreenOverlay;

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns the pause overlay when entering [`AppState::Paused`].
pub fn setup_pause_screen(
    mut commands: Commands,
    pause_cfg: PauseScreenParams,
    heading_cfg: ScreenHeadingHudParams,
    btn_cfg: MenuButtonHudParams,
    asset_server: Option<Res<AssetServer>>,
    settings: Option<Res<GameSettings>>,
) {
    let lang = settings.as_deref().map(|s| s.language).unwrap_or_default();
    let font: Handle<Font> = asset_server
        .map(|s| s.load(font_for_lang(lang)))
        .unwrap_or_default();

    let overlay_color = pause_cfg.overlay_color();
    let heading_color = pause_cfg.heading_color();

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(overlay_color),
            ZIndex(10),
            DespawnOnExit(AppState::Paused),
            PauseScreenOverlay,
        ))
        .with_children(|parent| {
            spawn_screen_heading(
                parent,
                t("pause_title", lang),
                heading_color,
                heading_cfg.get(),
                font.clone(),
            );

            spawn_large_menu_button(
                parent,
                t("btn_resume", lang),
                ButtonAction::ResumeGame,
                btn_cfg.get(),
                font.clone(),
                Some("btn_resume"),
            );

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

/// Toggles between [`AppState::Playing`] and [`AppState::Paused`] on ESC.
pub fn toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        match state.get() {
            AppState::Playing => next_state.set(AppState::Paused),
            AppState::Paused => next_state.set(AppState::Playing),
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::components::MenuButton;
    use crate::hud::screen_heading::ScreenHeadingHud;

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<AppState>();
        app
    }

    fn enter_paused(app: &mut App) {
        // Default state is Title; go to Playing first, then Paused.
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Playing);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Paused);
        app.update();
        app.update();
    }

    #[test]
    fn setup_pause_screen_spawns_overlay() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Paused), setup_pause_screen);
        enter_paused(&mut app);

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<PauseScreenOverlay>>();
        assert_eq!(
            q.iter(app.world()).count(),
            1,
            "exactly one PauseScreenOverlay expected"
        );
    }

    #[test]
    fn setup_pause_screen_has_two_buttons() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Paused), setup_pause_screen);
        enter_paused(&mut app);

        let mut q = app.world_mut().query_filtered::<Entity, With<Button>>();
        assert_eq!(
            q.iter(app.world()).count(),
            2,
            "pause screen should have exactly two buttons"
        );
    }

    #[test]
    fn pause_screen_has_resume_and_title_buttons() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Paused), setup_pause_screen);
        enter_paused(&mut app);

        let mut q = app.world_mut().query::<&MenuButton>();
        let actions: Vec<ButtonAction> = q.iter(app.world()).map(|b| b.action).collect();
        assert!(
            actions.contains(&ButtonAction::ResumeGame),
            "pause screen must have a ResumeGame button"
        );
        assert!(
            actions.contains(&ButtonAction::GoToTitle),
            "pause screen must have a GoToTitle button"
        );
    }

    #[test]
    fn pause_screen_has_heading() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Paused), setup_pause_screen);
        enter_paused(&mut app);

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
    fn pause_screen_despawns_on_exit() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Paused), setup_pause_screen);
        enter_paused(&mut app);

        {
            let mut q = app
                .world_mut()
                .query_filtered::<Entity, With<PauseScreenOverlay>>();
            assert_eq!(q.iter(app.world()).count(), 1);
        }

        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Playing);
        app.update();
        app.update();

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<PauseScreenOverlay>>();
        assert_eq!(
            q.iter(app.world()).count(),
            0,
            "pause overlay must despawn after leaving Paused state"
        );
    }

    #[test]
    fn toggle_pause_playing_to_paused() {
        use bevy::ecs::system::RunSystemOnce as _;
        use bevy::input::InputPlugin;
        let mut app = build_app();
        app.add_plugins(InputPlugin);

        // Start in Playing state.
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Playing);
        app.update();

        // Press ESC and run toggle_pause directly (bypasses PreUpdate clear).
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.world_mut().run_system_once(toggle_pause).unwrap();
        app.update(); // apply state transition

        assert_eq!(
            *app.world().resource::<State<AppState>>(),
            AppState::Paused,
            "ESC in Playing should transition to Paused"
        );
    }

    #[test]
    fn toggle_pause_paused_to_playing() {
        use bevy::ecs::system::RunSystemOnce as _;
        use bevy::input::InputPlugin;
        let mut app = build_app();
        app.add_plugins(InputPlugin);

        // Start in Paused state.
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Playing);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Paused);
        app.update();

        // Press ESC and run toggle_pause directly (bypasses PreUpdate clear).
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.world_mut().run_system_once(toggle_pause).unwrap();
        app.update(); // apply state transition

        assert_eq!(
            *app.world().resource::<State<AppState>>(),
            AppState::Playing,
            "ESC in Paused should transition to Playing"
        );
    }
}
