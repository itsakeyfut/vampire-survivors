//! Fire Wand weapon — fireball targeting the highest-HP enemy with AoE explosion.
//!
//! The Fire Wand fires a slow, large fireball aimed at the enemy with the
//! highest current HP.  On impact with any enemy the fireball explodes,
//! dealing full damage to the struck enemy and reduced area damage to all
//! other enemies within the explosion radius.
//!
//! ## Level progression
//!
//! | Level | Direct | AoE   | AoE Radius (px) |
//! |-------|--------|-------|-----------------|
//! | 1     | 80     | 40    | 80              |
//! | 2     | 100    | 50    | 90              |
//! | 3     | 120    | 60    | 100             |
//! | 4     | 150    | 75    | 110             |
//! | 5     | 180    | 90    | 120             |
//! | 6     | 220    | 110   | 130             |
//! | 7     | 270    | 135   | 140             |
//! | 8     | 330    | 165   | 150             |
//!
//! ## Targeting
//!
//! Every activation scans all living enemies and targets the one with the
//! highest `current_hp`.  The fireball travels in a straight line at constant
//! speed.  Because enemies move, the fireball may not always reach its
//! original target — it will explode on whichever enemy it first contacts.
//!
//! ## Explosion
//!
//! [`fireball_enemy_collision`] runs after [`move_fireballs`] each frame.  On
//! the first contact it:
//! 1. Emits a [`DamageEnemyEvent`] for the struck enemy (full damage).
//! 2. Queries the [`SpatialGrid`] and emits [`DamageEnemyEvent`] for every
//!    other enemy within the AoE radius (reduced damage).
//! 3. Spawns a short-lived [`FireballExplosionEffect`] visual at the hit point.
//! 4. Despawns the fireball entity.
//!
//! [`move_fireballs`]: self::move_fireballs

use bevy::prelude::*;

use crate::{
    components::{
        CircleCollider, Enemy, GameSessionEntity, Player, PlayerStats, ProjectileVelocity,
    },
    config::weapon::fire_wand::FireWandParams,
    events::{DamageEnemyEvent, WeaponFiredEvent},
    resources::SpatialGrid,
    systems::collision::check_circle_collision,
    types::WeaponType,
};

// ---------------------------------------------------------------------------
// Fallback constants (used while RON config is still loading)
// ---------------------------------------------------------------------------

/// Direct hit damage per level (index 0 = level 1).
const DEFAULT_FIRE_WAND_DAMAGE_BY_LEVEL: [f32; 8] =
    [80.0, 100.0, 120.0, 150.0, 180.0, 220.0, 270.0, 330.0];
/// AoE explosion damage per level.
const DEFAULT_FIRE_WAND_AOE_DAMAGE_BY_LEVEL: [f32; 8] =
    [40.0, 50.0, 60.0, 75.0, 90.0, 110.0, 135.0, 165.0];
/// AoE explosion radius (pixels) per level.
const DEFAULT_FIRE_WAND_AOE_RADIUS_BY_LEVEL: [f32; 8] =
    [80.0, 90.0, 100.0, 110.0, 120.0, 130.0, 140.0, 150.0];
/// Fireball travel speed in pixels/second.
const DEFAULT_FIRE_WAND_SPEED: f32 = 250.0;
/// Fireball lifetime in seconds before automatic despawn.
const DEFAULT_FIRE_WAND_LIFETIME: f32 = 3.0;
/// Fireball collider radius for first-contact detection.
const DEFAULT_FIRE_WAND_COLLIDER_RADIUS: f32 = 12.0;
/// Duration the explosion visual remains on screen.
const DEFAULT_FIRE_WAND_EXPLOSION_DURATION: f32 = 0.3;
/// RGBA colour of the explosion visual.
const DEFAULT_FIRE_WAND_EXPLOSION_COLOR: (f32, f32, f32, f32) = (1.0, 0.4, 0.1, 0.8);
/// Z-depth of the explosion visual.
const DEFAULT_FIRE_WAND_EXPLOSION_Z: f32 = 7.0;
/// Fireball sprite z-depth.
const FIRE_WAND_PROJECTILE_Z: f32 = 5.5;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Carried by every fireball entity spawned by [`fire_fire_wand`].
///
/// Because fireballs explode on impact rather than piercing, they do not use
/// the standard [`Projectile`](crate::components::Projectile) component.
/// Instead [`move_fireballs`] and [`fireball_enemy_collision`] handle their
/// movement and hit detection separately.
#[derive(Component, Debug)]
pub struct FireballProjectile {
    /// Direct damage dealt to the first enemy struck.
    pub damage: f32,
    /// Area-of-effect damage dealt to all other enemies in the blast radius.
    pub aoe_damage: f32,
    /// Explosion radius in pixels (scaled by `player.area_multiplier`).
    pub aoe_radius: f32,
    /// Remaining lifetime in seconds; the fireball auto-despawns at zero.
    pub lifetime: f32,
}

/// Short-lived explosion visual spawned by [`fireball_enemy_collision`].
///
/// [`despawn_explosion_effects`] ticks this down each frame and removes the
/// entity when `remaining` reaches zero.
#[derive(Component, Debug)]
pub struct FireballExplosionEffect {
    /// Remaining display time in seconds.
    pub remaining: f32,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Fires a [`FireballProjectile`] toward the enemy with the highest current HP
/// when a [`WeaponFiredEvent`] for [`WeaponType::FireWand`] arrives.
///
/// If no enemies are present or all enemies are exactly on the player, the
/// event is silently consumed.
///
/// [`PlayerStats::area_multiplier`] scales the explosion radius; all other
/// damage and speed values use their respective multipliers.
pub fn fire_fire_wand(
    mut fired_events: MessageReader<WeaponFiredEvent>,
    mut commands: Commands,
    player_q: Query<(&Transform, &PlayerStats), With<Player>>,
    enemy_q: Query<(&Transform, &Enemy)>,
    fire_wand_cfg: FireWandParams,
) {
    let cfg = fire_wand_cfg.get();
    let speed = cfg.map(|c| c.speed).unwrap_or(DEFAULT_FIRE_WAND_SPEED);
    let lifetime = cfg
        .map(|c| c.lifetime)
        .unwrap_or(DEFAULT_FIRE_WAND_LIFETIME);
    let collider_r = cfg
        .map(|c| c.collider_radius)
        .unwrap_or(DEFAULT_FIRE_WAND_COLLIDER_RADIUS);

    for event in fired_events.read() {
        if event.weapon_type != WeaponType::FireWand {
            continue;
        }

        let Ok((player_tf, stats)) = player_q.get(event.player) else {
            continue;
        };

        let player_pos = player_tf.translation.truncate();
        let level = event.level.clamp(1, 8) as usize;

        let damage = cfg
            .and_then(|c| c.damage_by_level.get(level - 1).copied())
            .unwrap_or(DEFAULT_FIRE_WAND_DAMAGE_BY_LEVEL[level - 1])
            * stats.damage_multiplier;

        let aoe_damage = cfg
            .and_then(|c| c.aoe_damage_by_level.get(level - 1).copied())
            .unwrap_or(DEFAULT_FIRE_WAND_AOE_DAMAGE_BY_LEVEL[level - 1])
            * stats.damage_multiplier;

        let aoe_radius = cfg
            .and_then(|c| c.aoe_radius_by_level.get(level - 1).copied())
            .unwrap_or(DEFAULT_FIRE_WAND_AOE_RADIUS_BY_LEVEL[level - 1])
            * stats.area_multiplier;

        // Target the enemy with the highest current HP; skip any enemy
        // exactly on the player so direction is always non-zero.
        let target = enemy_q
            .iter()
            .filter(|(tf, _)| tf.translation.truncate().distance_squared(player_pos) > f32::EPSILON)
            .max_by(|(_, a), (_, b)| {
                a.current_hp
                    .partial_cmp(&b.current_hp)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

        let Some((target_tf, _)) = target else {
            continue; // no targetable enemies
        };

        let target_pos = target_tf.translation.truncate();
        let dir = (target_pos - player_pos).normalize_or_zero();
        if dir == Vec2::ZERO {
            continue;
        }

        let velocity = dir * speed * stats.projectile_speed_mult;

        commands.spawn((
            GameSessionEntity,
            FireballProjectile {
                damage,
                aoe_damage,
                aoe_radius,
                lifetime,
            },
            ProjectileVelocity(velocity),
            CircleCollider { radius: collider_r },
            // Orange-red fireball placeholder sprite.
            Sprite {
                color: Color::srgb(1.0, 0.4, 0.1),
                custom_size: Some(Vec2::splat(collider_r * 2.0)),
                ..default()
            },
            Transform::from_xyz(player_pos.x, player_pos.y, FIRE_WAND_PROJECTILE_Z),
        ));
    }
}

/// Advances every [`FireballProjectile`] entity along its velocity.
///
/// Fireballs do not use the standard `move_projectiles` system because they
/// carry [`FireballProjectile`] instead of the standard `Projectile` marker.
pub fn move_fireballs(
    mut query: Query<(&mut Transform, &ProjectileVelocity), With<FireballProjectile>>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += (velocity.0 * delta).extend(0.0);
    }
}

/// Detects [`FireballProjectile`]–enemy collisions and triggers explosions.
///
/// For each fireball this system:
/// 1. Queries the [`SpatialGrid`] for nearby enemies.
/// 2. Performs an exact circle-overlap check against each candidate.
/// 3. On the first hit:
///    a. Emits a [`DamageEnemyEvent`] for the struck enemy (full damage).
///    b. Emits [`DamageEnemyEvent`] for all other enemies within `aoe_radius` (AoE damage).
///    c. Spawns a [`FireballExplosionEffect`] visual at the impact point.
///    d. Despawns the fireball entity.
///
/// Must run **after** [`move_fireballs`] and **after** the spatial grid has
/// been updated for this frame.
pub fn fireball_enemy_collision(
    fireball_q: Query<(Entity, &Transform, &CircleCollider, &FireballProjectile)>,
    enemy_q: Query<(&Transform, &CircleCollider), With<Enemy>>,
    spatial_grid: Res<SpatialGrid>,
    fire_wand_cfg: FireWandParams,
    mut damage_events: MessageWriter<DamageEnemyEvent>,
    mut commands: Commands,
) {
    let cfg = fire_wand_cfg.get();
    let explosion_duration = cfg
        .map(|c| c.explosion_duration)
        .unwrap_or(DEFAULT_FIRE_WAND_EXPLOSION_DURATION);
    let (er, eg, eb, ea) = cfg
        .map(|c| c.explosion_color)
        .unwrap_or(DEFAULT_FIRE_WAND_EXPLOSION_COLOR);
    let explosion_z = cfg
        .map(|c| c.explosion_z)
        .unwrap_or(DEFAULT_FIRE_WAND_EXPLOSION_Z);

    // Compute the maximum enemy collider radius from live data so the spatial
    // query never under-shoots for unusually large enemies.
    let max_enemy_r = enemy_q
        .iter()
        .map(|(_, c)| c.radius)
        .fold(0.0_f32, f32::max);

    for (entity, tf, collider, projectile) in fireball_q.iter() {
        let pos = tf.translation.truncate();
        let query_radius = collider.radius + max_enemy_r;
        let candidates = spatial_grid.get_nearby(pos, query_radius);

        // Find the first enemy the fireball overlaps.
        let mut hit: Option<(Entity, Vec2)> = None;
        for candidate in candidates {
            let Ok((enemy_tf, enemy_collider)) = enemy_q.get(candidate) else {
                continue;
            };
            let enemy_pos = enemy_tf.translation.truncate();
            if check_circle_collision(pos, collider.radius, enemy_pos, enemy_collider.radius) {
                hit = Some((candidate, enemy_pos));
                break;
            }
        }

        let Some((hit_entity, explosion_center)) = hit else {
            continue;
        };

        // Emit direct-hit damage.
        damage_events.write(DamageEnemyEvent {
            entity: hit_entity,
            damage: projectile.damage,
            weapon_type: WeaponType::FireWand,
        });

        // AoE: damage all other enemies within the explosion radius.
        let aoe_query_radius = projectile.aoe_radius + max_enemy_r;
        let aoe_candidates = spatial_grid.get_nearby(explosion_center, aoe_query_radius);
        for candidate in aoe_candidates {
            if candidate == hit_entity {
                continue; // already dealt full damage above
            }
            let Ok((enemy_tf, enemy_collider)) = enemy_q.get(candidate) else {
                continue;
            };
            let enemy_pos = enemy_tf.translation.truncate();
            if check_circle_collision(
                explosion_center,
                projectile.aoe_radius,
                enemy_pos,
                enemy_collider.radius,
            ) {
                damage_events.write(DamageEnemyEvent {
                    entity: candidate,
                    damage: projectile.aoe_damage,
                    weapon_type: WeaponType::FireWand,
                });
            }
        }

        // Explosion visual — size matches the AoE radius.
        let visual_size = projectile.aoe_radius * 2.0;
        commands.spawn((
            GameSessionEntity,
            FireballExplosionEffect {
                remaining: explosion_duration,
            },
            Sprite {
                color: Color::srgba(er, eg, eb, ea),
                custom_size: Some(Vec2::splat(visual_size)),
                ..default()
            },
            Transform::from_xyz(explosion_center.x, explosion_center.y, explosion_z),
        ));

        // Despawn the fireball (deferred — safe to call during query iteration).
        commands.entity(entity).despawn();
    }
}

/// Ticks down each [`FireballExplosionEffect`] and despawns it when expired.
pub fn despawn_explosion_effects(
    mut commands: Commands,
    mut query: Query<(Entity, &mut FireballExplosionEffect)>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    for (entity, mut effect) in query.iter_mut() {
        effect.remaining -= delta;
        if effect.remaining <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Ticks down each [`FireballProjectile`]'s lifetime and despawns it when
/// expired (for fireballs that travel off-screen without hitting anything).
pub fn despawn_expired_fireballs(
    mut commands: Commands,
    mut query: Query<(Entity, &mut FireballProjectile)>,
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
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::ecs::system::RunSystemOnce as _;

    use super::*;
    use crate::{
        components::{Enemy, WeaponInventory},
        events::{DamageEnemyEvent, WeaponFiredEvent},
        resources::SpatialGrid,
        types::{EnemyType, WeaponState, WeaponType},
    };

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<WeaponFiredEvent>();
        app.add_message::<DamageEnemyEvent>();
        app.insert_resource(SpatialGrid::default());
        app
    }

    fn advance(app: &mut App) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(1.0 / 60.0));
    }

    fn spawn_player(app: &mut App) -> Entity {
        let mut weapon = WeaponState::new(WeaponType::FireWand);
        weapon.cooldown_timer = 0.0;
        app.world_mut()
            .spawn((
                Player,
                PlayerStats::default(),
                WeaponInventory {
                    weapons: vec![weapon],
                },
                Transform::from_xyz(0.0, 0.0, 10.0),
            ))
            .id()
    }

    fn spawn_enemy_with_hp(app: &mut App, pos: Vec2, hp: f32) -> Entity {
        let mut enemy = Enemy::from_type(EnemyType::Bat, 1.0);
        enemy.current_hp = hp;
        enemy.max_hp = hp;
        app.world_mut()
            .spawn((
                enemy,
                Transform::from_xyz(pos.x, pos.y, 1.0),
                CircleCollider { radius: 10.0 },
            ))
            .id()
    }

    fn update_grid(app: &mut App) {
        use crate::systems::spatial::update_spatial_grid;
        app.world_mut()
            .run_system_once(update_spatial_grid)
            .expect("update_spatial_grid should run");
    }

    fn fireball_count(app: &mut App) -> usize {
        app.world_mut()
            .query::<&FireballProjectile>()
            .iter(app.world())
            .count()
    }

    fn damage_events(app: &App) -> Vec<DamageEnemyEvent> {
        let messages = app.world().resource::<Messages<DamageEnemyEvent>>();
        let mut cursor = messages.get_cursor();
        cursor.read(messages).cloned().collect()
    }

    fn fire_event(app: &mut App, player: Entity, level: u8) {
        app.world_mut().write_message(WeaponFiredEvent {
            player,
            weapon_type: WeaponType::FireWand,
            level,
        });
        app.world_mut()
            .run_system_once(fire_fire_wand)
            .expect("fire_fire_wand should run");
        app.world_mut().flush();
    }

    // -----------------------------------------------------------------------
    // fire_fire_wand tests
    // -----------------------------------------------------------------------

    /// With one enemy present, exactly one fireball is spawned.
    #[test]
    fn fire_fire_wand_spawns_fireball() {
        let mut app = build_app();
        let player = spawn_player(&mut app);
        spawn_enemy_with_hp(&mut app, Vec2::new(200.0, 0.0), 30.0);

        fire_event(&mut app, player, 1);

        assert_eq!(fireball_count(&mut app), 1, "should spawn 1 fireball");
    }

    /// No enemies → no fireball spawned.
    #[test]
    fn fire_fire_wand_no_enemies_no_fireball() {
        let mut app = build_app();
        let player = spawn_player(&mut app);

        fire_event(&mut app, player, 1);

        assert_eq!(fireball_count(&mut app), 0, "no fireball without enemies");
    }

    /// Non-FireWand events are ignored.
    #[test]
    fn fire_fire_wand_ignores_other_weapons() {
        let mut app = build_app();
        let player = spawn_player(&mut app);
        spawn_enemy_with_hp(&mut app, Vec2::new(100.0, 0.0), 50.0);

        app.world_mut().write_message(WeaponFiredEvent {
            player,
            weapon_type: WeaponType::MagicWand,
            level: 1,
        });
        app.world_mut()
            .run_system_once(fire_fire_wand)
            .expect("fire_fire_wand should run");
        app.world_mut().flush();

        assert_eq!(
            fireball_count(&mut app),
            0,
            "fire_fire_wand should ignore non-FireWand events"
        );
    }

    /// The fireball targets the enemy with the highest HP, not the nearest.
    #[test]
    fn fire_fire_wand_targets_highest_hp_enemy() {
        let mut app = build_app();
        let player = spawn_player(&mut app); // at (0, 0)

        // Low-HP enemy is close (right); high-HP enemy is far (left).
        spawn_enemy_with_hp(&mut app, Vec2::new(50.0, 0.0), 10.0);
        spawn_enemy_with_hp(&mut app, Vec2::new(-200.0, 0.0), 100.0);

        fire_event(&mut app, player, 1);

        // Fireball should travel leftward (toward the high-HP enemy).
        let vel = app
            .world_mut()
            .query::<&ProjectileVelocity>()
            .iter(app.world())
            .next()
            .map(|v| v.0)
            .expect("fireball should exist");

        assert!(
            vel.x < 0.0,
            "fireball should aim at the high-HP enemy on the left, vel = {vel:?}"
        );
    }

    /// The spawned fireball carries the correct component values.
    #[test]
    fn fire_fire_wand_fireball_has_correct_data() {
        let mut app = build_app();
        let player = spawn_player(&mut app);
        spawn_enemy_with_hp(&mut app, Vec2::new(100.0, 0.0), 50.0);

        fire_event(&mut app, player, 1);

        let fireball = app
            .world_mut()
            .query::<&FireballProjectile>()
            .iter(app.world())
            .next()
            .expect("fireball should exist");

        assert!(fireball.damage > 0.0, "damage should be positive");
        assert!(fireball.aoe_damage > 0.0, "aoe_damage should be positive");
        assert!(fireball.aoe_radius > 0.0, "aoe_radius should be positive");
        assert!(fireball.lifetime > 0.0, "lifetime should be positive");
    }

    /// Enemy exactly on the player is skipped; a farther one is targeted.
    #[test]
    fn fire_fire_wand_skips_enemy_on_player() {
        let mut app = build_app();
        let player = spawn_player(&mut app); // at (0, 0)

        spawn_enemy_with_hp(&mut app, Vec2::ZERO, 100.0); // on top of player
        spawn_enemy_with_hp(&mut app, Vec2::new(150.0, 0.0), 50.0); // valid target

        fire_event(&mut app, player, 1);

        let vel = app
            .world_mut()
            .query::<&ProjectileVelocity>()
            .iter(app.world())
            .next()
            .map(|v| v.0)
            .expect("fireball should exist");

        assert!(
            vel.x > 0.0,
            "should target the enemy at (150, 0), not the one on the player"
        );
    }

    // -----------------------------------------------------------------------
    // move_fireballs tests
    // -----------------------------------------------------------------------

    /// Fireballs advance along their velocity each frame.
    #[test]
    fn move_fireballs_advances_position() {
        let mut app = build_app();

        let entity = app
            .world_mut()
            .spawn((
                FireballProjectile {
                    damage: 80.0,
                    aoe_damage: 40.0,
                    aoe_radius: 80.0,
                    lifetime: 3.0,
                },
                ProjectileVelocity(Vec2::new(250.0, 0.0)),
                Transform::from_xyz(0.0, 0.0, 5.5),
            ))
            .id();

        let delta = 1.0 / 60.0_f32;
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(delta));
        app.world_mut()
            .run_system_once(move_fireballs)
            .expect("move_fireballs should run");

        let tf = app.world().get::<Transform>(entity).unwrap();
        let expected_x = 250.0 * delta;
        assert!(
            (tf.translation.x - expected_x).abs() < 1e-3,
            "expected x ≈ {expected_x:.4}, got {:.4}",
            tf.translation.x
        );
    }

    // -----------------------------------------------------------------------
    // fireball_enemy_collision tests
    // -----------------------------------------------------------------------

    fn spawn_fireball_at(app: &mut App, pos: Vec2) -> Entity {
        app.world_mut()
            .spawn((
                FireballProjectile {
                    damage: 80.0,
                    aoe_damage: 40.0,
                    aoe_radius: 80.0,
                    lifetime: 3.0,
                },
                CircleCollider { radius: 12.0 },
                Transform::from_xyz(pos.x, pos.y, 5.5),
            ))
            .id()
    }

    fn run_collision(app: &mut App) {
        app.world_mut()
            .run_system_once(fireball_enemy_collision)
            .expect("fireball_enemy_collision should run");
    }

    /// A fireball overlapping an enemy emits a direct DamageEnemyEvent.
    #[test]
    fn fireball_collision_deals_direct_damage() {
        let mut app = build_app();

        let enemy = spawn_enemy_with_hp(&mut app, Vec2::new(5.0, 0.0), 50.0);
        spawn_fireball_at(&mut app, Vec2::ZERO);

        update_grid(&mut app);
        run_collision(&mut app);

        let events = damage_events(&app);
        assert!(
            events.iter().any(|e| e.entity == enemy && e.damage == 80.0),
            "should deal 80 direct damage to the hit enemy, events = {events:?}"
        );
    }

    /// AoE enemies (not the directly hit one) receive AoE damage.
    #[test]
    fn fireball_collision_deals_aoe_damage_to_nearby_enemies() {
        let mut app = build_app();

        // Direct hit enemy at (5, 0).
        let _hit_enemy = spawn_enemy_with_hp(&mut app, Vec2::new(5.0, 0.0), 50.0);
        // AoE enemy within blast radius (80 px).
        let aoe_enemy = spawn_enemy_with_hp(&mut app, Vec2::new(30.0, 0.0), 30.0);

        spawn_fireball_at(&mut app, Vec2::ZERO);

        update_grid(&mut app);
        run_collision(&mut app);

        let events = damage_events(&app);
        assert!(
            events
                .iter()
                .any(|e| e.entity == aoe_enemy && e.damage == 40.0),
            "AoE enemy should receive 40 splash damage, events = {events:?}"
        );
    }

    /// The directly-hit enemy does NOT receive AoE damage on top of direct damage.
    #[test]
    fn fireball_collision_hit_enemy_not_double_damaged() {
        let mut app = build_app();

        let hit_enemy = spawn_enemy_with_hp(&mut app, Vec2::new(5.0, 0.0), 50.0);
        spawn_fireball_at(&mut app, Vec2::ZERO);

        update_grid(&mut app);
        run_collision(&mut app);

        let events = damage_events(&app);
        let hit_count = events.iter().filter(|e| e.entity == hit_enemy).count();
        assert_eq!(
            hit_count, 1,
            "directly hit enemy should receive exactly one damage event, got {hit_count}"
        );
    }

    /// Enemy outside the AoE radius is not damaged.
    #[test]
    fn fireball_collision_enemy_outside_aoe_not_damaged() {
        let mut app = build_app();

        let hit_enemy = spawn_enemy_with_hp(&mut app, Vec2::new(5.0, 0.0), 50.0);
        let far_enemy = spawn_enemy_with_hp(&mut app, Vec2::new(500.0, 0.0), 30.0);

        spawn_fireball_at(&mut app, Vec2::ZERO);

        update_grid(&mut app);
        run_collision(&mut app);

        let events = damage_events(&app);
        assert!(
            !events.iter().any(|e| e.entity == far_enemy),
            "enemy outside aoe_radius should not be damaged"
        );
        // But the direct hit should still happen.
        assert!(events.iter().any(|e| e.entity == hit_enemy));
    }

    /// The fireball is despawned after impacting an enemy.
    #[test]
    fn fireball_collision_despawns_fireball() {
        let mut app = build_app();

        spawn_enemy_with_hp(&mut app, Vec2::new(5.0, 0.0), 50.0);
        let fireball = spawn_fireball_at(&mut app, Vec2::ZERO);

        update_grid(&mut app);
        run_collision(&mut app);
        app.world_mut().flush();

        assert!(
            app.world().get_entity(fireball).is_err(),
            "fireball should be despawned after hitting an enemy"
        );
    }

    /// An explosion visual is spawned at the impact point.
    #[test]
    fn fireball_collision_spawns_explosion_effect() {
        let mut app = build_app();

        spawn_enemy_with_hp(&mut app, Vec2::new(5.0, 0.0), 50.0);
        spawn_fireball_at(&mut app, Vec2::ZERO);

        update_grid(&mut app);
        run_collision(&mut app);
        app.world_mut().flush();

        let count = app
            .world_mut()
            .query::<&FireballExplosionEffect>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "one explosion effect should be spawned on impact");
    }

    /// No collision → no damage events, fireball survives.
    #[test]
    fn fireball_collision_no_hit_no_events() {
        let mut app = build_app();

        spawn_enemy_with_hp(&mut app, Vec2::new(500.0, 0.0), 50.0);
        let fireball = spawn_fireball_at(&mut app, Vec2::ZERO);

        update_grid(&mut app);
        run_collision(&mut app);
        app.world_mut().flush();

        assert!(
            damage_events(&app).is_empty(),
            "miss should produce no damage events"
        );
        assert!(
            app.world().get_entity(fireball).is_ok(),
            "fireball should survive a miss"
        );
    }

    // -----------------------------------------------------------------------
    // despawn_expired_fireballs tests
    // -----------------------------------------------------------------------

    /// Fireball with expired lifetime is despawned.
    #[test]
    fn despawn_expired_fireballs_removes_timed_out() {
        let mut app = build_app();

        let entity = app
            .world_mut()
            .spawn(FireballProjectile {
                damage: 80.0,
                aoe_damage: 40.0,
                aoe_radius: 80.0,
                lifetime: 0.001, // nearly expired
            })
            .id();

        advance(&mut app);
        app.world_mut()
            .run_system_once(despawn_expired_fireballs)
            .expect("despawn_expired_fireballs should run");
        app.world_mut().flush();

        assert!(
            app.world().get_entity(entity).is_err(),
            "expired fireball should be despawned"
        );
    }

    /// Fireball with remaining lifetime survives.
    #[test]
    fn despawn_expired_fireballs_keeps_active() {
        let mut app = build_app();

        let entity = app
            .world_mut()
            .spawn(FireballProjectile {
                damage: 80.0,
                aoe_damage: 40.0,
                aoe_radius: 80.0,
                lifetime: 99.0,
            })
            .id();

        advance(&mut app);
        app.world_mut()
            .run_system_once(despawn_expired_fireballs)
            .expect("despawn_expired_fireballs should run");
        app.world_mut().flush();

        assert!(
            app.world().get_entity(entity).is_ok(),
            "active fireball should not be despawned"
        );
    }

    // -----------------------------------------------------------------------
    // despawn_explosion_effects tests
    // -----------------------------------------------------------------------

    /// Explosion effect with expired timer is despawned.
    #[test]
    fn despawn_explosion_effects_removes_timed_out() {
        let mut app = build_app();

        let entity = app
            .world_mut()
            .spawn(FireballExplosionEffect { remaining: 0.001 })
            .id();

        advance(&mut app);
        app.world_mut()
            .run_system_once(despawn_explosion_effects)
            .expect("despawn_explosion_effects should run");
        app.world_mut().flush();

        assert!(
            app.world().get_entity(entity).is_err(),
            "expired explosion effect should be despawned"
        );
    }

    /// Explosion effect with remaining time survives.
    #[test]
    fn despawn_explosion_effects_keeps_active() {
        let mut app = build_app();

        let entity = app
            .world_mut()
            .spawn(FireballExplosionEffect { remaining: 99.0 })
            .id();

        advance(&mut app);
        app.world_mut()
            .run_system_once(despawn_explosion_effects)
            .expect("despawn_explosion_effects should run");
        app.world_mut().flush();

        assert!(
            app.world().get_entity(entity).is_ok(),
            "active explosion effect should not be despawned"
        );
    }
}
