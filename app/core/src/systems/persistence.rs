//! Auto-save Bevy systems for persistent game data.
//!
//! These systems are registered in [`crate::GameCorePlugin`] on state
//! transitions and call the save methods on the relevant resources.

use bevy::prelude::*;

use crate::resources::{GameSettings, MetaProgress};

// ---------------------------------------------------------------------------
// MetaProgress auto-save
// ---------------------------------------------------------------------------

/// Saves [`MetaProgress`] to disk when the player transitions to
/// [`crate::states::AppState::GameOver`].
///
/// Save is suppressed when [`MetaProgress::load_failed`] is `true` so that a
/// corrupt `meta.json` is never overwritten by a synthetic default.
pub fn save_meta_on_game_over(meta: Res<MetaProgress>) {
    if meta.load_failed {
        warn!("Skipping meta progress save (load failed — original file preserved)");
        return;
    }
    info!("Saving meta progress (game over)…");
    meta.save();
}

/// Saves [`MetaProgress`] to disk when the player transitions to
/// [`crate::states::AppState::Victory`].
///
/// Save is suppressed when [`MetaProgress::load_failed`] is `true`.
pub fn save_meta_on_victory(meta: Res<MetaProgress>) {
    if meta.load_failed {
        warn!("Skipping meta progress save (load failed — original file preserved)");
        return;
    }
    info!("Saving meta progress (victory)…");
    meta.save();
}

/// Saves [`MetaProgress`] to disk when the player exits the
/// [`crate::states::AppState::MetaShop`] screen.
///
/// Save is suppressed when [`MetaProgress::load_failed`] is `true`.
pub fn save_meta_on_shop_exit(meta: Res<MetaProgress>) {
    if meta.load_failed {
        warn!("Skipping meta progress save (load failed — original file preserved)");
        return;
    }
    info!("Saving meta progress (shop exit)…");
    meta.save();
}

// ---------------------------------------------------------------------------
// GameSettings auto-save
// ---------------------------------------------------------------------------

/// Saves [`GameSettings`] to `save/settings.json` when the player exits the
/// [`crate::states::AppState::Settings`] screen.
pub fn save_settings_on_exit(settings: Res<GameSettings>) {
    info!("Saving settings (settings screen exit)…");
    settings.save();
}
