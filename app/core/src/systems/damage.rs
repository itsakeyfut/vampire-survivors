//! Damage application system.
//!
//! [`apply_damage_to_enemies`] reads every [`DamageEnemyEvent`] queued this
//! frame and reduces the target enemy's HP via [`Enemy::take_damage`].
//! Enemies whose HP reaches zero are despawned and an [`EnemyDiedEvent`] is
//! emitted so downstream systems (XP gems, gold coins) can react.

use bevy::prelude::*;

use crate::{
    components::Enemy,
    events::{DamageEnemyEvent, EnemyDiedEvent},
};

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Reads every [`DamageEnemyEvent`] and applies the damage to the target enemy.
///
/// - If the target entity no longer exists (already despawned) the event is
///   silently skipped.
/// - Enemies reduced to zero HP are despawned and an [`EnemyDiedEvent`] is
///   emitted carrying the entity, world position, and enemy type for loot
///   spawning.
pub fn apply_damage_to_enemies(
    mut damage_events: MessageReader<DamageEnemyEvent>,
    mut died_events: MessageWriter<EnemyDiedEvent>,
    mut enemy_q: Query<(&mut Enemy, &Transform)>,
    mut commands: Commands,
) {
    for event in damage_events.read() {
        let Ok((mut enemy, transform)) = enemy_q.get_mut(event.entity) else {
            continue;
        };
        enemy.take_damage(event.damage);
        if enemy.is_dead() {
            let position = transform.translation.truncate();
            let enemy_type = enemy.enemy_type;
            commands.entity(event.entity).despawn();
            died_events.write(EnemyDiedEvent {
                entity: event.entity,
                position,
                enemy_type,
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce as _;

    use super::*;
    use crate::{
        components::Enemy,
        events::{DamageEnemyEvent, EnemyDiedEvent},
        types::{EnemyType, WeaponType},
    };

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<DamageEnemyEvent>();
        app.add_message::<EnemyDiedEvent>();
        app
    }

    fn spawn_enemy(app: &mut App) -> Entity {
        app.world_mut()
            .spawn((
                Enemy::from_type(EnemyType::Bat, 1.0),
                Transform::from_xyz(10.0, 20.0, 0.0),
            ))
            .id()
    }

    fn send_damage(app: &mut App, entity: Entity, damage: f32) {
        app.world_mut().write_message(DamageEnemyEvent {
            entity,
            damage,
            weapon_type: WeaponType::Whip,
        });
    }

    fn run_apply(app: &mut App) {
        app.world_mut()
            .run_system_once(apply_damage_to_enemies)
            .expect("apply_damage_to_enemies should run");
    }

    fn died_events(app: &App) -> Vec<EnemyDiedEvent> {
        let messages = app.world().resource::<Messages<EnemyDiedEvent>>();
        let mut cursor = messages.get_cursor();
        cursor.read(messages).cloned().collect()
    }

    /// Damage event reduces enemy HP by the specified amount.
    #[test]
    fn damage_reduces_enemy_hp() {
        let mut app = build_app();
        let entity = spawn_enemy(&mut app);
        let initial_hp = app.world().get::<Enemy>(entity).unwrap().current_hp;

        send_damage(&mut app, entity, 5.0);
        run_apply(&mut app);

        let hp = app.world().get::<Enemy>(entity).unwrap().current_hp;
        assert_eq!(hp, initial_hp - 5.0);
    }

    /// Enemy with HP reduced to zero is despawned.
    #[test]
    fn lethal_damage_despawns_enemy() {
        let mut app = build_app();
        let entity = spawn_enemy(&mut app);

        send_damage(&mut app, entity, 9999.0);
        run_apply(&mut app);

        assert!(
            app.world().get_entity(entity).is_err(),
            "dead enemy should be despawned"
        );
    }

    /// Lethal damage emits an EnemyDiedEvent with correct position and type.
    #[test]
    fn lethal_damage_emits_enemy_died_event() {
        let mut app = build_app();
        let entity = spawn_enemy(&mut app); // spawned at (10, 20)

        send_damage(&mut app, entity, 9999.0);
        run_apply(&mut app);

        let events = died_events(&app);
        assert_eq!(
            events.len(),
            1,
            "exactly one EnemyDiedEvent should be emitted"
        );
        assert_eq!(events[0].entity, entity);
        assert_eq!(events[0].position, Vec2::new(10.0, 20.0));
        assert_eq!(events[0].enemy_type, EnemyType::Bat);
    }

    /// Non-lethal damage does not emit an EnemyDiedEvent.
    #[test]
    fn non_lethal_damage_no_died_event() {
        let mut app = build_app();
        let entity = spawn_enemy(&mut app);

        send_damage(&mut app, entity, 1.0);
        run_apply(&mut app);

        assert!(
            died_events(&app).is_empty(),
            "non-lethal damage should not emit EnemyDiedEvent"
        );
        // Entity still alive
        assert!(app.world().get_entity(entity).is_ok());
    }

    /// Event targeting a non-existent entity is silently ignored.
    #[test]
    fn stale_entity_is_ignored() {
        let mut app = build_app();
        let entity = spawn_enemy(&mut app);
        app.world_mut().entity_mut(entity).despawn();

        send_damage(&mut app, entity, 10.0);
        // Should not panic.
        run_apply(&mut app);
    }

    /// Zero damage event leaves HP unchanged.
    #[test]
    fn zero_damage_is_noop() {
        let mut app = build_app();
        let entity = spawn_enemy(&mut app);
        let initial_hp = app.world().get::<Enemy>(entity).unwrap().current_hp;

        send_damage(&mut app, entity, 0.0);
        run_apply(&mut app);

        let hp = app.world().get::<Enemy>(entity).unwrap().current_hp;
        assert_eq!(hp, initial_hp);
    }

    /// Multiple damage events on the same frame stack correctly.
    #[test]
    fn multiple_events_stack() {
        let mut app = build_app();
        let entity = spawn_enemy(&mut app);
        let initial_hp = app.world().get::<Enemy>(entity).unwrap().current_hp;

        send_damage(&mut app, entity, 2.0);
        send_damage(&mut app, entity, 3.0);
        run_apply(&mut app);

        let hp = app.world().get::<Enemy>(entity).unwrap().current_hp;
        assert_eq!(hp, initial_hp - 5.0);
    }
}
