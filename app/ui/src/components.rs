//! Reusable UI components and interaction handler.
//!
//! Provides [`MenuButton`], [`ButtonAction`], and the
//! [`handle_button_interaction`] system so every screen can share the same
//! mouse-click logic without coupling individual screens to state-transition
//! code.

use bevy::prelude::*;
use vs_core::states::AppState;

use crate::config::UiStyleParams;
use crate::styles::{DEFAULT_BUTTON_HOVER, DEFAULT_BUTTON_NORMAL, DEFAULT_BUTTON_PRESSED};

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/// Marker component attached to every interactive menu button.
///
/// Stores the [`ButtonAction`] that should be executed when the button is
/// clicked so that a single interaction system can handle all button presses.
#[derive(Component, Debug, Clone)]
pub struct MenuButton {
    /// The action triggered when this button is activated.
    pub action: ButtonAction,
}

// ---------------------------------------------------------------------------
// Enum
// ---------------------------------------------------------------------------

/// Identifies the intended behavior of a [`MenuButton`].
///
/// A single button-interaction system matches on this enum to perform the
/// correct state transition, avoiding per-screen input handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonAction {
    /// Transition from Title to Playing — starts a new run.
    StartGame,
    /// Return to the Title screen from any state.
    GoToTitle,
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Handles mouse interaction with [`MenuButton`] entities.
///
/// Changes the button background color on hover / press, and triggers the
/// appropriate [`AppState`] transition when a button is clicked.
/// Colors are read from [`UiStyleParams`]; `DEFAULT_BUTTON_*` constants are
/// used as fallbacks while the config is loading.
pub fn handle_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &MenuButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut next_state: ResMut<NextState<AppState>>,
    ui_style: UiStyleParams,
) {
    let color_normal = ui_style
        .get()
        .map(|c| Color::from(&c.button_normal))
        .unwrap_or(DEFAULT_BUTTON_NORMAL);
    let color_hover = ui_style
        .get()
        .map(|c| Color::from(&c.button_hover))
        .unwrap_or(DEFAULT_BUTTON_HOVER);
    let color_pressed = ui_style
        .get()
        .map(|c| Color::from(&c.button_pressed))
        .unwrap_or(DEFAULT_BUTTON_PRESSED);

    for (interaction, button, mut bg) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg = BackgroundColor(color_pressed);
                apply_action(button.action, &mut next_state);
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(color_hover);
            }
            Interaction::None => {
                *bg = BackgroundColor(color_normal);
            }
        }
    }
}

fn apply_action(action: ButtonAction, next_state: &mut NextState<AppState>) {
    match action {
        ButtonAction::StartGame => {
            next_state.set(AppState::Playing);
        }
        ButtonAction::GoToTitle => {
            next_state.set(AppState::Title);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_action_is_copy() {
        let a = ButtonAction::StartGame;
        let b = a; // Copy — must compile
        assert_eq!(a, b);
    }

    #[test]
    fn menu_button_stores_action() {
        let btn = MenuButton {
            action: ButtonAction::StartGame,
        };
        assert_eq!(btn.action, ButtonAction::StartGame);
    }

    #[test]
    fn apply_action_start_game_sets_playing_state() {
        use bevy::state::app::StatesPlugin;
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<AppState>();

        {
            let mut next_state = app.world_mut().resource_mut::<NextState<AppState>>();
            apply_action(ButtonAction::StartGame, &mut next_state);
        }
        app.update();

        assert_eq!(
            *app.world().resource::<State<AppState>>(),
            AppState::Playing
        );
    }

    #[test]
    fn apply_action_go_to_title_sets_title_state() {
        use bevy::state::app::StatesPlugin;
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<AppState>();

        // Transition away from Loading (default) first so Title is reachable.
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Playing);
        app.update();

        {
            let mut next_state = app.world_mut().resource_mut::<NextState<AppState>>();
            apply_action(ButtonAction::GoToTitle, &mut next_state);
        }
        app.update();

        assert_eq!(*app.world().resource::<State<AppState>>(), AppState::Title);
    }

    #[test]
    fn menu_button_is_clone() {
        let original = MenuButton {
            action: ButtonAction::StartGame,
        };
        let cloned = original.clone();
        assert_eq!(original.action, cloned.action);
    }
}
