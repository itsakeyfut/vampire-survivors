//! Damage application system.
//!
//! [`apply_damage`] reads every [`DamageEnemyEvent`] queued this frame and
//! reduces the target enemy's HP via [`Enemy::take_damage`].  Enemies whose HP
//! reaches zero are despawned immediately.  Reward spawning (XP gems, gold
//! coins) will be added in a later phase.

use bevy::prelude::*;

use crate::{components::Enemy, events::DamageEnemyEvent};

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Reads every [`DamageEnemyEvent`] and applies the damage to the target enemy.
///
/// - If the target entity no longer exists (already despawned) the event is
///   silently skipped.
/// - Enemies reduced to zero HP are despawned immediately.
pub fn apply_damage(
    mut events: MessageReader<DamageEnemyEvent>,
    mut enemy_q: Query<&mut Enemy>,
    mut commands: Commands,
) {
    for event in events.read() {
        let Ok(mut enemy) = enemy_q.get_mut(event.entity) else {
            continue;
        };
        enemy.take_damage(event.damage);
        if enemy.is_dead() {
            commands.entity(event.entity).despawn();
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
        events::DamageEnemyEvent,
        types::{EnemyType, WeaponType},
    };

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<DamageEnemyEvent>();
        app
    }

    fn spawn_enemy(app: &mut App) -> Entity {
        app.world_mut()
            .spawn(Enemy::from_type(EnemyType::Bat, 1.0))
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
            .run_system_once(apply_damage)
            .expect("apply_damage should run");
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
