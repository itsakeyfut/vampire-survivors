//! XP gem drop system.
//!
//! [`spawn_xp_gems`] listens for [`EnemyDiedEvent`] and spawns an
//! [`ExperienceGem`] entity at the enemy's last known position.
//!
//! The gem value is determined by the enemy's type via [`xp_value_for`],
//! using the same base values as [`Enemy::from_type`] in
//! [`crate::components::enemy`].  Gems are rendered as a green circle
//! placeholder sprite until real art is available.

use bevy::{prelude::*, state::state_scoped::DespawnOnExit};

use crate::{
    components::ExperienceGem, events::EnemyDiedEvent, states::AppState, types::EnemyType,
};

// ---------------------------------------------------------------------------
// Fallback XP values (must match DEFAULT_ENEMY_STATS_* in components/enemy.rs)
// ---------------------------------------------------------------------------

const DEFAULT_XP_BAT: u32 = 3;
const DEFAULT_XP_SKELETON: u32 = 5;
const DEFAULT_XP_ZOMBIE: u32 = 8;
const DEFAULT_XP_GHOST: u32 = 6;
const DEFAULT_XP_DEMON: u32 = 10;
const DEFAULT_XP_MEDUSA: u32 = 8;
const DEFAULT_XP_DRAGON: u32 = 15;
const DEFAULT_XP_BOSS_DEATH: u32 = 500;

/// Gem visual radius in pixels (placeholder; replace with real sprite later).
const GEM_RADIUS: f32 = 6.0;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns the base XP value for the given enemy type.
fn xp_value_for(enemy_type: EnemyType) -> u32 {
    match enemy_type {
        EnemyType::Bat => DEFAULT_XP_BAT,
        EnemyType::Skeleton => DEFAULT_XP_SKELETON,
        EnemyType::Zombie => DEFAULT_XP_ZOMBIE,
        EnemyType::Ghost => DEFAULT_XP_GHOST,
        EnemyType::Demon => DEFAULT_XP_DEMON,
        EnemyType::Medusa => DEFAULT_XP_MEDUSA,
        EnemyType::Dragon => DEFAULT_XP_DRAGON,
        EnemyType::BossDeath => DEFAULT_XP_BOSS_DEATH,
    }
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Spawns an [`ExperienceGem`] at the death position for every
/// [`EnemyDiedEvent`] received this frame.
///
/// - Gem value is set from [`xp_value_for`] using the event's `enemy_type`.
/// - Rendered as a green circle placeholder sprite (`GEM_RADIUS` px radius).
/// - Tagged with [`DespawnOnExit`]`(`[`AppState::Playing`]`)` so gems are
///   automatically cleaned up when the run ends.
///
/// Must run after [`apply_damage_to_enemies`](crate::systems::damage::apply_damage_to_enemies)
/// so that `EnemyDiedEvent` messages are already written before this system
/// reads them.
pub fn spawn_xp_gems(mut commands: Commands, mut died_events: MessageReader<EnemyDiedEvent>) {
    for event in died_events.read() {
        let value = xp_value_for(event.enemy_type);
        commands.spawn((
            DespawnOnExit(AppState::Playing),
            ExperienceGem { value },
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

    fn send_died(app: &mut App, enemy_type: EnemyType, position: Vec2) {
        let entity = app.world_mut().spawn_empty().id();
        app.world_mut().write_message(EnemyDiedEvent {
            entity,
            position,
            enemy_type,
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

    /// Bat death spawns a gem with the correct XP value.
    #[test]
    fn bat_death_spawns_gem_with_correct_value() {
        let mut app = build_app();
        send_died(&mut app, EnemyType::Bat, Vec2::ZERO);
        run_system(&mut app);

        let gs = gems(&mut app);
        assert_eq!(gs.len(), 1, "exactly one gem should be spawned");
        assert_eq!(
            gs[0].0.value, DEFAULT_XP_BAT,
            "bat gem must have correct XP value"
        );
    }

    /// Boss death spawns a gem with the high XP value.
    #[test]
    fn boss_death_spawns_gem_with_high_value() {
        let mut app = build_app();
        send_died(&mut app, EnemyType::BossDeath, Vec2::ZERO);
        run_system(&mut app);

        let gs = gems(&mut app);
        assert_eq!(gs.len(), 1);
        assert_eq!(gs[0].0.value, DEFAULT_XP_BOSS_DEATH);
    }

    /// Gem is spawned at the enemy's death position.
    #[test]
    fn gem_spawned_at_death_position() {
        let mut app = build_app();
        let pos = Vec2::new(123.0, -456.0);
        send_died(&mut app, EnemyType::Skeleton, pos);
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
        send_died(&mut app, EnemyType::Bat, Vec2::new(0.0, 0.0));
        send_died(&mut app, EnemyType::Skeleton, Vec2::new(100.0, 0.0));
        send_died(&mut app, EnemyType::Zombie, Vec2::new(200.0, 0.0));
        run_system(&mut app);

        assert_eq!(gems(&mut app).len(), 3, "three deaths → three gems");
    }

    /// All enemy types produce gems with positive XP values.
    #[test]
    fn all_enemy_types_produce_positive_xp() {
        use EnemyType::*;
        for et in [
            Bat, Skeleton, Zombie, Ghost, Demon, Medusa, Dragon, BossDeath,
        ] {
            assert!(xp_value_for(et) > 0, "{et:?} must drop at least 1 XP");
        }
    }

    /// Gem has a Sprite component (placeholder visual).
    #[test]
    fn gem_has_sprite_component() {
        let mut app = build_app();
        send_died(&mut app, EnemyType::Bat, Vec2::ZERO);
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
