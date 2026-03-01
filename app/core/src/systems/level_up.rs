//! Level-up detection and state transition system.
//!
//! [`check_level_up`] runs every frame during [`AppState::Playing`].  When
//! [`GameData::current_xp`] reaches or exceeds [`GameData::xp_to_next_level`],
//! the system:
//!
//! 1. Subtracts the threshold from `current_xp` (excess XP carries over).
//! 2. Increments [`GameData::current_level`].
//! 3. Recomputes [`GameData::xp_to_next_level`] using an exponential curve:
//!    `xp_level_base × xp_level_multiplier^(new_level − 1)`.
//! 4. Emits a [`LevelUpEvent`].
//! 5. Transitions to [`AppState::LevelUp`], pausing all gameplay systems
//!    until the player selects an upgrade card.

use bevy::prelude::*;

use crate::{config::GameParams, events::LevelUpEvent, resources::GameData, states::AppState};

// ---------------------------------------------------------------------------
// Fallback constants
// ---------------------------------------------------------------------------

/// XP required for the very first level-up.
///
/// Used when the RON game config has not yet loaded.
const DEFAULT_XP_LEVEL_BASE: u32 = 20;

/// Exponential multiplier applied to the XP threshold on each level-up.
///
/// Used when the RON game config has not yet loaded.
const DEFAULT_XP_LEVEL_MULTIPLIER: f32 = 1.2;

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Checks whether accumulated XP triggers a level-up and handles the
/// transition to [`AppState::LevelUp`].
///
/// Only one level-up is processed per call — if the player somehow has enough
/// XP to cross multiple thresholds at once, the excess is carried forward and
/// the next level-up fires on the following re-entry to `Playing`.
pub fn check_level_up(
    mut game_data: ResMut<GameData>,
    mut next_state: ResMut<NextState<AppState>>,
    mut level_up_events: MessageWriter<LevelUpEvent>,
    game_cfg: GameParams,
) {
    if game_data.current_xp < game_data.xp_to_next_level {
        return;
    }

    // Carry excess XP into the new level.
    game_data.current_xp -= game_data.xp_to_next_level;
    game_data.current_level += 1;

    // Recompute the threshold for the next level using the exponential curve.
    let base = game_cfg
        .get()
        .map(|c| c.xp_level_base)
        .unwrap_or(DEFAULT_XP_LEVEL_BASE);
    let mult = game_cfg
        .get()
        .map(|c| c.xp_level_multiplier)
        .unwrap_or(DEFAULT_XP_LEVEL_MULTIPLIER);
    game_data.xp_to_next_level =
        (base as f32 * mult.powi(game_data.current_level as i32 - 1)).round() as u32;

    level_up_events.write(LevelUpEvent {
        new_level: game_data.current_level,
    });
    next_state.set(AppState::LevelUp);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce as _;
    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::{events::LevelUpEvent, resources::GameData};

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<AppState>();
        app.add_message::<LevelUpEvent>();
        app.insert_resource(GameData::default());
        app
    }

    fn level_up_events(app: &App) -> Vec<LevelUpEvent> {
        let messages = app.world().resource::<Messages<LevelUpEvent>>();
        let mut cursor = messages.get_cursor();
        cursor.read(messages).cloned().collect()
    }

    fn run_check(app: &mut App) {
        app.world_mut()
            .run_system_once(check_level_up)
            .expect("check_level_up should run");
    }

    /// No level-up when XP is below threshold.
    #[test]
    fn no_level_up_below_threshold() {
        let mut app = build_app();
        app.world_mut().resource_mut::<GameData>().current_xp = 10;

        run_check(&mut app);

        let gd = app.world().resource::<GameData>();
        assert_eq!(gd.current_level, 1, "level must not change");
        assert!(level_up_events(&app).is_empty(), "no event expected");
    }

    /// Exact threshold triggers a level-up.
    #[test]
    fn exact_threshold_triggers_level_up() {
        let mut app = build_app();
        app.world_mut().resource_mut::<GameData>().current_xp = 20; // == xp_to_next_level

        run_check(&mut app);

        let gd = app.world().resource::<GameData>();
        assert_eq!(gd.current_level, 2, "level should be 2");
        assert_eq!(gd.current_xp, 0, "no excess XP");
        assert_eq!(level_up_events(&app).len(), 1, "exactly one LevelUpEvent");
        assert_eq!(level_up_events(&app)[0].new_level, 2);
    }

    /// Excess XP carries over into the new level.
    #[test]
    fn excess_xp_carries_over() {
        let mut app = build_app();
        app.world_mut().resource_mut::<GameData>().current_xp = 25; // 5 over threshold

        run_check(&mut app);

        let gd = app.world().resource::<GameData>();
        assert_eq!(gd.current_xp, 5, "5 XP should carry over");
        assert_eq!(gd.current_level, 2);
    }

    /// XP threshold for next level is larger than the current one.
    #[test]
    fn next_threshold_is_larger() {
        let mut app = build_app();
        let initial_threshold = app.world().resource::<GameData>().xp_to_next_level;
        app.world_mut().resource_mut::<GameData>().current_xp = initial_threshold;

        run_check(&mut app);

        let next_threshold = app.world().resource::<GameData>().xp_to_next_level;
        assert!(
            next_threshold > initial_threshold,
            "next threshold ({next_threshold}) must exceed previous ({initial_threshold})"
        );
    }

    /// LevelUpEvent carries the correct new level.
    #[test]
    fn level_up_event_carries_new_level() {
        let mut app = build_app();
        app.world_mut().resource_mut::<GameData>().current_xp = 20;

        run_check(&mut app);

        let events = level_up_events(&app);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].new_level, 2);
    }

    /// State transitions to LevelUp on level-up.
    #[test]
    fn transitions_to_level_up_state() {
        let mut app = build_app();
        // Start in Playing state.
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Playing);
        app.update();

        app.world_mut().resource_mut::<GameData>().current_xp = 20;
        run_check(&mut app);
        app.update(); // flush NextState

        assert_eq!(
            *app.world().resource::<State<AppState>>(),
            AppState::LevelUp,
            "state should be LevelUp after levelling up"
        );
    }

    /// No state change or event when XP is below threshold.
    #[test]
    fn no_state_change_below_threshold() {
        let mut app = build_app();
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Playing);
        app.update();

        app.world_mut().resource_mut::<GameData>().current_xp = 5;
        run_check(&mut app);
        app.update();

        assert_eq!(
            *app.world().resource::<State<AppState>>(),
            AppState::Playing,
            "state should remain Playing when XP is insufficient"
        );
    }
}
