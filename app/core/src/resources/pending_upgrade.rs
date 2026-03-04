use bevy::prelude::*;

/// Stores the index into [`super::LevelUpChoices::choices`] that the player
/// selected on the level-up card screen.
///
/// Set by the UI layer when a card button is clicked
/// (`ButtonAction::SelectUpgrade(index)`), consumed and cleared by
/// [`crate::systems::xp::apply::apply_selected_upgrade`] on re-entry to
/// [`crate::states::AppState::Playing`].
///
/// A value of `None` means no upgrade is pending (e.g. normal game start).
#[derive(Resource, Debug, Default)]
pub struct PendingUpgradeIndex(pub Option<usize>);
