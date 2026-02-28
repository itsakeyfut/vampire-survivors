//! XP gem drop system.
//!
//! [`spawn_xp_gems`] listens for [`EnemyDiedEvent`] and spawns an
//! [`ExperienceGem`] entity at the enemy's last known position.
//!
//! The gem value is taken directly from [`EnemyDiedEvent::xp_value`], which
//! is populated by [`apply_damage_to_enemies`] from the enemy's
//! [`Enemy::xp_value`] field (sourced from `enemy.ron` at spawn time).
//! No separate config lookup is needed here.

use bevy::{prelude::*, state::state_scoped::DespawnOnExit};

use crate::{components::ExperienceGem, events::EnemyDiedEvent, states::AppState};

/// Gem visual radius in pixels (placeholder; replace with real sprite later).
const GEM_RADIUS: f32 = 6.0;

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Spawns an [`ExperienceGem`] at the death position for every
/// [`EnemyDiedEvent`] received this frame.
///
/// - Gem value is taken directly from `event.xp_value`, which was set from
///   `enemy.ron` config when the enemy spawned.
/// - Rendered as a green circle placeholder sprite (`GEM_RADIUS` px radius).
/// - Tagged with [`DespawnOnExit`]`(`[`AppState::Playing`]`)` so gems are
///   automatically cleaned up when the run ends.
///
/// Must run after [`apply_damage_to_enemies`](crate::systems::damage::apply_damage_to_enemies)
/// so that `EnemyDiedEvent` messages are already written before this system
/// reads them.
pub fn spawn_xp_gems(mut commands: Commands, mut died_events: MessageReader<EnemyDiedEvent>) {
    for event in died_events.read() {
        commands.spawn((
            DespawnOnExit(AppState::Playing),
            ExperienceGem {
                value: event.xp_value,
            },
            // Green circle placeholder sprite; replace with real art in Phase 17.
            Sprite {
                color: Color::srgb(0.2, 0.9, 0.2),
                custom_size: Some(Vec2::splat(GEM_RADIUS * 2.0)),
                ..default()
            },
            // z = 0.5 — above ground (z = 0) but below enemies (z ≈ 1).
            Transform::from_xyz(event.position.x, event.position.y, 0.5),
        ));
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce as _;

    use super::*;
    use crate::{components::ExperienceGem, events::EnemyDiedEvent, types::EnemyType};

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<EnemyDiedEvent>();
        app
    }

    fn send_died(app: &mut App, enemy_type: EnemyType, position: Vec2, xp_value: u32) {
        let entity = app.world_mut().spawn_empty().id();
        app.world_mut().write_message(EnemyDiedEvent {
            entity,
            position,
            enemy_type,
            xp_value,
        });
    }

    fn run_system(app: &mut App) {
        app.world_mut()
            .run_system_once(spawn_xp_gems)
            .expect("spawn_xp_gems should run");
    }

    fn gems(app: &mut App) -> Vec<(ExperienceGem, Transform)> {
        let mut q = app.world_mut().query::<(&ExperienceGem, &Transform)>();
        q.iter(app.world())
            .map(|(g, t)| (ExperienceGem { value: g.value }, *t))
            .collect()
    }

    /// No EnemyDiedEvent → no gems spawned.
    #[test]
    fn no_event_no_gem() {
        let mut app = build_app();
        run_system(&mut app);
        assert!(gems(&mut app).is_empty(), "no gems without events");
    }

    /// Bat death spawns a gem with the XP value carried by the event.
    #[test]
    fn bat_death_spawns_gem_with_correct_value() {
        let mut app = build_app();
        send_died(&mut app, EnemyType::Bat, Vec2::ZERO, 3);
        run_system(&mut app);

        let gs = gems(&mut app);
        assert_eq!(gs.len(), 1, "exactly one gem should be spawned");
        assert_eq!(gs[0].0.value, 3, "bat gem must have correct XP value");
    }

    /// Boss death spawns a gem with the high XP value.
    #[test]
    fn boss_death_spawns_gem_with_high_value() {
        let mut app = build_app();
        send_died(&mut app, EnemyType::BossDeath, Vec2::ZERO, 500);
        run_system(&mut app);

        let gs = gems(&mut app);
        assert_eq!(gs.len(), 1);
        assert_eq!(gs[0].0.value, 500);
    }

    /// Gem is spawned at the enemy's death position.
    #[test]
    fn gem_spawned_at_death_position() {
        let mut app = build_app();
        let pos = Vec2::new(123.0, -456.0);
        send_died(&mut app, EnemyType::Skeleton, pos, 5);
        run_system(&mut app);

        let gs = gems(&mut app);
        assert_eq!(gs.len(), 1);
        let translation = gs[0].1.translation;
        assert!(
            (translation.x - pos.x).abs() < 0.001,
            "gem x should match death position"
        );
        assert!(
            (translation.y - pos.y).abs() < 0.001,
            "gem y should match death position"
        );
    }

    /// Each EnemyDiedEvent produces exactly one gem.
    #[test]
    fn multiple_deaths_spawn_multiple_gems() {
        let mut app = build_app();
        send_died(&mut app, EnemyType::Bat, Vec2::new(0.0, 0.0), 3);
        send_died(&mut app, EnemyType::Skeleton, Vec2::new(100.0, 0.0), 5);
        send_died(&mut app, EnemyType::Zombie, Vec2::new(200.0, 0.0), 8);
        run_system(&mut app);

        assert_eq!(gems(&mut app).len(), 3, "three deaths → three gems");
    }

    /// The gem's value equals exactly what was carried in the event.
    #[test]
    fn gem_value_equals_event_xp_value() {
        let mut app = build_app();
        send_died(&mut app, EnemyType::Dragon, Vec2::ZERO, 15);
        run_system(&mut app);

        let gs = gems(&mut app);
        assert_eq!(gs[0].0.value, 15);
    }

    /// Gem has a Sprite component (placeholder visual).
    #[test]
    fn gem_has_sprite_component() {
        let mut app = build_app();
        send_died(&mut app, EnemyType::Bat, Vec2::ZERO, 3);
        run_system(&mut app);

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, (With<ExperienceGem>, With<Sprite>)>();
        assert_eq!(
            q.iter(app.world()).count(),
            1,
            "gem entity must have a Sprite component"
        );
    }
}
