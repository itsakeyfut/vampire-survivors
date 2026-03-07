//! Kill count tracking.
//!
//! [`track_kill_count`] reads every [`EnemyDiedEvent`] and increments
//! [`GameData::kill_count`] so the gameplay HUD can display the running total.

use bevy::prelude::*;

use crate::events::EnemyDiedEvent;
use crate::resources::GameData;

/// Increments [`GameData::kill_count`] for every enemy death this frame.
///
/// Counts all enemy types including bosses and mini-bosses.
pub fn track_kill_count(
    mut died_events: MessageReader<EnemyDiedEvent>,
    mut game_data: ResMut<GameData>,
) {
    for _ in died_events.read() {
        game_data.kill_count += 1;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce as _;

    use super::*;
    use crate::{events::EnemyDiedEvent, resources::GameData, types::EnemyType};

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<EnemyDiedEvent>();
        app.insert_resource(GameData::default());
        app
    }

    fn write_death(app: &mut App) {
        app.world_mut().write_message(EnemyDiedEvent {
            entity: Entity::PLACEHOLDER,
            position: Vec2::ZERO,
            enemy_type: EnemyType::Bat,
            xp_value: 3,
        });
    }

    /// Each enemy death increments kill_count by 1.
    #[test]
    fn single_death_increments_kill_count() {
        let mut app = build_app();
        write_death(&mut app);

        app.world_mut().run_system_once(track_kill_count).unwrap();

        assert_eq!(app.world().resource::<GameData>().kill_count, 1);
    }

    /// Multiple deaths in one frame all increment kill_count.
    #[test]
    fn multiple_deaths_accumulate() {
        let mut app = build_app();
        write_death(&mut app);
        write_death(&mut app);
        write_death(&mut app);

        app.world_mut().run_system_once(track_kill_count).unwrap();

        assert_eq!(app.world().resource::<GameData>().kill_count, 3);
    }

    /// No deaths leaves kill_count unchanged.
    #[test]
    fn no_deaths_leaves_count_unchanged() {
        let mut app = build_app();

        app.world_mut().run_system_once(track_kill_count).unwrap();

        assert_eq!(app.world().resource::<GameData>().kill_count, 0);
    }
}
