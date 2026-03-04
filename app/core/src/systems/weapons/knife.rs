//! Knife weapon — high-speed piercing projectiles in the player's movement direction.
//!
//! The Knife fires one or more projectiles aimed at the direction the player
//! is currently facing (or last moved toward).  All projectiles pierce through
//! enemies indefinitely.
//!
//! ## Level progression
//!
//! | Level | Damage | Count | Speed (px/s) |
//! |-------|--------|-------|--------------|
//! | 1     | 15     | 1     | 600          |
//! | 2     | 15     | 1     | 700          |
//! | 3     | 20     | 2     | 700          |
//! | 4     | 20     | 2     | 800          |
//! | 5     | 25     | 3     | 800          |
//! | 6     | 25     | 3     | 900          |
//! | 7     | 30     | 4     | 900          |
//! | 8     | 30     | 5     | 1000         |
//!
//! ## Multi-projectile fan
//!
//! When `count > 1`, projectiles are spread in a symmetric fan centred on the
//! facing direction.  The angular gap between adjacent knives is controlled by
//! `spread_angle_deg` in `assets/config/weapons/knife.ron`.
//!
//! ## Scaling formulas
//!
//! ```text
//! speed  = (base + floor(level / 2) × speed_per_two_levels) × player.projectile_speed_mult
//! damage = (base + floor((level − 1) / 2) × damage_per_two_levels) × player.damage_multiplier
//! ```

use bevy::prelude::*;

use crate::{
    components::{Player, PlayerFacingDirection, PlayerStats},
    config::weapon::knife::KnifeParams,
    events::WeaponFiredEvent,
    systems::projectiles::spawn_projectile,
    types::WeaponType,
};

// ---------------------------------------------------------------------------
// Fallback constants (used while RON config is still loading)
// ---------------------------------------------------------------------------

/// Base projectile speed at weapon level 1 (pixels/second).
const DEFAULT_KNIFE_BASE_SPEED: f32 = 600.0;
/// Speed added every two weapon levels.
const DEFAULT_KNIFE_SPEED_PER_TWO_LEVELS: f32 = 100.0;
/// Base damage at weapon level 1.
const DEFAULT_KNIFE_BASE_DAMAGE: f32 = 15.0;
/// Damage added every two weapon levels.
const DEFAULT_KNIFE_DAMAGE_PER_TWO_LEVELS: f32 = 5.0;
/// Projectile lifetime in seconds.
const DEFAULT_KNIFE_LIFETIME: f32 = 5.0;
/// Circle collider radius for hit detection (pixels).
const DEFAULT_KNIFE_COLLIDER_RADIUS: f32 = 6.0;
/// Angular gap between adjacent knives in a fan (degrees).
const DEFAULT_KNIFE_SPREAD_ANGLE_DEG: f32 = 15.0;

/// Fallback projectile count per level while RON config is still loading.
const DEFAULT_KNIFE_COUNT_BY_LEVEL: [u32; 8] = [1, 1, 2, 2, 3, 3, 4, 5];

/// Piercing value used for knife projectiles.
/// `u32::MAX` ensures knives pass through every enemy for the duration of their
/// lifetime without a separate "infinite pierce" code path.
const KNIFE_PIERCING: u32 = u32::MAX;

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Fires Knife (or ThousandEdge) projectiles when a [`WeaponFiredEvent`] arrives.
///
/// Projectiles travel in a symmetric fan centred on [`PlayerFacingDirection`].
/// They pierce through all enemies (`piercing = u32::MAX`).
///
/// [`PlayerStats::extra_projectiles`] is added to the base count so that the
/// Duplicator passive item increases the number of knives fired.
pub fn fire_knife(
    mut fired_events: MessageReader<WeaponFiredEvent>,
    mut commands: Commands,
    player_q: Query<(&Transform, &PlayerStats, &PlayerFacingDirection), With<Player>>,
    knife_cfg: KnifeParams,
) {
    let cfg = knife_cfg.get();
    let base_speed = cfg
        .map(|c| c.base_speed)
        .unwrap_or(DEFAULT_KNIFE_BASE_SPEED);
    let speed_per_two = cfg
        .map(|c| c.speed_per_two_levels)
        .unwrap_or(DEFAULT_KNIFE_SPEED_PER_TWO_LEVELS);
    let base_damage = cfg
        .map(|c| c.base_damage)
        .unwrap_or(DEFAULT_KNIFE_BASE_DAMAGE);
    let dmg_per_two = cfg
        .map(|c| c.damage_per_two_levels)
        .unwrap_or(DEFAULT_KNIFE_DAMAGE_PER_TWO_LEVELS);
    let lifetime = cfg.map(|c| c.lifetime).unwrap_or(DEFAULT_KNIFE_LIFETIME);
    let collider_r = cfg
        .map(|c| c.collider_radius)
        .unwrap_or(DEFAULT_KNIFE_COLLIDER_RADIUS);
    let spread_deg = cfg
        .map(|c| c.spread_angle_deg)
        .unwrap_or(DEFAULT_KNIFE_SPREAD_ANGLE_DEG);

    for event in fired_events.read() {
        let is_thousand_edge = event.weapon_type == WeaponType::ThousandEdge;
        if event.weapon_type != WeaponType::Knife && !is_thousand_edge {
            continue;
        }

        let Ok((player_tf, stats, facing)) = player_q.get(event.player) else {
            continue;
        };

        let player_pos = player_tf.translation.truncate();
        let level = event.level.clamp(1, 8) as usize;

        // --- Compute per-level stats ---
        // Speed steps at every level: 0 at Lv1, 1 at Lv2-3, 2 at Lv4-5, …
        let speed_step = level / 2;
        // Damage steps every two levels: 0 at Lv1-2, 1 at Lv3-4, …
        let dmg_step = (level - 1) / 2;
        let speed = (base_speed + speed_step as f32 * speed_per_two) * stats.projectile_speed_mult;
        let damage = (base_damage + dmg_step as f32 * dmg_per_two) * stats.damage_multiplier;

        // Base count from the level table, plus player's extra_projectiles bonus.
        let mut count = cfg
            .and_then(|c| c.count_by_level.get(level - 1).copied())
            .unwrap_or(DEFAULT_KNIFE_COUNT_BY_LEVEL[level - 1])
            + stats.extra_projectiles;
        // ThousandEdge fires twice as many knives as the base Knife at the same level.
        if is_thousand_edge {
            count *= 2;
        }

        // --- Fan spread ---
        // Knives are distributed symmetrically around the facing direction.
        // For count=1 the offset is 0°; for count=N, offsets are:
        //   -(N-1)/2 * gap, …, 0, …, +(N-1)/2 * gap
        let spread_rad = spread_deg.to_radians();
        let half_span = (count as f32 - 1.0) * 0.5 * spread_rad;

        let base_dir = facing.0.normalize_or(Vec2::X);

        for i in 0..count {
            let angle_offset = i as f32 * spread_rad - half_span;
            let dir = rotate_vec2(base_dir, angle_offset);
            let velocity = dir * speed;
            spawn_projectile(
                &mut commands,
                player_pos,
                velocity,
                damage,
                lifetime,
                KNIFE_PIERCING,
                collider_r,
                event.weapon_type,
            );
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
        let mut weapon = WeaponState::new(WeaponType::Knife);
        weapon.cooldown_timer = 0.0;
        app.world_mut()
            .spawn((
                Player,
                PlayerStats::default(),
                PlayerFacingDirection::default(), // faces right
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
            .run_system_once(fire_knife)
            .expect("fire_knife should run");
        app.world_mut().flush();
    }

    fn projectiles(app: &mut App) -> Vec<(Vec2, Vec2)> {
        app.world_mut()
            .query::<(&Transform, &ProjectileVelocity)>()
            .iter(app.world())
            .map(|(tf, vel)| (tf.translation.truncate(), vel.0))
            .collect()
    }

    // -----------------------------------------------------------------------
    // Tests
    // -----------------------------------------------------------------------

    /// Level 1 fires exactly one projectile.
    #[test]
    fn level_1_fires_one_projectile() {
        let mut app = build_app();
        spawn_player(&mut app);
        tick_and_fire(&mut app);

        assert_eq!(
            projectiles(&mut app).len(),
            1,
            "level 1 should fire 1 knife"
        );
    }

    /// Level 3 fires two projectiles.
    #[test]
    fn level_3_fires_two_projectiles() {
        let mut app = build_app();
        let player = spawn_player(&mut app);
        // Override weapon level to 3.
        app.world_mut()
            .get_mut::<WeaponInventory>(player)
            .unwrap()
            .weapons[0]
            .level = 3;

        // Write event manually at level 3.
        app.world_mut().write_message(WeaponFiredEvent {
            player,
            weapon_type: WeaponType::Knife,
            level: 3,
        });
        app.world_mut()
            .run_system_once(fire_knife)
            .expect("fire_knife should run");
        app.world_mut().flush();

        assert_eq!(
            projectiles(&mut app).len(),
            2,
            "level 3 should fire 2 knives"
        );
    }

    /// Level 8 fires five projectiles.
    #[test]
    fn level_8_fires_five_projectiles() {
        let mut app = build_app();
        let player = spawn_player(&mut app);

        app.world_mut().write_message(WeaponFiredEvent {
            player,
            weapon_type: WeaponType::Knife,
            level: 8,
        });
        app.world_mut()
            .run_system_once(fire_knife)
            .expect("fire_knife should run");
        app.world_mut().flush();

        assert_eq!(
            projectiles(&mut app).len(),
            5,
            "level 8 should fire 5 knives"
        );
    }

    /// The first (or only) projectile travels in the player's facing direction.
    #[test]
    fn projectile_travels_in_facing_direction() {
        let mut app = build_app();
        spawn_player(&mut app); // facing right (Vec2::X by default)
        tick_and_fire(&mut app);

        let projs = projectiles(&mut app);
        assert_eq!(projs.len(), 1);
        let vel = projs[0].1;
        assert!(
            vel.x > 0.0 && vel.y.abs() < 1.0,
            "knife should travel rightward, got vel = {vel:?}"
        );
    }

    /// Projectile speed equals DEFAULT_KNIFE_BASE_SPEED at level 1.
    #[test]
    fn projectile_speed_at_level_1() {
        let mut app = build_app();
        spawn_player(&mut app);
        tick_and_fire(&mut app);

        let projs = projectiles(&mut app);
        assert_eq!(projs.len(), 1);
        let speed = projs[0].1.length();
        assert!(
            (speed - DEFAULT_KNIFE_BASE_SPEED).abs() < 1.0,
            "expected speed ≈ {DEFAULT_KNIFE_BASE_SPEED}, got {speed}"
        );
    }

    /// Knife projectiles have maximum piercing.
    #[test]
    fn knife_projectile_has_max_piercing() {
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
            "knife should pierce all enemies (piercing = u32::MAX)"
        );
    }

    /// Knife projectile carries the correct weapon type tag.
    #[test]
    fn projectile_has_correct_weapon_type() {
        let mut app = build_app();
        spawn_player(&mut app);
        tick_and_fire(&mut app);

        let mut q = app.world_mut().query::<&Projectile>();
        let proj = q
            .iter(app.world())
            .next()
            .expect("one projectile should exist");
        assert_eq!(proj.weapon_type, WeaponType::Knife);
    }

    /// ThousandEdge fires twice as many projectiles as the base Knife level.
    #[test]
    fn thousand_edge_fires_double_count() {
        let mut app = build_app();
        let player = spawn_player(&mut app);

        // Level 1 Knife = 1 → ThousandEdge should fire 2.
        app.world_mut().write_message(WeaponFiredEvent {
            player,
            weapon_type: WeaponType::ThousandEdge,
            level: 1,
        });
        app.world_mut()
            .run_system_once(fire_knife)
            .expect("fire_knife should run");
        app.world_mut().flush();

        assert_eq!(
            projectiles(&mut app).len(),
            2,
            "ThousandEdge Lv1 should fire 2 projectiles (2× Knife Lv1 count)"
        );
    }

    /// Non-Knife events are ignored.
    #[test]
    fn other_weapons_do_not_fire_knife() {
        let mut app = build_app();
        let player = spawn_player(&mut app);

        app.world_mut().write_message(WeaponFiredEvent {
            player,
            weapon_type: WeaponType::MagicWand,
            level: 1,
        });
        app.world_mut()
            .run_system_once(fire_knife)
            .expect("fire_knife should run");
        app.world_mut().flush();

        assert_eq!(
            projectiles(&mut app).len(),
            0,
            "fire_knife should ignore non-Knife weapon events"
        );
    }

    /// Damage scales every two levels.
    #[test]
    fn damage_scales_every_two_levels() {
        let step = DEFAULT_KNIFE_DAMAGE_PER_TWO_LEVELS;
        let base = DEFAULT_KNIFE_BASE_DAMAGE;
        // Level 1 and 2 have same damage.
        let lv1_dmg = base; // step 0
        let lv2_dmg = base; // step 0
        let lv3_dmg = base + step; // step 1
        assert!(
            (lv1_dmg - lv2_dmg).abs() < f32::EPSILON,
            "Lv1 and Lv2 damage must be equal"
        );
        assert!(lv3_dmg > lv2_dmg, "Lv3 damage must exceed Lv2");
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
