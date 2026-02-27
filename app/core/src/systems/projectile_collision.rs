//! Projectile vs enemy collision detection.
//!
//! [`projectile_enemy_collision`] runs once per frame after the spatial grid
//! has been updated and projectiles have moved.  For each live projectile it
//! queries the [`SpatialGrid`] for candidate enemies, performs exact circle
//! overlap checks, and — on a hit — emits a [`DamageEnemyEvent`].
//!
//! # Piercing behaviour
//!
//! A projectile's `piercing` counter controls how many *additional* enemies it
//! penetrates after the first hit:
//!
//! - `piercing == 0` → single-hit; the projectile is despawned on the first hit.
//! - `piercing >= 1` → multi-hit; each hit decrements `piercing` by 1 and
//!   records the struck enemy in `hit_enemies` to prevent a double-hit on the
//!   same target.  The projectile is despawned only when `piercing` reaches 0
//!   and the next enemy is hit.

use bevy::prelude::*;

use crate::{
    components::{CircleCollider, Enemy, Projectile},
    events::DamageEnemyEvent,
    resources::SpatialGrid,
    systems::collision::check_circle_collision,
};

// ---------------------------------------------------------------------------
// Fallback constant
// ---------------------------------------------------------------------------

/// Conservative upper bound on any enemy's collider radius (pixels).
///
/// Used to widen the spatial-grid query so that large enemies whose centres
/// lie just outside the projectile's radius are still returned as candidates.
const MAX_ENEMY_COLLIDER_RADIUS: f32 = 32.0;

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Detects projectile–enemy overlaps and emits [`DamageEnemyEvent`] on hits.
///
/// For each projectile the system:
/// 1. Queries the [`SpatialGrid`] with `proj_radius + MAX_ENEMY_COLLIDER_RADIUS`
///    to obtain candidate entities (may include false positives).
/// 2. Skips candidates already present in `projectile.hit_enemies`.
/// 3. Performs an exact [`check_circle_collision`] check.
/// 4. On a hit, writes a [`DamageEnemyEvent`] and either:
///    - despawns the projectile (`piercing == 0`), or
///    - records the enemy in `hit_enemies` and decrements `piercing`.
pub fn projectile_enemy_collision(
    mut projectile_q: Query<(Entity, &mut Projectile, &Transform, &CircleCollider)>,
    enemy_q: Query<(&Transform, &CircleCollider), With<Enemy>>,
    spatial_grid: Res<SpatialGrid>,
    mut damage_events: MessageWriter<DamageEnemyEvent>,
    mut commands: Commands,
) {
    for (proj_entity, mut projectile, proj_tf, proj_collider) in projectile_q.iter_mut() {
        let proj_pos = proj_tf.translation.truncate();
        let query_radius = proj_collider.radius + MAX_ENEMY_COLLIDER_RADIUS;

        let candidates = spatial_grid.get_nearby(proj_pos, query_radius);

        let mut should_despawn = false;

        for candidate in candidates {
            if projectile.hit_enemies.contains(&candidate) {
                continue;
            }

            let Ok((enemy_tf, enemy_collider)) = enemy_q.get(candidate) else {
                continue;
            };

            let enemy_pos = enemy_tf.translation.truncate();
            if !check_circle_collision(
                proj_pos,
                proj_collider.radius,
                enemy_pos,
                enemy_collider.radius,
            ) {
                continue;
            }

            // Hit confirmed — emit damage event.
            damage_events.write(DamageEnemyEvent {
                entity: candidate,
                damage: projectile.damage,
                weapon_type: projectile.weapon_type,
            });

            if projectile.piercing == 0 {
                should_despawn = true;
                break;
            } else {
                projectile.hit_enemies.push(candidate);
                projectile.piercing -= 1;
            }
        }

        if should_despawn {
            commands.entity(proj_entity).despawn();
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
        components::{CircleCollider, Enemy, Projectile, ProjectileVelocity},
        events::DamageEnemyEvent,
        resources::SpatialGrid,
        types::{EnemyType, WeaponType},
    };

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<DamageEnemyEvent>();
        app.insert_resource(SpatialGrid::default());
        app
    }

    fn spawn_projectile(app: &mut App, pos: Vec2, radius: f32, piercing: u32) -> Entity {
        app.world_mut()
            .spawn((
                Projectile {
                    damage: 10.0,
                    piercing,
                    hit_enemies: Vec::new(),
                    lifetime: 5.0,
                    weapon_type: WeaponType::MagicWand,
                },
                ProjectileVelocity(Vec2::ZERO),
                Transform::from_xyz(pos.x, pos.y, 5.0),
                CircleCollider { radius },
            ))
            .id()
    }

    fn spawn_enemy(app: &mut App, pos: Vec2, radius: f32) -> Entity {
        app.world_mut()
            .spawn((
                Enemy::from_type(EnemyType::Bat, 1.0),
                Transform::from_xyz(pos.x, pos.y, 1.0),
                CircleCollider { radius },
            ))
            .id()
    }

    /// Populate the SpatialGrid with all current enemy positions.
    fn update_grid(app: &mut App) {
        use crate::systems::spatial::update_spatial_grid;
        app.world_mut()
            .run_system_once(update_spatial_grid)
            .expect("update_spatial_grid should run");
    }

    fn run_collision(app: &mut App) {
        app.world_mut()
            .run_system_once(projectile_enemy_collision)
            .expect("projectile_enemy_collision should run");
    }

    fn damage_events(app: &App) -> Vec<DamageEnemyEvent> {
        let messages = app.world().resource::<Messages<DamageEnemyEvent>>();
        let mut cursor = messages.get_cursor();
        cursor.read(messages).cloned().collect()
    }

    // -----------------------------------------------------------------------
    // Tests
    // -----------------------------------------------------------------------

    /// A projectile overlapping an enemy emits a DamageEnemyEvent.
    #[test]
    fn hit_emits_damage_event() {
        let mut app = build_app();
        let enemy = spawn_enemy(&mut app, Vec2::new(5.0, 0.0), 10.0);
        spawn_projectile(&mut app, Vec2::ZERO, 8.0, 0);

        update_grid(&mut app);
        run_collision(&mut app);

        let events = damage_events(&app);
        assert_eq!(events.len(), 1, "expected one damage event on hit");
        assert_eq!(events[0].entity, enemy);
        assert_eq!(events[0].damage, 10.0);
    }

    /// A non-piercing projectile (piercing == 0) is despawned after the first hit.
    #[test]
    fn non_piercing_projectile_despawns_on_hit() {
        let mut app = build_app();
        spawn_enemy(&mut app, Vec2::new(5.0, 0.0), 10.0);
        let proj = spawn_projectile(&mut app, Vec2::ZERO, 8.0, 0);

        update_grid(&mut app);
        run_collision(&mut app);
        app.world_mut().flush();

        assert!(
            app.world().get_entity(proj).is_err(),
            "non-piercing projectile should be despawned after hit"
        );
    }

    /// A piercing projectile (piercing == 1) survives the first hit and is
    /// despawned only after it hits the second enemy.
    #[test]
    fn piercing_projectile_survives_first_hit() {
        let mut app = build_app();
        spawn_enemy(&mut app, Vec2::new(5.0, 0.0), 10.0);
        let proj = spawn_projectile(&mut app, Vec2::ZERO, 8.0, 1);

        update_grid(&mut app);
        run_collision(&mut app);
        app.world_mut().flush();

        assert!(
            app.world().get_entity(proj).is_ok(),
            "piercing projectile should survive its first hit"
        );
        // Piercing counter decremented.
        let p = app.world().get::<Projectile>(proj).unwrap();
        assert_eq!(p.piercing, 0);
        assert_eq!(p.hit_enemies.len(), 1);
    }

    /// A piercing projectile is despawned after exhausting its pierce count.
    #[test]
    fn piercing_projectile_despawns_after_exhausting_pierce() {
        let mut app = build_app();
        // Two enemies at the same position — both overlap the projectile.
        spawn_enemy(&mut app, Vec2::new(5.0, 0.0), 10.0);
        spawn_enemy(&mut app, Vec2::new(5.0, 0.0), 10.0);
        let proj = spawn_projectile(&mut app, Vec2::ZERO, 8.0, 1);

        update_grid(&mut app);
        run_collision(&mut app);
        app.world_mut().flush();

        assert!(
            app.world().get_entity(proj).is_err(),
            "projectile with piercing=1 should be despawned after hitting two enemies"
        );
        let events = damage_events(&app);
        assert_eq!(events.len(), 2, "both enemies should have been hit");
    }

    /// An already-hit enemy is not damaged again by the same projectile.
    #[test]
    fn already_hit_enemy_is_not_hit_again() {
        let mut app = build_app();
        let enemy = spawn_enemy(&mut app, Vec2::new(5.0, 0.0), 10.0);
        let proj = spawn_projectile(&mut app, Vec2::ZERO, 8.0, 2);

        // Pre-populate hit_enemies so the enemy is already recorded.
        app.world_mut()
            .get_mut::<Projectile>(proj)
            .unwrap()
            .hit_enemies
            .push(enemy);

        update_grid(&mut app);
        run_collision(&mut app);

        let events = damage_events(&app);
        assert!(
            events.is_empty(),
            "enemy already in hit_enemies must not be hit again"
        );
    }

    /// A projectile that does not overlap any enemy emits no events.
    #[test]
    fn miss_emits_no_event() {
        let mut app = build_app();
        // Enemy far away — no collision.
        spawn_enemy(&mut app, Vec2::new(500.0, 0.0), 10.0);
        spawn_projectile(&mut app, Vec2::ZERO, 8.0, 0);

        update_grid(&mut app);
        run_collision(&mut app);

        assert!(
            damage_events(&app).is_empty(),
            "miss should produce no events"
        );
    }

    /// No enemies in the world — system completes without panic.
    #[test]
    fn no_enemies_no_events() {
        let mut app = build_app();
        spawn_projectile(&mut app, Vec2::ZERO, 8.0, 0);

        update_grid(&mut app);
        run_collision(&mut app);

        assert!(damage_events(&app).is_empty());
    }

    /// No projectiles in the world — system completes without panic.
    #[test]
    fn no_projectiles_no_events() {
        let mut app = build_app();
        spawn_enemy(&mut app, Vec2::ZERO, 10.0);

        update_grid(&mut app);
        run_collision(&mut app);

        assert!(damage_events(&app).is_empty());
    }
}
