//! Magic Wand weapon — homing projectile toward the nearest enemy.
//!
//! The Magic Wand fires a single projectile aimed at the closest enemy each
//! activation.  The projectile travels in a straight line at constant speed
//! and is removed when its lifetime expires.
//!
//! ## Targeting
//!
//! The system iterates over all current enemy transforms to find the one with
//! the smallest squared distance from the player.  A full scan is appropriate
//! here because targeting is global (nearest enemy on the entire map, not
//! bounded by a fixed range) and occurs only on weapon fire (infrequently).
//!
//! ## Damage formula
//!
//! ```text
//! damage = (base + per_level × (level − 1)) × player.damage_multiplier
//! ```

use bevy::prelude::*;

use crate::{
    components::{Enemy, Player, PlayerStats},
    events::WeaponFiredEvent,
    systems::projectile::spawn_projectile,
    types::WeaponType,
};

// ---------------------------------------------------------------------------
// Fallback constants (used when RON config is not yet loaded)
// ---------------------------------------------------------------------------

/// Projectile travel speed in pixels/second.
const DEFAULT_MAGIC_WAND_SPEED: f32 = 600.0;
/// Base damage at weapon level 1.
const DEFAULT_MAGIC_WAND_BASE_DAMAGE: f32 = 20.0;
/// Additional damage per weapon level above 1.
const DEFAULT_MAGIC_WAND_DAMAGE_PER_LEVEL: f32 = 10.0;
/// Projectile lifetime in seconds.
const DEFAULT_MAGIC_WAND_LIFETIME: f32 = 5.0;
/// Circle collider radius for hit detection (pixels).
const DEFAULT_MAGIC_WAND_COLLIDER_RADIUS: f32 = 8.0;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn magic_wand_damage_for_level(level: u8) -> f32 {
    DEFAULT_MAGIC_WAND_BASE_DAMAGE
        + DEFAULT_MAGIC_WAND_DAMAGE_PER_LEVEL * (level.clamp(1, 8) as f32 - 1.0)
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Fires a [`Projectile`](crate::components::Projectile) toward the nearest
/// enemy when a [`WeaponFiredEvent`] for [`WeaponType::MagicWand`] (or its
/// evolution [`WeaponType::HolyWand`]) arrives.
///
/// If no enemies are present the event is silently consumed.
pub fn fire_magic_wand(
    mut fired_events: MessageReader<WeaponFiredEvent>,
    mut commands: Commands,
    player_q: Query<(&Transform, &PlayerStats), With<Player>>,
    enemy_q: Query<&Transform, With<Enemy>>,
) {
    for event in fired_events.read() {
        if event.weapon_type != WeaponType::MagicWand && event.weapon_type != WeaponType::HolyWand {
            continue;
        }

        let Ok((player_tf, stats)) = player_q.get(event.player) else {
            continue;
        };

        let player_pos = player_tf.translation.truncate();

        // Full scan: targeting is global (nearest on entire map) so a range-
        // bounded SpatialGrid query would not help here.
        let nearest = enemy_q
            .iter()
            .map(|tf| tf.translation.truncate())
            .min_by(|a, b| {
                let da = a.distance_squared(player_pos);
                let db = b.distance_squared(player_pos);
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            });

        let Some(target_pos) = nearest else {
            continue; // no enemies on screen
        };

        let dir = (target_pos - player_pos).normalize_or_zero();
        if dir == Vec2::ZERO {
            continue; // enemy exactly on player position — cannot aim
        }

        let damage = magic_wand_damage_for_level(event.level) * stats.damage_multiplier;
        spawn_projectile(
            &mut commands,
            player_pos,
            dir * DEFAULT_MAGIC_WAND_SPEED,
            damage,
            DEFAULT_MAGIC_WAND_LIFETIME,
            0, // piercing = 0 (single hit)
            DEFAULT_MAGIC_WAND_COLLIDER_RADIUS,
            event.weapon_type,
        );
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
        components::{Projectile, ProjectileVelocity, WeaponInventory},
        events::WeaponFiredEvent,
        types::{EnemyType, WeaponState, WeaponType},
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

    fn advance(app: &mut App, secs: f32) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(secs));
    }

    /// Spawns a player with a MagicWand at timer=0 (fires on first tick).
    fn spawn_player(app: &mut App) -> Entity {
        let mut weapon = WeaponState::new(WeaponType::MagicWand);
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

    fn spawn_enemy(app: &mut App, pos: Vec2) -> Entity {
        app.world_mut()
            .spawn((
                Enemy::from_type(EnemyType::Bat, 1.0),
                Transform::from_xyz(pos.x, pos.y, 1.0),
            ))
            .id()
    }

    /// Runs tick_weapon_cooldowns then fire_magic_wand for one frame.
    fn tick_and_fire(app: &mut App) {
        use crate::systems::weapon_cooldown::tick_weapon_cooldowns;
        advance(app, 1.0 / 60.0);
        app.world_mut()
            .run_system_once(tick_weapon_cooldowns)
            .expect("tick_weapon_cooldowns should run");
        app.world_mut()
            .run_system_once(fire_magic_wand)
            .expect("fire_magic_wand should run");
        // Flush Commands so spawned entities are visible immediately.
        app.world_mut().flush();
    }

    fn projectiles(app: &mut App) -> Vec<(Vec2, Vec2)> {
        // Returns (position, velocity) pairs.
        app.world_mut()
            .query::<(&Transform, &ProjectileVelocity)>()
            .iter(app.world())
            .map(|(tf, vel)| (tf.translation.truncate(), vel.0))
            .collect()
    }

    // -----------------------------------------------------------------------
    // Tests
    // -----------------------------------------------------------------------

    /// With one enemy in range, exactly one projectile is spawned.
    #[test]
    fn fires_one_projectile_per_event() {
        let mut app = build_app();
        spawn_player(&mut app);
        spawn_enemy(&mut app, Vec2::new(200.0, 0.0));

        tick_and_fire(&mut app);

        let projs = projectiles(&mut app);
        assert_eq!(projs.len(), 1, "expected exactly one projectile");
    }

    /// The projectile velocity points from the player toward the enemy.
    #[test]
    fn projectile_aims_at_enemy() {
        let mut app = build_app();
        spawn_player(&mut app); // player at (0, 0)
        spawn_enemy(&mut app, Vec2::new(100.0, 0.0)); // enemy directly right

        tick_and_fire(&mut app);

        let projs = projectiles(&mut app);
        assert_eq!(projs.len(), 1);
        let vel = projs[0].1;
        // Velocity should be purely rightward.
        assert!(vel.x > 0.0, "velocity x should be positive (rightward)");
        assert!(
            vel.y.abs() < 1e-4,
            "velocity y should be ~0 for enemy directly right, got {}",
            vel.y
        );
    }

    /// The projectile aims at the NEAREST enemy, not any random one.
    #[test]
    fn targets_nearest_enemy() {
        let mut app = build_app();
        spawn_player(&mut app); // player at (0, 0)
        spawn_enemy(&mut app, Vec2::new(500.0, 0.0)); // far right
        spawn_enemy(&mut app, Vec2::new(-80.0, 0.0)); // near left — nearest

        tick_and_fire(&mut app);

        let projs = projectiles(&mut app);
        assert_eq!(projs.len(), 1);
        let vel = projs[0].1;
        // Must aim left (toward the closer enemy at -80, 0).
        assert!(
            vel.x < 0.0,
            "should aim left toward the nearer enemy, vel = {vel:?}"
        );
    }

    /// No enemies → no projectile spawned.
    #[test]
    fn no_projectile_without_enemies() {
        let mut app = build_app();
        spawn_player(&mut app);

        tick_and_fire(&mut app);

        let projs = projectiles(&mut app);
        assert_eq!(
            projs.len(),
            0,
            "no projectile should be spawned without enemies"
        );
    }

    /// The projectile carries the correct weapon_type tag.
    #[test]
    fn projectile_has_correct_weapon_type() {
        let mut app = build_app();
        spawn_player(&mut app);
        spawn_enemy(&mut app, Vec2::new(100.0, 0.0));

        tick_and_fire(&mut app);

        app.world_mut().flush();

        let mut q = app.world_mut().query::<&Projectile>();
        let proj = q
            .iter(app.world())
            .next()
            .expect("one projectile should exist");
        assert_eq!(proj.weapon_type, WeaponType::MagicWand);
        assert_eq!(proj.piercing, 0, "MagicWand should not pierce");
    }

    /// Projectile speed equals DEFAULT_MAGIC_WAND_SPEED.
    #[test]
    fn projectile_travels_at_correct_speed() {
        let mut app = build_app();
        spawn_player(&mut app);
        spawn_enemy(&mut app, Vec2::new(0.0, 200.0)); // directly above

        tick_and_fire(&mut app);

        let projs = projectiles(&mut app);
        assert_eq!(projs.len(), 1);
        let speed = projs[0].1.length();
        assert!(
            (speed - DEFAULT_MAGIC_WAND_SPEED).abs() < 1.0,
            "expected speed ≈ {DEFAULT_MAGIC_WAND_SPEED}, got {speed}"
        );
    }

    /// Damage scales with weapon level.
    #[test]
    fn damage_scales_with_level() {
        assert!(
            magic_wand_damage_for_level(2) > magic_wand_damage_for_level(1),
            "level 2 should deal more damage than level 1"
        );
        assert!(
            magic_wand_damage_for_level(8) > magic_wand_damage_for_level(1),
            "level 8 should deal the most damage"
        );
    }

    /// HolyWand (evolved form) also spawns a projectile.
    #[test]
    fn holy_wand_fires_projectile() {
        let mut app = build_app();
        spawn_enemy(&mut app, Vec2::new(100.0, 0.0));

        // Manually write a HolyWand fired event.
        let player = app
            .world_mut()
            .spawn((
                Player,
                PlayerStats::default(),
                WeaponInventory { weapons: vec![] },
                Transform::from_xyz(0.0, 0.0, 10.0),
            ))
            .id();
        app.world_mut().write_message(WeaponFiredEvent {
            player,
            weapon_type: WeaponType::HolyWand,
            level: 1,
        });
        app.world_mut()
            .run_system_once(fire_magic_wand)
            .expect("fire_magic_wand should run");
        app.world_mut().flush();

        let projs = projectiles(&mut app);
        assert_eq!(projs.len(), 1, "HolyWand should fire a projectile");
    }
}
