use bevy::prelude::*;

use crate::types::UpgradeChoice;

/// Holds the current set of upgrade cards shown during a level-up.
#[derive(Resource, Debug, Default)]
pub struct LevelUpChoices {
    pub choices: Vec<UpgradeChoice>,
}
