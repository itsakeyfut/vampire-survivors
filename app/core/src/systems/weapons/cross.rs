//! Cross weapon — boomerang projectile that reverses at max range.
//!
//! The Cross fires one or more projectiles aimed at the direction the player
//! is currently facing.  Each projectile travels outward until it reaches
//! [`CrossBoomerang::max_range`] pixels from its spawn point, then reverses
//! direction and returns.  Enemies can be hit on both the outbound and return
//! trips.
//!
//! ## Level progression
//!
//! | Level | Damage | Count | Speed (px/s) | Max Range (px) |
//! |-------|--------|-------|--------------|----------------|
//! | 1     | 50     | 1     | 300          | 150            |
//! | 2     | 60     | 1     | 320          | 160            |
//! | 3     | 70     | 1     | 340          | 175            |
//! | 4     | 80     | 1     | 360          | 190            |
//! | 5     | 90     | 2     | 380          | 205            |
//! | 6     | 110    | 2     | 400          | 220            |
//! | 7     | 130    | 2     | 430          | 235            |
//! | 8     | 160    | 2     | 460          | 250            |
//!
//! ## Boomerang behaviour
//!
//! Each projectile carries a [`CrossBoomerang`] component that tracks its
//! origin and maximum range.  [`update_cross`] runs every frame after
//! [`move_projectiles`] and flips the projectile's velocity once the
//! travelled distance reaches `max_range`.  The `hit_enemies` list is cleared
//! at that moment so enemies near the reversal point can be struck again on the
//! return pass.
//!
//! [`move_projectiles`]: crate::systems::projectiles::move_projectiles

use bevy::prelude::*;

use crate::{
    components::{Player, PlayerFacingDirection, PlayerStats, Projectile, ProjectileVelocity},
    config::weapon::cross::CrossParams,
    events::WeaponFiredEvent,
    systems::projectiles::spawn_projectile,
    types::WeaponType,
};

// ---------------------------------------------------------------------------
// Fallback constants (used while RON config is still loading)
// ---------------------------------------------------------------------------

/// Damage per hit at each weapon level while RON config is loading.
const DEFAULT_CROSS_DAMAGE_BY_LEVEL: [f32; 8] = [50.0, 60.0, 70.0, 80.0, 90.0, 110.0, 130.0, 160.0];
/// Projectile speed (px/s) at each weapon level while RON config is loading.
const DEFAULT_CROSS_SPEED_BY_LEVEL: [f32; 8] =
    [300.0, 320.0, 340.0, 360.0, 380.0, 400.0, 430.0, 460.0];
/// Maximum outbound range (px) at each weapon level while RON config is loading.
const DEFAULT_CROSS_MAX_RANGE_BY_LEVEL: [f32; 8] =
    [150.0, 160.0, 175.0, 190.0, 205.0, 220.0, 235.0, 250.0];
/// Number of projectiles per activation at each weapon level while RON config is loading.
const DEFAULT_CROSS_COUNT_BY_LEVEL: [u32; 8] = [1, 1, 1, 1, 2, 2, 2, 2];
/// Angular gap between projectiles in the fan (degrees).
const DEFAULT_CROSS_SPREAD_ANGLE_DEG: f32 = 30.0;
/// Circle collider radius for hit detection (pixels).
const DEFAULT_CROSS_COLLIDER_RADIUS: f32 = 8.0;

/// Piercing value for cross projectiles.
///
/// `u32::MAX` lets the projectile pass through every enemy it touches,
/// which is necessary for both the outbound and return trips to deal damage.
const CROSS_PIERCING: u32 = u32::MAX;

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/// Marker component attached to every projectile spawned by [`fire_cross`].
///
/// [`update_cross`] reads this each frame to decide when to reverse the
/// projectile's velocity.
#[derive(Component, Debug)]
pub struct CrossBoomerang {
    /// World-space position where this projectile was spawned.
    pub spawn_pos: Vec2,
    /// Distance (pixels) the projectile travels before reversing.
    pub max_range: f32,
    /// `true` once the projectile has reversed and is heading back toward the
    /// player.  Prevents a second reversal.
    pub returning: bool,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Fires Cross projectiles when a [`WeaponFiredEvent`] arrives.
///
/// Each projectile is aimed at [`PlayerFacingDirection`] (falling back to
/// `Vec2::X` when stationary).  For `count > 1`, projectiles are spread in a
/// symmetric fan.  Every projectile carries [`CrossBoomerang`] so that
/// [`update_cross`] can reverse it at `max_range`.
///
/// Lifetime is set to `2 × max_range / speed` so the projectile has just
/// enough time to travel out and return.
///
/// [`PlayerStats::extra_projectiles`] adds to the base count so the Duplicator
/// passive increases the number of crosses fired.
pub fn fire_cross(
    mut fired_events: MessageReader<WeaponFiredEvent>,
    mut commands: Commands,
    player_q: Query<(&Transform, &PlayerStats, &PlayerFacingDirection), With<Player>>,
    cross_cfg: CrossParams,
) {
    let cfg = cross_cfg.get();
    let spread_deg = cfg
        .map(|c| c.spread_angle_deg)
        .unwrap_or(DEFAULT_CROSS_SPREAD_ANGLE_DEG);
    let collider_r = cfg
        .map(|c| c.collider_radius)
        .unwrap_or(DEFAULT_CROSS_COLLIDER_RADIUS);

    for event in fired_events.read() {
        if event.weapon_type != WeaponType::Cross {
            continue;
        }

        let Ok((player_tf, stats, facing)) = player_q.get(event.player) else {
            continue;
        };

        let player_pos = player_tf.translation.truncate();
        let level = event.level.clamp(1, 8) as usize;

        let damage = cfg
            .and_then(|c| c.damage_by_level.get(level - 1).copied())
            .unwrap_or(DEFAULT_CROSS_DAMAGE_BY_LEVEL[level - 1])
            * stats.damage_multiplier;

        let speed = cfg
            .and_then(|c| c.speed_by_level.get(level - 1).copied())
            .unwrap_or(DEFAULT_CROSS_SPEED_BY_LEVEL[level - 1])
            * stats.projectile_speed_mult;

        let max_range = cfg
            .and_then(|c| c.max_range_by_level.get(level - 1).copied())
            .unwrap_or(DEFAULT_CROSS_MAX_RANGE_BY_LEVEL[level - 1]);

        let count = cfg
            .and_then(|c| c.count_by_level.get(level - 1).copied())
            .unwrap_or(DEFAULT_CROSS_COUNT_BY_LEVEL[level - 1])
            + stats.extra_projectiles;

        // Lifetime = round-trip time with a small buffer so the projectile
        // isn't despawned before it completes the return.
        let lifetime = 2.0 * max_range / speed + 0.5;

        let spread_rad = spread_deg.to_radians();
        let half_span = (count as f32 - 1.0) * 0.5 * spread_rad;
        let base_dir = facing.0.normalize_or(Vec2::X);

        for i in 0..count {
            let angle_offset = i as f32 * spread_rad - half_span;
            let dir = rotate_vec2(base_dir, angle_offset);
            let velocity = dir * speed;

            let entity = spawn_projectile(
                &mut commands,
                player_pos,
                velocity,
                damage,
                lifetime,
                CROSS_PIERCING,
                collider_r,
                event.weapon_type,
            );
            commands.entity(entity).insert(CrossBoomerang {
                spawn_pos: player_pos,
                max_range,
                returning: false,
            });
        }
    }
}

/// Reverses each Cross projectile once it has reached its maximum range.
///
/// For each projectile carrying [`CrossBoomerang`]:
/// - If the distance from `spawn_pos` is ≥ `max_range` and it is not yet
///   returning, the velocity is negated, `hit_enemies` is cleared (so enemies
///   near the reversal point can be struck again on the return pass), and
///   `returning` is set to `true`.
///
/// This system must run **after** [`move_projectiles`] so it acts on the
/// position that was just written this frame.
///
/// [`move_projectiles`]: crate::systems::projectiles::move_projectiles
pub fn update_cross(
    mut cross_q: Query<(
        &Transform,
        &mut ProjectileVelocity,
        &mut Projectile,
        &mut CrossBoomerang,
    )>,
) {
    for (transform, mut velocity, mut projectile, mut boomerang) in cross_q.iter_mut() {
        if boomerang.returning {
            continue;
        }

        let pos = transform.translation.truncate();
        if pos.distance(boomerang.spawn_pos) >= boomerang.max_range {
            velocity.0 = -velocity.0;
            projectile.hit_enemies.clear();
            boomerang.returning = true;
        }
    }
}

/// Rotates a 2D vector by `angle` radians (counter-clockwise positive).
#[inline]
fn rotate_vec2(v: Vec2, angle: f32) -> Vec2 {
    let (sin, cos) = angle.sin_cos();
    Vec2::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos)
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
        components::{Projectile, ProjectileVelocity, WeaponInventory},
        events::WeaponFiredEvent,
        types::{WeaponState, WeaponType},
    };

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<WeaponFiredEvent>();
        app
    }

    fn advance(app: &mut App) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(1.0 / 60.0));
    }

    fn spawn_player(app: &mut App) -> Entity {
        let mut weapon = WeaponState::new(WeaponType::Cross);
        weapon.cooldown_timer = 0.0;
        app.world_mut()
            .spawn((
                Player,
                PlayerStats::default(),
                PlayerFacingDirection::default(), // faces right (Vec2::X)
                WeaponInventory {
                    weapons: vec![weapon],
                },
                Transform::from_xyz(0.0, 0.0, 10.0),
            ))
            .id()
    }

    fn tick_and_fire(app: &mut App) {
        use crate::systems::weapons::cooldown::tick_weapon_cooldowns;
        advance(app);
        app.world_mut()
            .run_system_once(tick_weapon_cooldowns)
            .expect("tick_weapon_cooldowns should run");
        app.world_mut()
            .run_system_once(fire_cross)
            .expect("fire_cross should run");
        app.world_mut().flush();
    }

    fn projectile_count(app: &mut App) -> usize {
        app.world_mut()
            .query::<&Projectile>()
            .iter(app.world())
            .count()
    }

    fn spawn_cross_boomerang(
        app: &mut App,
        pos: Vec2,
        velocity: Vec2,
        spawn_pos: Vec2,
        max_range: f32,
        returning: bool,
    ) -> Entity {
        app.world_mut()
            .spawn((
                Projectile {
                    damage: 50.0,
                    piercing: u32::MAX,
                    hit_enemies: Vec::new(),
                    lifetime: 5.0,
                    weapon_type: WeaponType::Cross,
                },
                ProjectileVelocity(velocity),
                CrossBoomerang {
                    spawn_pos,
                    max_range,
                    returning,
                },
                Transform::from_xyz(pos.x, pos.y, 5.0),
            ))
            .id()
    }

    // -----------------------------------------------------------------------
    // fire_cross tests
    // -----------------------------------------------------------------------

    /// Level 1 fires exactly one projectile.
    #[test]
    fn fire_cross_level_1_fires_one_projectile() {
        let mut app = build_app();
        spawn_player(&mut app);
        tick_and_fire(&mut app);

        assert_eq!(projectile_count(&mut app), 1, "level 1 should fire 1 cross");
    }

    /// Level 5 fires two projectiles.
    #[test]
    fn fire_cross_level_5_fires_two_projectiles() {
        let mut app = build_app();
        let player = spawn_player(&mut app);

        app.world_mut().write_message(WeaponFiredEvent {
            player,
            weapon_type: WeaponType::Cross,
            level: 5,
        });
        app.world_mut()
            .run_system_once(fire_cross)
            .expect("fire_cross should run");
        app.world_mut().flush();

        assert_eq!(
            projectile_count(&mut app),
            2,
            "level 5 should fire 2 crosses"
        );
    }

    /// Non-Cross events are ignored.
    #[test]
    fn fire_cross_ignores_other_weapons() {
        let mut app = build_app();
        let player = spawn_player(&mut app);

        app.world_mut().write_message(WeaponFiredEvent {
            player,
            weapon_type: WeaponType::Knife,
            level: 1,
        });
        app.world_mut()
            .run_system_once(fire_cross)
            .expect("fire_cross should run");
        app.world_mut().flush();

        assert_eq!(
            projectile_count(&mut app),
            0,
            "fire_cross should ignore non-Cross weapon events"
        );
    }

    /// The projectile travels in the player's facing direction.
    #[test]
    fn fire_cross_projectile_travels_in_facing_direction() {
        let mut app = build_app();
        spawn_player(&mut app); // facing right (Vec2::X) by default
        tick_and_fire(&mut app);

        let mut q = app
            .world_mut()
            .query::<&ProjectileVelocity>()
            .iter(app.world())
            .map(|v| v.0)
            .collect::<Vec<_>>();

        assert_eq!(q.len(), 1);
        let vel = q.remove(0);
        assert!(
            vel.x > 0.0 && vel.y.abs() < 1.0,
            "cross should travel rightward, got vel = {vel:?}"
        );
    }

    /// Cross projectiles have maximum piercing.
    #[test]
    fn fire_cross_projectile_has_infinite_piercing() {
        let mut app = build_app();
        spawn_player(&mut app);
        tick_and_fire(&mut app);

        let mut q = app.world_mut().query::<&Projectile>();
        let proj = q
            .iter(app.world())
            .next()
            .expect("one projectile should exist");
        assert_eq!(
            proj.piercing,
            u32::MAX,
            "cross should pierce all enemies (piercing = u32::MAX)"
        );
    }

    /// Cross projectiles carry the CrossBoomerang component.
    #[test]
    fn fire_cross_projectile_has_boomerang_component() {
        let mut app = build_app();
        spawn_player(&mut app);
        tick_and_fire(&mut app);

        let count = app
            .world_mut()
            .query::<&CrossBoomerang>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "fired cross should have CrossBoomerang component");
    }

    /// CrossBoomerang starts with returning = false and spawn_pos = player pos.
    #[test]
    fn fire_cross_boomerang_starts_outbound() {
        let mut app = build_app();
        spawn_player(&mut app); // at (0, 0)
        tick_and_fire(&mut app);

        let mut q = app.world_mut().query::<&CrossBoomerang>();
        let boomerang = q
            .iter(app.world())
            .next()
            .expect("CrossBoomerang should exist");
        assert!(!boomerang.returning, "should start in outbound phase");
        assert_eq!(
            boomerang.spawn_pos,
            Vec2::ZERO,
            "spawn_pos should match player position"
        );
    }

    /// extra_projectiles increases the count.
    #[test]
    fn fire_cross_extra_projectiles_increases_count() {
        let mut app = build_app();
        let player = spawn_player(&mut app);

        // Add 1 extra projectile via PlayerStats.
        app.world_mut()
            .get_mut::<PlayerStats>(player)
            .unwrap()
            .extra_projectiles = 1;

        app.world_mut().write_message(WeaponFiredEvent {
            player,
            weapon_type: WeaponType::Cross,
            level: 1,
        });
        app.world_mut()
            .run_system_once(fire_cross)
            .expect("fire_cross should run");
        app.world_mut().flush();

        // Base count at Lv1 = 1, +1 extra = 2
        assert_eq!(
            projectile_count(&mut app),
            2,
            "extra_projectiles should add to the cross count"
        );
    }

    // -----------------------------------------------------------------------
    // update_cross tests
    // -----------------------------------------------------------------------

    /// A projectile exactly at max_range is reversed.
    #[test]
    fn update_cross_reverses_at_max_range() {
        let mut app = build_app();

        let spawn_pos = Vec2::ZERO;
        let max_range = 100.0;
        let entity = spawn_cross_boomerang(
            &mut app,
            Vec2::new(100.0, 0.0), // at max_range
            Vec2::new(300.0, 0.0),
            spawn_pos,
            max_range,
            false,
        );

        app.world_mut()
            .run_system_once(update_cross)
            .expect("update_cross should run");

        let vel = app.world().get::<ProjectileVelocity>(entity).unwrap();
        assert_eq!(
            vel.0,
            Vec2::new(-300.0, 0.0),
            "velocity should be reversed at max_range"
        );

        let boomerang = app.world().get::<CrossBoomerang>(entity).unwrap();
        assert!(boomerang.returning, "should be marked as returning");
    }

    /// A projectile beyond max_range is also reversed.
    #[test]
    fn update_cross_reverses_beyond_max_range() {
        let mut app = build_app();

        let entity = spawn_cross_boomerang(
            &mut app,
            Vec2::new(120.0, 0.0), // past max_range
            Vec2::new(300.0, 0.0),
            Vec2::ZERO,
            100.0,
            false,
        );

        app.world_mut()
            .run_system_once(update_cross)
            .expect("update_cross should run");

        let boomerang = app.world().get::<CrossBoomerang>(entity).unwrap();
        assert!(boomerang.returning, "should reverse when past max_range");
    }

    /// A projectile short of max_range is not reversed.
    #[test]
    fn update_cross_does_not_reverse_before_max_range() {
        let mut app = build_app();

        let entity = spawn_cross_boomerang(
            &mut app,
            Vec2::new(50.0, 0.0), // halfway to max_range
            Vec2::new(300.0, 0.0),
            Vec2::ZERO,
            100.0,
            false,
        );

        app.world_mut()
            .run_system_once(update_cross)
            .expect("update_cross should run");

        let vel = app.world().get::<ProjectileVelocity>(entity).unwrap();
        assert_eq!(
            vel.0,
            Vec2::new(300.0, 0.0),
            "velocity should not change before max_range"
        );

        let boomerang = app.world().get::<CrossBoomerang>(entity).unwrap();
        assert!(!boomerang.returning, "should still be outbound");
    }

    /// A projectile already returning is not reversed again.
    #[test]
    fn update_cross_reverses_only_once() {
        let mut app = build_app();

        // Place it at max_range but already returning.
        let entity = spawn_cross_boomerang(
            &mut app,
            Vec2::new(100.0, 0.0),
            Vec2::new(-300.0, 0.0), // already reversed
            Vec2::ZERO,
            100.0,
            true, // already returning
        );

        app.world_mut()
            .run_system_once(update_cross)
            .expect("update_cross should run");

        // Velocity should remain unchanged.
        let vel = app.world().get::<ProjectileVelocity>(entity).unwrap();
        assert_eq!(
            vel.0,
            Vec2::new(-300.0, 0.0),
            "should not reverse again when already returning"
        );
    }

    /// hit_enemies is cleared on reversal to allow return-trip hits.
    #[test]
    fn update_cross_clears_hit_enemies_on_reversal() {
        let mut app = build_app();

        // Spawn a real entity to record as a "hit" enemy.
        let fake_enemy = app.world_mut().spawn_empty().id();
        let entity = spawn_cross_boomerang(
            &mut app,
            Vec2::new(100.0, 0.0),
            Vec2::new(300.0, 0.0),
            Vec2::ZERO,
            100.0,
            false,
        );

        // Pre-populate hit_enemies as if it struck something outbound.
        app.world_mut()
            .get_mut::<Projectile>(entity)
            .unwrap()
            .hit_enemies
            .push(fake_enemy);

        app.world_mut()
            .run_system_once(update_cross)
            .expect("update_cross should run");

        let proj = app.world().get::<Projectile>(entity).unwrap();
        assert!(
            proj.hit_enemies.is_empty(),
            "hit_enemies should be cleared on reversal"
        );
    }

    /// `rotate_vec2` with zero angle returns the original vector.
    #[test]
    fn rotate_vec2_zero_angle_is_identity() {
        let v = Vec2::new(1.0, 0.0);
        let rotated = rotate_vec2(v, 0.0);
        assert!((rotated.x - v.x).abs() < 1e-6);
        assert!((rotated.y - v.y).abs() < 1e-6);
    }

    /// `rotate_vec2` by 90° turns (1, 0) → (0, 1).
    #[test]
    fn rotate_vec2_90_deg() {
        let v = Vec2::new(1.0, 0.0);
        let rotated = rotate_vec2(v, std::f32::consts::FRAC_PI_2);
        assert!((rotated.x).abs() < 1e-6, "x should be ~0");
        assert!((rotated.y - 1.0).abs() < 1e-6, "y should be ~1");
    }
}
