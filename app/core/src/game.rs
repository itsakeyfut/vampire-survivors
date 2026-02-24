//! Game-level systems that operate on the overall run state.
//!
//! Systems here update global resources such as [`GameData`] that are not
//! owned by any single entity category (player, enemy, weapon, etc.).

use bevy::prelude::*;

use crate::resources::GameData;

/// Advances the run timer every frame.
///
/// Registered with `run_if(in_state(AppState::Playing))`, so it is
/// automatically suspended during `LevelUp` and `Paused` states.
pub fn update_game_timer(time: Res<Time>, mut game_data: ResMut<GameData>) {
    game_data.elapsed_time += time.delta_secs();
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::ecs::system::RunSystemOnce as _;

    use super::*;

    fn setup() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(GameData::default());
        app
    }

    #[test]
    fn elapsed_time_increases_by_delta() {
        let mut app = setup();

        let dt = 1.0 / 60.0_f32;
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(dt));
        app.world_mut()
            .run_system_once(update_game_timer)
            .expect("update_game_timer should run");

        let elapsed = app.world().resource::<GameData>().elapsed_time;
        assert!(
            (elapsed - dt).abs() < 1e-6,
            "elapsed_time should equal delta (got {elapsed})"
        );
    }

    #[test]
    fn elapsed_time_accumulates_across_frames() {
        let mut app = setup();

        let dt = 1.0 / 60.0_f32;
        for _ in 0..3 {
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(Duration::from_secs_f32(dt));
            app.world_mut()
                .run_system_once(update_game_timer)
                .expect("update_game_timer should run");
        }

        let elapsed = app.world().resource::<GameData>().elapsed_time;
        assert!(
            (elapsed - dt * 3.0).abs() < 1e-5,
            "elapsed_time should accumulate over 3 frames (got {elapsed})"
        );
    }

    #[test]
    fn elapsed_time_starts_at_zero() {
        let gd = GameData::default();
        assert_eq!(gd.elapsed_time, 0.0);
    }
}
