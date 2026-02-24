//! Reusable UI components and interaction handler.
//!
//! Provides [`MenuButton`], [`ButtonAction`], and the
//! [`handle_button_interaction`] system so every screen can share the same
//! mouse-click logic without coupling individual screens to state-transition
//! code.

use bevy::prelude::*;
use vs_core::states::AppState;

use crate::styles::{BUTTON_HOVER, BUTTON_NORMAL, BUTTON_PRESSED};

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
    // Additional actions (GoToTitle, ResumeGame, etc.) will be added in
    // Phase 11 as further screens are implemented.
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Handles mouse interaction with [`MenuButton`] entities.
///
/// Changes the button background color on hover / press, and triggers the
/// appropriate [`AppState`] transition when a button is clicked.
pub fn handle_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &MenuButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, button, mut bg) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg = BackgroundColor(BUTTON_PRESSED);
                apply_action(button.action, &mut next_state);
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(BUTTON_HOVER);
            }
            Interaction::None => {
                *bg = BackgroundColor(BUTTON_NORMAL);
            }
        }
    }
}

fn apply_action(action: ButtonAction, next_state: &mut NextState<AppState>) {
    match action {
        ButtonAction::StartGame => {
            next_state.set(AppState::Playing);
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
    fn menu_button_is_clone() {
        let original = MenuButton {
            action: ButtonAction::StartGame,
        };
        let cloned = original.clone();
        assert_eq!(original.action, cloned.action);
    }
}
