//! Projectile movement, lifetime management, and spawn helper.
//!
//! Two per-frame systems run in [`AppState::Playing`]:
//!
//! - [`move_projectiles`] — translates every projectile by its
//!   [`ProjectileVelocity`] each frame.
//! - [`despawn_expired_projectiles`] — decrements each projectile's
//!   `lifetime` field and calls `despawn()` when it reaches zero.
//!
//! The free function [`spawn_projectile`] is a thin helper used by
//! weapon-specific fire systems (tasks 4.4 / 4.5) to spawn a fully
//! equipped projectile entity without duplicating component boilerplate.

use bevy::{prelude::*, state::state_scoped::DespawnOnExit};

use crate::{
    components::{CircleCollider, Projectile, ProjectileVelocity},
    states::AppState,
    types::WeaponType,
};

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Advances every active projectile along its velocity vector.
///
/// Requires [`Projectile`], [`ProjectileVelocity`], and [`Transform`].
pub fn move_projectiles(
    mut query: Query<(&mut Transform, &ProjectileVelocity), With<Projectile>>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += (velocity.0 * delta).extend(0.0);
    }
}

/// Counts down each projectile's remaining lifetime and despawns it when
/// the timer reaches zero.
///
/// Uses immediate `despawn()` — children are not expected on bare
/// projectiles.
pub fn despawn_expired_projectiles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Projectile)>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    for (entity, mut projectile) in query.iter_mut() {
        projectile.lifetime -= delta;
        if projectile.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

// ---------------------------------------------------------------------------
// Spawn helper
// ---------------------------------------------------------------------------

/// Spawns a single [`Projectile`] entity traveling in `velocity`.
///
/// The entity automatically carries [`DespawnOnExit`] so it is cleaned
/// up when the game leaves [`AppState::Playing`].
///
/// # Parameters
/// - `position` — World-space spawn point (pixels).
/// - `velocity` — Direction × speed in pixels/second.
/// - `damage` — Base damage per hit (before `damage_multiplier`).
/// - `lifetime` — Maximum life in seconds; despawned when it reaches 0.
/// - `piercing` — Extra enemies this projectile penetrates past the first
///   (`0` = normal single-hit behaviour).
/// - `collider_radius` — Circle collider radius for hit detection (pixels).
/// - `weapon_type` — Source weapon; used by downstream collision systems.
///
/// # Returns
/// The [`Entity`] ID of the newly spawned projectile.
#[allow(clippy::too_many_arguments)]
pub fn spawn_projectile(
    commands: &mut Commands,
    position: Vec2,
    velocity: Vec2,
    damage: f32,
    lifetime: f32,
    piercing: u32,
    collider_radius: f32,
    weapon_type: WeaponType,
) -> Entity {
    commands
        .spawn((
            DespawnOnExit(AppState::Playing),
            Projectile {
                damage,
                piercing,
                hit_enemies: Vec::new(),
                lifetime,
                weapon_type,
            },
            ProjectileVelocity(velocity),
            CircleCollider {
                radius: collider_radius,
            },
            // Warm-yellow placeholder sprite; replace with real sprite later.
            Sprite {
                color: Color::srgb(1.0, 0.95, 0.3),
                custom_size: Some(Vec2::splat(collider_radius * 2.0)),
                ..default()
            },
            // z = 5.0 — above enemies (z ≈ 1) but below the player (z = 10).
            Transform::from_xyz(position.x, position.y, 5.0),
        ))
        .id()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::ecs::system::RunSystemOnce as _;

    use super::*;
    use crate::types::WeaponType;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app
    }

    fn advance_and_run_move(app: &mut App, delta_secs: f32) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(delta_secs));
        app.world_mut()
            .run_system_once(move_projectiles)
            .expect("move_projectiles should run");
    }

    fn advance_and_run_despawn(app: &mut App, delta_secs: f32) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(delta_secs));
        app.world_mut()
            .run_system_once(despawn_expired_projectiles)
            .expect("despawn_expired_projectiles should run");
    }

    fn spawn_test_projectile(app: &mut App, velocity: Vec2, lifetime: f32) -> Entity {
        app.world_mut()
            .spawn((
                Projectile {
                    damage: 10.0,
                    piercing: 0,
                    hit_enemies: Vec::new(),
                    lifetime,
                    weapon_type: WeaponType::MagicWand,
                },
                ProjectileVelocity(velocity),
                Transform::from_xyz(0.0, 0.0, 5.0),
            ))
            .id()
    }

    // -----------------------------------------------------------------------
    // move_projectiles tests
    // -----------------------------------------------------------------------

    /// Each frame the projectile moves by `velocity * delta`.
    #[test]
    fn projectile_moves_by_velocity() {
        let mut app = build_app();
        let entity = spawn_test_projectile(&mut app, Vec2::new(300.0, 0.0), 5.0);

        let delta = 1.0 / 60.0_f32;
        advance_and_run_move(&mut app, delta);

        let transform = app.world().get::<Transform>(entity).unwrap();
        let expected_x = 300.0 * delta;
        assert!(
            (transform.translation.x - expected_x).abs() < 1e-4,
            "expected x ≈ {expected_x:.4}, got {:.4}",
            transform.translation.x
        );
        assert!(
            transform.translation.y.abs() < 1e-6,
            "y should remain 0, got {}",
            transform.translation.y
        );
    }

    /// A stationary projectile (velocity = zero) stays in place.
    #[test]
    fn stationary_projectile_does_not_move() {
        let mut app = build_app();
        let entity = spawn_test_projectile(&mut app, Vec2::ZERO, 5.0);

        advance_and_run_move(&mut app, 1.0 / 60.0);

        let transform = app.world().get::<Transform>(entity).unwrap();
        assert!(
            transform.translation.xy().length() < 1e-6,
            "projectile with zero velocity should not move, got {:?}",
            transform.translation
        );
    }

    /// Diagonal movement: both x and y advance proportionally.
    #[test]
    fn projectile_moves_diagonally() {
        let mut app = build_app();
        let vel = Vec2::new(200.0, 150.0);
        let entity = spawn_test_projectile(&mut app, vel, 5.0);

        let delta = 1.0 / 60.0_f32;
        advance_and_run_move(&mut app, delta);

        let transform = app.world().get::<Transform>(entity).unwrap();
        assert!(
            (transform.translation.x - vel.x * delta).abs() < 1e-4,
            "x mismatch"
        );
        assert!(
            (transform.translation.y - vel.y * delta).abs() < 1e-4,
            "y mismatch"
        );
    }

    // -----------------------------------------------------------------------
    // despawn_expired_projectiles tests
    // -----------------------------------------------------------------------

    /// A projectile whose lifetime drops to or below 0 is despawned.
    #[test]
    fn expired_projectile_is_despawned() {
        let mut app = build_app();
        // lifetime very small — one frame at 60 fps will expire it.
        let entity = spawn_test_projectile(&mut app, Vec2::ZERO, 0.001);

        advance_and_run_despawn(&mut app, 1.0 / 60.0);

        // Commands are flushed between system runs; flush manually.
        app.world_mut().flush();

        assert!(
            app.world().get_entity(entity).is_err(),
            "entity should have been despawned"
        );
    }

    /// A projectile with remaining lifetime is not despawned.
    #[test]
    fn active_projectile_is_not_despawned() {
        let mut app = build_app();
        let entity = spawn_test_projectile(&mut app, Vec2::ZERO, 10.0);

        advance_and_run_despawn(&mut app, 1.0 / 60.0);
        app.world_mut().flush();

        assert!(
            app.world().get_entity(entity).is_ok(),
            "entity should still be alive"
        );
    }

    /// Two projectiles are tracked independently: only the expired one is removed.
    #[test]
    fn only_expired_projectile_is_despawned() {
        let mut app = build_app();
        let dying = spawn_test_projectile(&mut app, Vec2::ZERO, 0.001);
        let alive = spawn_test_projectile(&mut app, Vec2::ZERO, 99.0);

        advance_and_run_despawn(&mut app, 1.0 / 60.0);
        app.world_mut().flush();

        assert!(
            app.world().get_entity(dying).is_err(),
            "dying projectile should have been despawned"
        );
        assert!(
            app.world().get_entity(alive).is_ok(),
            "alive projectile should remain"
        );
    }

    /// `spawn_projectile` creates an entity with the required components.
    #[test]
    fn spawn_projectile_creates_entity_with_components() {
        let mut app = build_app();

        let entity = {
            let mut commands = app.world_mut().commands();
            spawn_projectile(
                &mut commands,
                Vec2::new(10.0, 20.0),
                Vec2::new(300.0, 0.0),
                15.0, // damage
                5.0,  // lifetime
                1,    // piercing
                5.0,  // collider_radius
                WeaponType::MagicWand,
            )
        };
        app.world_mut().flush();

        assert!(
            app.world().get::<Projectile>(entity).is_some(),
            "should have Projectile component"
        );
        assert!(
            app.world().get::<ProjectileVelocity>(entity).is_some(),
            "should have ProjectileVelocity component"
        );
        assert!(
            app.world().get::<Transform>(entity).is_some(),
            "should have Transform component"
        );
        assert!(
            app.world().get::<CircleCollider>(entity).is_some(),
            "should have CircleCollider component"
        );

        let proj = app.world().get::<Projectile>(entity).unwrap();
        assert_eq!(proj.damage, 15.0);
        assert_eq!(proj.piercing, 1);
        assert_eq!(proj.weapon_type, WeaponType::MagicWand);

        let vel = app.world().get::<ProjectileVelocity>(entity).unwrap();
        assert_eq!(vel.0, Vec2::new(300.0, 0.0));

        let tf = app.world().get::<Transform>(entity).unwrap();
        assert_eq!(tf.translation.x, 10.0);
        assert_eq!(tf.translation.y, 20.0);
    }
}
