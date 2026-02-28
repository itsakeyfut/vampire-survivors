//! Player death detection and game-over state transition.
//!
//! [`check_player_death`] runs every frame during [`AppState::Playing`].
//! When [`PlayerStats::current_hp`] drops to zero it emits a [`GameOverEvent`]
//! and transitions the app to [`AppState::GameOver`].

use bevy::prelude::*;

use crate::{
    components::{Player, PlayerStats},
    events::GameOverEvent,
    states::AppState,
};

/// Checks whether the player's HP has reached zero and triggers game over.
///
/// Emits a [`GameOverEvent`] and sets [`NextState`] to [`AppState::GameOver`]
/// on the first frame that `current_hp` is zero or below.  The system is a
/// no-op while the player entity does not exist (e.g., mid-despawn).
pub fn check_player_death(
    player_q: Query<&PlayerStats, With<Player>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut game_over_events: MessageWriter<GameOverEvent>,
) {
    let Ok(stats) = player_q.single() else {
        return;
    };
    if stats.current_hp <= 0.0 {
        game_over_events.write(GameOverEvent);
        next_state.set(AppState::GameOver);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce as _;
    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::{components::PlayerStats, events::GameOverEvent};

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<AppState>();
        app.add_message::<GameOverEvent>();
        app
    }

    fn spawn_player(app: &mut App, hp: f32) -> Entity {
        let mut stats = PlayerStats::default();
        stats.current_hp = hp;
        app.world_mut()
            .spawn((Player, stats, Transform::default()))
            .id()
    }

    fn game_over_events(app: &App) -> Vec<GameOverEvent> {
        let messages = app.world().resource::<Messages<GameOverEvent>>();
        let mut cursor = messages.get_cursor();
        cursor.read(messages).cloned().collect()
    }

    /// A player with positive HP does not trigger game over.
    #[test]
    fn alive_player_no_event() {
        let mut app = build_app();
        spawn_player(&mut app, 50.0);

        app.world_mut()
            .run_system_once(check_player_death)
            .expect("check_player_death should run");

        assert!(
            game_over_events(&app).is_empty(),
            "no GameOverEvent expected while player is alive"
        );
    }

    /// HP exactly zero triggers a GameOverEvent.
    #[test]
    fn zero_hp_emits_game_over_event() {
        let mut app = build_app();
        spawn_player(&mut app, 0.0);

        app.world_mut()
            .run_system_once(check_player_death)
            .expect("check_player_death should run");

        assert_eq!(
            game_over_events(&app).len(),
            1,
            "exactly one GameOverEvent expected when HP == 0"
        );
    }

    /// HP exactly zero transitions state to GameOver.
    #[test]
    fn zero_hp_transitions_to_game_over_state() {
        let mut app = build_app();
        spawn_player(&mut app, 0.0);

        // Transition to Playing first so NextState::set(GameOver) is meaningful.
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Playing);
        app.update();

        app.world_mut()
            .run_system_once(check_player_death)
            .expect("check_player_death should run");
        app.update();

        assert_eq!(
            *app.world().resource::<State<AppState>>(),
            AppState::GameOver,
            "state should be GameOver after player death"
        );
    }

    /// No player entity — system does not panic and emits no event.
    #[test]
    fn no_player_no_event() {
        let mut app = build_app();

        app.world_mut()
            .run_system_once(check_player_death)
            .expect("check_player_death should run without a player entity");

        assert!(
            game_over_events(&app).is_empty(),
            "no GameOverEvent expected when no player exists"
        );
    }
}
