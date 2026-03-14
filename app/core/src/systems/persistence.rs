//! Auto-save Bevy systems for persistent game data.
//!
//! These systems are registered in [`crate::GameCorePlugin`] on state
//! transitions and call the save methods on the relevant resources.

use bevy::prelude::*;

use crate::resources::{GameData, GameSettings, MetaProgress};

// ---------------------------------------------------------------------------
// Gold carry-over
// ---------------------------------------------------------------------------

/// Adds the gold earned during the run to [`MetaProgress::total_gold`] on
/// game over.
///
/// Runs on [`crate::states::AppState::GameOver`] entry, **before**
/// [`save_meta_on_game_over`] (the two systems are chained in
/// [`crate::GameCorePlugin`]).
pub fn accrue_gold_on_game_over(data: Res<GameData>, mut meta: ResMut<MetaProgress>) {
    let earned = data.gold_earned;
    if earned > 0 {
        meta.total_gold = meta.total_gold.saturating_add(earned);
        info!(
            "Gold carry-over (game over): +{earned} → total {}",
            meta.total_gold
        );
    }
}

/// Adds the gold earned during the run to [`MetaProgress::total_gold`] on
/// victory.
///
/// Runs on [`crate::states::AppState::Victory`] entry, **before**
/// [`save_meta_on_victory`].
pub fn accrue_gold_on_victory(data: Res<GameData>, mut meta: ResMut<MetaProgress>) {
    let earned = data.gold_earned;
    if earned > 0 {
        meta.total_gold = meta.total_gold.saturating_add(earned);
        info!(
            "Gold carry-over (victory): +{earned} → total {}",
            meta.total_gold
        );
    }
}

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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce as _;
    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::resources::{GameData, MetaProgress};

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.insert_resource(GameData::default());
        app.insert_resource(MetaProgress::default());
        app
    }

    /// Gold earned this run is added to total_gold on game over.
    #[test]
    fn accrue_gold_on_game_over_adds_gold() {
        let mut app = build_app();
        app.world_mut().resource_mut::<GameData>().gold_earned = 150;
        app.world_mut().resource_mut::<MetaProgress>().total_gold = 500;

        app.world_mut()
            .run_system_once(accrue_gold_on_game_over)
            .expect("system should run");

        let meta = app.world().resource::<MetaProgress>();
        assert_eq!(meta.total_gold, 650);
    }

    /// Gold earned this run is added to total_gold on victory.
    #[test]
    fn accrue_gold_on_victory_adds_gold() {
        let mut app = build_app();
        app.world_mut().resource_mut::<GameData>().gold_earned = 300;
        app.world_mut().resource_mut::<MetaProgress>().total_gold = 0;

        app.world_mut()
            .run_system_once(accrue_gold_on_victory)
            .expect("system should run");

        let meta = app.world().resource::<MetaProgress>();
        assert_eq!(meta.total_gold, 300);
    }

    /// Zero gold earned does not change total_gold.
    #[test]
    fn accrue_gold_no_op_when_zero_earned() {
        let mut app = build_app();
        app.world_mut().resource_mut::<GameData>().gold_earned = 0;
        app.world_mut().resource_mut::<MetaProgress>().total_gold = 1000;

        app.world_mut()
            .run_system_once(accrue_gold_on_game_over)
            .expect("system should run");

        let meta = app.world().resource::<MetaProgress>();
        assert_eq!(
            meta.total_gold, 1000,
            "total_gold should not change for zero earned"
        );
    }

    /// Accumulation saturates at u32::MAX instead of wrapping.
    #[test]
    fn accrue_gold_saturates_at_u32_max() {
        let mut app = build_app();
        app.world_mut().resource_mut::<GameData>().gold_earned = 1;
        app.world_mut().resource_mut::<MetaProgress>().total_gold = u32::MAX;

        app.world_mut()
            .run_system_once(accrue_gold_on_game_over)
            .expect("system should run");

        let meta = app.world().resource::<MetaProgress>();
        assert_eq!(meta.total_gold, u32::MAX, "should saturate at u32::MAX");
    }
}
