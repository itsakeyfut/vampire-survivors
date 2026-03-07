//! Boss defeat detection and victory state transition.
//!
//! [`check_boss_defeated`] runs every frame during [`AppState::Playing`].
//! When an [`EnemyDiedEvent`] is received for [`EnemyType::BossDeath`] it
//! emits a [`VictoryEvent`] and transitions the app to [`AppState::Victory`].

use bevy::prelude::*;

use crate::{
    events::{EnemyDiedEvent, VictoryEvent},
    states::AppState,
    types::EnemyType,
};

pub struct VictoryPlugin;

impl Plugin for VictoryPlugin {
    fn build(&self, app: &mut App) {
        use crate::systems::damage::apply_damage_to_enemies;
        app.add_systems(
            Update,
            check_boss_defeated
                .after(apply_damage_to_enemies)
                .run_if(in_state(AppState::Playing)),
        );
    }
}

/// Checks whether Boss Death was defeated this frame and triggers victory.
///
/// Reads all [`EnemyDiedEvent`] messages in the current frame.  When one
/// carries [`EnemyType::BossDeath`], a [`VictoryEvent`] is emitted and the
/// state transitions to [`AppState::Victory`].  At most one transition fires
/// per run.
pub fn check_boss_defeated(
    mut died_events: MessageReader<EnemyDiedEvent>,
    mut victory_events: MessageWriter<VictoryEvent>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for event in died_events.read() {
        if event.enemy_type == EnemyType::BossDeath {
            victory_events.write(VictoryEvent);
            next_state.set(AppState::Victory);
            return; // only one boss; no need to continue
        }
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
    use crate::{
        events::{EnemyDiedEvent, VictoryEvent},
        types::EnemyType,
    };

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<AppState>();
        app.add_message::<EnemyDiedEvent>();
        app.add_message::<VictoryEvent>();
        app
    }

    fn send_died(app: &mut App, enemy_type: EnemyType) {
        app.world_mut().write_message(EnemyDiedEvent {
            entity: Entity::PLACEHOLDER,
            position: Vec2::ZERO,
            enemy_type,
            xp_value: 0,
        });
    }

    fn victory_events(app: &App) -> Vec<VictoryEvent> {
        let messages = app.world().resource::<Messages<VictoryEvent>>();
        let mut cursor = messages.get_cursor();
        cursor.read(messages).cloned().collect()
    }

    /// Boss death emits a VictoryEvent.
    #[test]
    fn boss_death_emits_victory_event() {
        let mut app = build_app();
        send_died(&mut app, EnemyType::BossDeath);

        app.world_mut()
            .run_system_once(check_boss_defeated)
            .expect("check_boss_defeated should run");

        assert_eq!(
            victory_events(&app).len(),
            1,
            "exactly one VictoryEvent expected when boss is defeated"
        );
    }

    /// Boss death transitions to Victory state.
    #[test]
    fn boss_death_transitions_to_victory_state() {
        let mut app = build_app();
        send_died(&mut app, EnemyType::BossDeath);

        // Set current state to Playing first so the transition is valid.
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Playing);
        app.update();

        app.world_mut()
            .run_system_once(check_boss_defeated)
            .expect("check_boss_defeated should run");
        app.update();

        assert_eq!(
            *app.world().resource::<State<AppState>>(),
            AppState::Victory,
            "state should be Victory after boss is defeated"
        );
    }

    /// Non-boss enemy death does not emit VictoryEvent.
    #[test]
    fn non_boss_death_no_victory_event() {
        let mut app = build_app();

        for enemy_type in [
            EnemyType::Bat,
            EnemyType::Skeleton,
            EnemyType::Zombie,
            EnemyType::Ghost,
            EnemyType::Demon,
            EnemyType::Medusa,
            EnemyType::Dragon,
            EnemyType::MiniDeath,
        ] {
            send_died(&mut app, enemy_type);
        }

        app.world_mut()
            .run_system_once(check_boss_defeated)
            .expect("check_boss_defeated should run");

        assert!(
            victory_events(&app).is_empty(),
            "no VictoryEvent expected when non-boss enemies die"
        );
    }

    /// No events → no VictoryEvent.
    #[test]
    fn no_died_events_no_victory() {
        let mut app = build_app();

        app.world_mut()
            .run_system_once(check_boss_defeated)
            .expect("check_boss_defeated should run");

        assert!(
            victory_events(&app).is_empty(),
            "no VictoryEvent expected when no enemies died"
        );
    }
}
