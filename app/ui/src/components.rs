//! Reusable UI components and interaction handler.
//!
//! Provides [`MenuButton`], [`ButtonAction`], and the
//! [`handle_button_interaction`] system so every screen can share the same
//! mouse-click logic without coupling individual screens to state-transition
//! code.

use bevy::prelude::*;
use vs_core::config::{CharacterConfig, CharacterParams, GameConfig, GameParams};
use vs_core::resources::{GameSettings, MetaProgress, PendingUpgradeIndex};
use vs_core::states::AppState;
use vs_core::types::{CharacterType, MetaUpgradeType, get_character_stats, upgrade_cost};

use crate::config::MenuButtonHudParams;

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
    /// Transition directly to Playing — used for dev shortcuts and stubs.
    StartGame,
    /// Transition from Title to CharacterSelect — the normal new-run entry point.
    GoToCharacterSelect,
    /// Transition from Title to MetaShop.
    GoToMetaShop,
    /// Transition from Title to Settings.
    GoToSettings,
    /// Return to the Title screen from any state.
    GoToTitle,
    /// Toggle the UI language between Japanese and English.
    ToggleLanguage,
    /// Confirm the upgrade card at the given index and resume gameplay.
    ///
    /// The index refers to the slot in [`vs_core::resources::LevelUpChoices`]
    /// that was selected.  Gameplay systems that apply upgrades read this
    /// index when re-entering [`AppState::Playing`].
    SelectUpgrade(usize),
    /// Resume gameplay from the pause screen — transitions Paused → Playing.
    ResumeGame,
    /// Unlock a character in the gold shop.
    ///
    /// Deducts the character's unlock cost from [`MetaProgress::total_gold`] and
    /// adds the character to [`MetaProgress::unlocked_characters`] when the
    /// player can afford it and has not already unlocked it.
    UnlockCharacter(CharacterType),
    /// Purchase a permanent upgrade in the gold shop.
    ///
    /// Deducts [`upgrade_cost`] from [`MetaProgress::total_gold`] and records
    /// the upgrade in [`MetaProgress::purchased_upgrades`] when the player can
    /// afford it and has not already purchased it.
    PurchaseUpgrade(MetaUpgradeType),
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Handles mouse interaction with [`MenuButton`] entities.
///
/// Changes the button background color on hover / press, and triggers the
/// appropriate [`AppState`] transition when a button is clicked.
/// Colors are read from [`MenuButtonHudParams`] via typed accessor methods,
/// which apply built-in fallbacks while the config is loading.
#[allow(clippy::too_many_arguments)]
pub fn handle_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &MenuButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut next_state: ResMut<NextState<AppState>>,
    btn_cfg: MenuButtonHudParams,
    mut pending: Option<ResMut<PendingUpgradeIndex>>,
    mut settings: Option<ResMut<GameSettings>>,
    mut meta: Option<ResMut<MetaProgress>>,
    char_params: CharacterParams,
    game_params: GameParams,
) {
    let color_normal = btn_cfg.color_normal();
    let color_hover = btn_cfg.color_hover();
    let color_pressed = btn_cfg.color_pressed();

    for (interaction, button, mut bg) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg = BackgroundColor(color_pressed);
                apply_action(
                    button.action,
                    &mut next_state,
                    &mut pending,
                    &mut settings,
                    &mut meta,
                    char_params.get(),
                    game_params.get(),
                );
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

fn apply_action(
    action: ButtonAction,
    next_state: &mut NextState<AppState>,
    pending: &mut Option<ResMut<PendingUpgradeIndex>>,
    settings: &mut Option<ResMut<GameSettings>>,
    meta: &mut Option<ResMut<MetaProgress>>,
    char_cfg: Option<&CharacterConfig>,
    game_cfg: Option<&GameConfig>,
) {
    match action {
        ButtonAction::StartGame => {
            next_state.set(AppState::Playing);
        }
        ButtonAction::GoToCharacterSelect => {
            next_state.set(AppState::CharacterSelect);
        }
        ButtonAction::GoToMetaShop => {
            next_state.set(AppState::MetaShop);
        }
        ButtonAction::GoToSettings => {
            next_state.set(AppState::Settings);
        }
        ButtonAction::GoToTitle => {
            next_state.set(AppState::Title);
        }
        ButtonAction::ToggleLanguage => {
            if let Some(s) = settings {
                s.language = s.language.next();
            }
        }
        ButtonAction::SelectUpgrade(index) => {
            if let Some(p) = pending {
                p.0 = Some(index);
            }
            next_state.set(AppState::Playing);
        }
        ButtonAction::ResumeGame => {
            next_state.set(AppState::Playing);
        }
        ButtonAction::UnlockCharacter(ct) => {
            if let Some(m) = meta {
                let cost = char_cfg
                    .map(|c| c.stats_for(ct).unlock_cost)
                    .unwrap_or_else(|| get_character_stats(ct).unlock_cost);
                if m.total_gold >= cost && !m.unlocked_characters.contains(&ct) {
                    m.total_gold = m.total_gold.saturating_sub(cost);
                    m.unlocked_characters.push(ct);
                    m.save();
                    info!("Unlocked character {:?} for {}G", ct, cost);
                }
            }
        }
        ButtonAction::PurchaseUpgrade(ut) => {
            if let Some(m) = meta {
                let cost = game_cfg
                    .map(|c| c.upgrade_cost(ut))
                    .unwrap_or_else(|| upgrade_cost(ut));
                if m.total_gold >= cost && !m.purchased_upgrades.contains(&ut) {
                    m.total_gold = m.total_gold.saturating_sub(cost);
                    m.purchased_upgrades.push(ut);
                    m.save();
                    info!("Purchased upgrade {:?} for {}G", ut, cost);
                }
            }
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
            apply_action(
                ButtonAction::StartGame,
                &mut next_state,
                &mut None,
                &mut None,
                &mut None,
                None,
                None,
            );
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
            apply_action(
                ButtonAction::GoToTitle,
                &mut next_state,
                &mut None,
                &mut None,
                &mut None,
                None,
                None,
            );
        }
        app.update();

        assert_eq!(*app.world().resource::<State<AppState>>(), AppState::Title);
    }

    #[test]
    fn apply_action_select_upgrade_sets_playing_state() {
        use bevy::state::app::StatesPlugin;
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<AppState>();

        {
            let mut next_state = app.world_mut().resource_mut::<NextState<AppState>>();
            // None pending — PendingUpgradeIndex not available in this isolated test.
            apply_action(
                ButtonAction::SelectUpgrade(1),
                &mut next_state,
                &mut None,
                &mut None,
                &mut None,
                None,
                None,
            );
        }
        app.update();

        assert_eq!(
            *app.world().resource::<State<AppState>>(),
            AppState::Playing
        );
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
