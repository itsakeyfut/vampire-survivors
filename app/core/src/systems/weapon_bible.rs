//! Bible weapon — orbiting projectile bodies that circle the player.
//!
//! The Bible spawns one or more [`BibleOrb`] entities as children of the player.
//! Each orb revolves around the player at a fixed radius, damaging any enemy it
//! overlaps.  A per-enemy hit cooldown prevents the same enemy from being hit
//! more than once per orbit.
//!
//! ## Level progression
//!
//! | Level | Damage | Radius (px) | Speed (rad/s) | Orbs |
//! |-------|--------|-------------|---------------|------|
//! | 1     | 20     | 80          | 2.0           | 1    |
//! | 2     | 25     | 80          | 2.0           | 1    |
//! | 3     | 30     | 80          | 2.3           | 2    |
//! | 4     | 35     | 90          | 2.3           | 2    |
//! | 5     | 40     | 90          | 2.5           | 3    |
//! | 6     | 50     | 100         | 2.5           | 3    |
//! | 7     | 60     | 100         | 2.8           | 3    |
//! | 8     | 80     | 110         | 3.0           | 3    |
//!
//! ## Orb entities
//!
//! Each [`BibleOrb`] is spawned as a **child** of the player so its
//! [`Transform`] is in player-local space.  [`orbit_bible`] advances
//! `orbit_angle` each frame and writes the new local position to `Transform`.
//! Bevy's transform propagation then keeps [`GlobalTransform`] in sync
//! automatically.
//!
//! When the weapon levels up to a tier that requires more orbs, [`fire_bible`]
//! despawns the old orbs and spawns the new set evenly spaced at
//! `2π × i / count` radians.  A [`HashSet`] guard prevents duplicate spawns
//! when multiple same-type events arrive in the same system run.
//!
//! ## Orbit radius scaling
//!
//! The effective orbit radius is `base_radius × player.area_multiplier`,
//! consistent with other area weapons (e.g. Garlic).
//!
//! ## Hit cooldown
//!
//! [`OrbitWeapon::hit_cooldown`] is a per-enemy timer map.  When an orb
//! overlaps an enemy it starts a `DEFAULT_BIBLE_HIT_COOLDOWN`-second cooldown
//! for that enemy, preventing continuous damage spam as the orb passes through.

use std::collections::{HashMap, HashSet};
use std::f32::consts::TAU;

use bevy::prelude::*;

use crate::{
    components::{Enemy, GameSessionEntity, OrbitWeapon, Player, PlayerStats},
    config::weapon::bible::BibleParams,
    events::{DamageEnemyEvent, WeaponFiredEvent},
    resources::SpatialGrid,
    types::WeaponType,
};

// ---------------------------------------------------------------------------
// Fallback constants (used while RON config is still loading)
// ---------------------------------------------------------------------------

/// Damage per hit at each weapon level (index 0 = level 1).
const DEFAULT_BIBLE_DAMAGE_BY_LEVEL: [f32; 8] = [20.0, 25.0, 30.0, 35.0, 40.0, 50.0, 60.0, 80.0];

/// Orbit radius in pixels at each weapon level.
const DEFAULT_BIBLE_ORBIT_RADIUS_BY_LEVEL: [f32; 8] =
    [80.0, 80.0, 80.0, 90.0, 90.0, 100.0, 100.0, 110.0];

/// Angular velocity (rad/s) at each weapon level.
const DEFAULT_BIBLE_ORBIT_SPEED_BY_LEVEL: [f32; 8] = [2.0, 2.0, 2.3, 2.3, 2.5, 2.5, 2.8, 3.0];

/// Number of orbiting bodies at each weapon level.
const DEFAULT_BIBLE_COUNT_BY_LEVEL: [u32; 8] = [1, 1, 2, 2, 3, 3, 3, 3];

/// How long (seconds) to wait before hitting the same enemy again.
const DEFAULT_BIBLE_HIT_COOLDOWN: f32 = 1.5;

/// Collision radius of each orb in pixels.
const DEFAULT_BIBLE_ORB_RADIUS: f32 = 12.0;

/// Visual radius of each orb circle mesh (pixels).
const BIBLE_ORB_VISUAL_RADIUS: f32 = 12.0;

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/// Marks a single orbiting body belonging to the Bible (or UnholyVespers).
///
/// Spawned as a top-level [`GameSessionEntity`] by [`fire_bible`].  Its
/// world-space [`Transform`] is updated every frame by [`orbit_bible`] using
/// the owning player's position plus the current orbital offset.
#[derive(Component, Debug)]
pub struct BibleOrb {
    /// Player entity that owns this orb.
    pub player: Entity,
    /// Index within the orb set (0-based); determines initial phase offset.
    pub orb_index: usize,
    /// Weapon type that spawned this orb (Bible or UnholyVespers).
    pub weapon_type: WeaponType,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns or updates [`BibleOrb`] entities on each [`WeaponFiredEvent`] for
/// [`WeaponType::Bible`] or [`WeaponType::UnholyVespers`].
///
/// - On the first activation a set of [`BibleOrb`] entities is spawned evenly
///   spaced at `2π × i / count` radians.
/// - When the weapon levels up to a tier with more orbs, additional orbs are
///   appended at angles `2π × i / count_needed` for the new indices.
/// - When the count is unchanged, damage/radius/speed on each existing orb is
///   updated in place without disturbing the current orbit angles.
/// - A [`HashSet`] guards against duplicate processing when multiple events
///   for the same player arrive in the same system run.
pub fn fire_bible(
    mut fired_events: MessageReader<WeaponFiredEvent>,
    player_q: Query<&PlayerStats, With<Player>>,
    mut orb_q: Query<(Entity, &BibleOrb, &mut OrbitWeapon)>,
    bible_cfg: BibleParams,
    mut commands: Commands,
) {
    let cfg = bible_cfg.get();
    let mut processed_players: HashSet<Entity> = HashSet::new();

    for event in fired_events.read() {
        let is_unholy = event.weapon_type == WeaponType::UnholyVespers;
        if event.weapon_type != WeaponType::Bible && !is_unholy {
            continue;
        }
        if !processed_players.insert(event.player) {
            continue;
        }

        let Ok(stats) = player_q.get(event.player) else {
            continue;
        };

        let level = event.level.clamp(1, 8) as usize;

        // --- Per-level stats (with config fallback) ---
        let base_damage = cfg
            .and_then(|c| c.damage_by_level.get(level - 1).copied())
            .unwrap_or(DEFAULT_BIBLE_DAMAGE_BY_LEVEL[level - 1]);
        let base_radius = cfg
            .and_then(|c| c.orbit_radius_by_level.get(level - 1).copied())
            .unwrap_or(DEFAULT_BIBLE_ORBIT_RADIUS_BY_LEVEL[level - 1]);
        let base_speed = cfg
            .and_then(|c| c.orbit_speed_by_level.get(level - 1).copied())
            .unwrap_or(DEFAULT_BIBLE_ORBIT_SPEED_BY_LEVEL[level - 1]);
        let count_needed = cfg
            .and_then(|c| c.count_by_level.get(level - 1).copied())
            .unwrap_or(DEFAULT_BIBLE_COUNT_BY_LEVEL[level - 1]);

        let damage = base_damage * stats.damage_multiplier;
        let radius = base_radius * stats.area_multiplier;

        // --- Collect existing orbs for this player ---
        let existing: Vec<Entity> = orb_q
            .iter()
            .filter(|(_, orb, _)| orb.player == event.player)
            .map(|(e, _, _)| e)
            .collect();
        let existing_count = existing.len();

        // Update stats on all existing orbs without disturbing orbit angles.
        for entity in &existing {
            if let Ok((_, _, mut orb_weapon)) = orb_q.get_mut(*entity) {
                orb_weapon.damage = damage;
                orb_weapon.orbit_radius = radius;
                orb_weapon.orbit_speed = base_speed;
            }
        }

        // Spawn additional orbs when the count increased (weapon levelled up).
        // New orbs slot in at `2π × i / count_needed` so the full set remains
        // evenly spaced for incremental counts 1→2 and 2→3.
        // Orbs are top-level entities (not children of the player); orbit_bible
        // writes their world-space Transform each frame.
        for i in existing_count..count_needed as usize {
            let angle = TAU * i as f32 / count_needed as f32;
            commands.spawn((
                BibleOrb {
                    player: event.player,
                    orb_index: i,
                    weapon_type: event.weapon_type,
                },
                OrbitWeapon {
                    damage,
                    orbit_radius: radius,
                    orbit_speed: base_speed,
                    orbit_angle: angle,
                    hit_cooldown: HashMap::new(),
                },
                // Placeholder transform; orbit_bible sets the world position each frame.
                Transform::from_xyz(0.0, 0.0, 5.0),
                GameSessionEntity,
            ));
        }
    }
}

/// Advances all [`BibleOrb`] orbital positions each frame and deals damage to
/// overlapping enemies.
///
/// Each frame, for every orb:
/// 1. Advances `orbit_angle += orbit_speed × Δt`.
/// 2. Writes the new **world-space** position to `Transform` using the owning
///    player's world position plus the orbital offset.
/// 3. Ticks down and cleans up per-enemy hit cooldowns.
/// 4. Uses [`SpatialGrid`] to find nearby enemies and emits a
///    [`DamageEnemyEvent`] for each one within [`DEFAULT_BIBLE_ORB_RADIUS`]
///    pixels that is not currently on cooldown.
#[allow(clippy::type_complexity)]
pub fn orbit_bible(
    time: Res<Time>,
    player_q: Query<&Transform, (With<Player>, Without<BibleOrb>)>,
    mut orb_q: Query<(&BibleOrb, &mut OrbitWeapon, &mut Transform), Without<Player>>,
    enemy_q: Query<&Transform, (With<Enemy>, Without<BibleOrb>)>,
    spatial_grid: Res<SpatialGrid>,
    mut damage_events: MessageWriter<DamageEnemyEvent>,
) {
    let dt = time.delta_secs();

    for (orb, mut orb_weapon, mut transform) in orb_q.iter_mut() {
        // --- Advance orbital angle ---
        orb_weapon.orbit_angle += orb_weapon.orbit_speed * dt;
        // Keep angle in [0, TAU) to prevent unbounded growth.
        orb_weapon.orbit_angle = orb_weapon.orbit_angle.rem_euclid(TAU);

        let angle = orb_weapon.orbit_angle;
        let radius = orb_weapon.orbit_radius;

        // --- Compute world position from player position + orbit offset ---
        let player_world_pos = player_q
            .get(orb.player)
            .map(|tf| tf.translation.truncate())
            .unwrap_or(Vec2::ZERO);
        let orb_world_pos =
            player_world_pos + Vec2::new(radius * angle.cos(), radius * angle.sin());

        // --- Update world-space transform ---
        transform.translation.x = orb_world_pos.x;
        transform.translation.y = orb_world_pos.y;
        // Keep z fixed (above enemies, below player).
        transform.translation.z = 5.0;

        // --- Tick down and purge expired hit cooldowns ---
        orb_weapon.hit_cooldown.retain(|_, cd| {
            *cd -= dt;
            *cd > 0.0
        });

        // --- Damage enemies within orb collision radius ---
        let check_radius = radius + DEFAULT_BIBLE_ORB_RADIUS;
        for enemy_entity in spatial_grid.get_nearby(orb_world_pos, check_radius) {
            if orb_weapon.hit_cooldown.contains_key(&enemy_entity) {
                continue;
            }
            let Ok(enemy_tf) = enemy_q.get(enemy_entity) else {
                continue;
            };
            let dist = (enemy_tf.translation.truncate() - orb_world_pos).length();
            if dist <= DEFAULT_BIBLE_ORB_RADIUS {
                damage_events.write(DamageEnemyEvent {
                    entity: enemy_entity,
                    damage: orb_weapon.damage,
                    weapon_type: orb.weapon_type,
                });
                orb_weapon
                    .hit_cooldown
                    .insert(enemy_entity, DEFAULT_BIBLE_HIT_COOLDOWN);
            }
        }
    }
}

/// Adds a circle visual mesh to newly spawned [`BibleOrb`] entities.
///
/// Runs whenever `Added<BibleOrb>` is detected (the frame after the entity
/// is flushed from [`fire_bible`]'s deferred commands).  Uses a fixed-radius
/// [`Circle`] mesh so the visual size matches the collision radius regardless
/// of level.
pub fn spawn_bible_visual(
    mut commands: Commands,
    query: Query<Entity, Added<BibleOrb>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for entity in query.iter() {
        let mesh = meshes.add(Circle::new(BIBLE_ORB_VISUAL_RADIUS));
        let material = materials.add(ColorMaterial::from_color(Color::srgba(1.0, 0.85, 0.0, 0.9)));
        commands
            .entity(entity)
            .insert((Mesh2d(mesh), MeshMaterial2d(material)));
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
        components::WeaponInventory,
        events::WeaponFiredEvent,
        resources::SpatialGrid,
        types::{WeaponState, WeaponType},
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
        let mut weapon = WeaponState::new(WeaponType::Bible);
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
                Enemy::from_type(crate::types::EnemyType::Bat, 1.0),
                Transform::from_xyz(pos.x, pos.y, 1.0),
            ))
            .id()
    }

    fn fire_once(app: &mut App, player: Entity, weapon_type: WeaponType, level: u8) {
        app.world_mut().write_message(WeaponFiredEvent {
            player,
            weapon_type,
            level,
        });
        app.world_mut()
            .run_system_once(fire_bible)
            .expect("fire_bible should run");
        app.world_mut().flush();
    }

    fn count_orbs(app: &mut App) -> usize {
        app.world_mut()
            .query::<&BibleOrb>()
            .iter(app.world())
            .count()
    }

    // -----------------------------------------------------------------------
    // fire_bible tests
    // -----------------------------------------------------------------------

    /// First fire at level 1 spawns exactly one BibleOrb.
    #[test]
    fn bible_spawns_one_orb_at_level_1() {
        let mut app = build_app();
        let player = spawn_player(&mut app);

        fire_once(&mut app, player, WeaponType::Bible, 1);

        assert_eq!(count_orbs(&mut app), 1, "level 1 should produce 1 orb");
    }

    /// Level 3 should produce 2 orbs.
    #[test]
    fn bible_spawns_two_orbs_at_level_3() {
        let mut app = build_app();
        let player = spawn_player(&mut app);

        fire_once(&mut app, player, WeaponType::Bible, 3);

        assert_eq!(count_orbs(&mut app), 2, "level 3 should produce 2 orbs");
    }

    /// Level 5 should produce 3 orbs.
    #[test]
    fn bible_spawns_three_orbs_at_level_5() {
        let mut app = build_app();
        let player = spawn_player(&mut app);

        fire_once(&mut app, player, WeaponType::Bible, 5);

        assert_eq!(count_orbs(&mut app), 3, "level 5 should produce 3 orbs");
    }

    /// Firing again at the same level does not duplicate orbs.
    #[test]
    fn bible_does_not_duplicate_orbs_on_second_fire() {
        let mut app = build_app();
        let player = spawn_player(&mut app);

        fire_once(&mut app, player, WeaponType::Bible, 1);
        fire_once(&mut app, player, WeaponType::Bible, 1);

        assert_eq!(count_orbs(&mut app), 1, "second fire should not add orbs");
    }

    /// Same-frame duplicate events do not produce extra orbs.
    #[test]
    fn bible_not_duplicated_on_same_frame_events() {
        let mut app = build_app();
        let player = spawn_player(&mut app);

        app.world_mut().write_message(WeaponFiredEvent {
            player,
            weapon_type: WeaponType::Bible,
            level: 1,
        });
        app.world_mut().write_message(WeaponFiredEvent {
            player,
            weapon_type: WeaponType::Bible,
            level: 1,
        });
        app.world_mut()
            .run_system_once(fire_bible)
            .expect("fire_bible should run");
        app.world_mut().flush();

        assert_eq!(
            count_orbs(&mut app),
            1,
            "two same-frame events must not produce two orbs"
        );
    }

    /// Level-up from 1→2 orbs spawns one additional orb.
    ///
    /// Directly pre-spawns 1 orb entity (bypassing fire_bible) to simulate
    /// the Lv1 state, then fires a level-3 event which should add a second orb.
    /// This avoids message-cursor issues that arise when run_system_once is
    /// called twice (each call resets the reader cursor to position 0).
    #[test]
    fn bible_spawns_additional_orb_on_count_increase() {
        let mut app = build_app();
        let player = spawn_player(&mut app);

        // Pre-spawn 1 orb (level-1 state) without going through fire_bible.
        app.world_mut().spawn((
            BibleOrb {
                player,
                orb_index: 0,
                weapon_type: WeaponType::Bible,
            },
            OrbitWeapon {
                damage: 20.0,
                orbit_radius: 80.0,
                orbit_speed: 2.0,
                orbit_angle: 0.0,
                hit_cooldown: HashMap::new(),
            },
            Transform::from_xyz(0.0, 0.0, 5.0),
            GameSessionEntity,
        ));
        assert_eq!(count_orbs(&mut app), 1);

        // Fire at level 3 (count_needed=2) — fire_bible sees 1 existing orb
        // and spawns 1 additional.
        fire_once(&mut app, player, WeaponType::Bible, 3);
        assert_eq!(
            count_orbs(&mut app),
            2,
            "level-up from 1→2 orbs should add one more"
        );
    }

    /// UnholyVespers (Bible evolution) also activates the orbs.
    #[test]
    fn unholy_vespers_fires_bible_orbs() {
        let mut app = build_app();
        let player = spawn_player(&mut app);

        fire_once(&mut app, player, WeaponType::UnholyVespers, 1);

        assert_eq!(
            count_orbs(&mut app),
            1,
            "UnholyVespers should spawn Bible orbs"
        );
    }

    /// Non-Bible weapon events are ignored by fire_bible.
    #[test]
    fn other_weapons_do_not_spawn_bible_orbs() {
        let mut app = build_app();
        let player = spawn_player(&mut app);

        fire_once(&mut app, player, WeaponType::MagicWand, 1);

        assert_eq!(
            count_orbs(&mut app),
            0,
            "fire_bible should ignore non-Bible events"
        );
    }

    // -----------------------------------------------------------------------
    // orbit_bible tests
    // -----------------------------------------------------------------------

    /// orbit_bible advances the orbit_angle each frame.
    #[test]
    fn orbit_bible_advances_angle() {
        let mut app = build_app();
        let player = spawn_player(&mut app);

        fire_once(&mut app, player, WeaponType::Bible, 1);

        // Orb starts at angle 0 (level 1, only orb).
        let initial_angle = app
            .world_mut()
            .query::<&OrbitWeapon>()
            .iter(app.world())
            .next()
            .expect("orb should exist")
            .orbit_angle;

        advance(&mut app);
        app.world_mut()
            .run_system_once(crate::systems::spatial::update_spatial_grid)
            .unwrap();
        app.world_mut()
            .run_system_once(orbit_bible)
            .expect("orbit_bible should run");

        let new_angle = app
            .world_mut()
            .query::<&OrbitWeapon>()
            .iter(app.world())
            .next()
            .unwrap()
            .orbit_angle;

        assert!(
            new_angle > initial_angle,
            "orbit angle should increase each frame ({initial_angle} → {new_angle})"
        );
    }

    /// orbit_bible damages an enemy that overlaps the orb position.
    #[test]
    fn orbit_bible_hits_enemy_at_orb_position() {
        let mut app = build_app();
        let player = spawn_player(&mut app);

        fire_once(&mut app, player, WeaponType::Bible, 1);

        // Read the orb's initial position (angle=0, radius=80 → world x≈80, y=0).
        let orb_radius = app
            .world_mut()
            .query::<&OrbitWeapon>()
            .iter(app.world())
            .next()
            .unwrap()
            .orbit_radius;

        // Place enemy directly on the orb's starting position (within 12 px).
        spawn_enemy(&mut app, Vec2::new(orb_radius, 0.0));

        advance(&mut app);
        app.world_mut()
            .run_system_once(crate::systems::spatial::update_spatial_grid)
            .unwrap();
        app.world_mut()
            .run_system_once(orbit_bible)
            .expect("orbit_bible should run");

        let messages = app.world().resource::<Messages<DamageEnemyEvent>>();
        let mut cursor = messages.get_cursor();
        let events: Vec<_> = cursor.read(messages).collect();
        assert_eq!(events.len(), 1, "enemy at orb position should be hit");
        assert!(events[0].damage > 0.0);
    }

    /// Hit cooldown prevents the same enemy from being hit twice in a row.
    #[test]
    fn orbit_bible_hit_cooldown_prevents_repeat_damage() {
        let mut app = build_app();
        let player = spawn_player(&mut app);

        fire_once(&mut app, player, WeaponType::Bible, 1);

        let orb_radius = app
            .world_mut()
            .query::<&OrbitWeapon>()
            .iter(app.world())
            .next()
            .unwrap()
            .orbit_radius;

        spawn_enemy(&mut app, Vec2::new(orb_radius, 0.0));

        // First tick — enemy gets hit.
        advance(&mut app);
        app.world_mut()
            .run_system_once(crate::systems::spatial::update_spatial_grid)
            .unwrap();
        app.world_mut().run_system_once(orbit_bible).unwrap();

        // Second tick (same frame) — enemy should be on cooldown.
        advance(&mut app);
        app.world_mut()
            .run_system_once(crate::systems::spatial::update_spatial_grid)
            .unwrap();
        app.world_mut().run_system_once(orbit_bible).unwrap();

        let messages = app.world().resource::<Messages<DamageEnemyEvent>>();
        let mut cursor = messages.get_cursor();
        let events: Vec<_> = cursor.read(messages).collect();
        assert_eq!(
            events.len(),
            1,
            "enemy should only be hit once due to hit cooldown"
        );
    }

    /// Two orbs at level 3 start at evenly spaced angles (0 and π).
    #[test]
    fn level_3_orbs_are_evenly_spaced() {
        let mut app = build_app();
        let player = spawn_player(&mut app);

        fire_once(&mut app, player, WeaponType::Bible, 3);

        let angles: Vec<f32> = app
            .world_mut()
            .query::<&OrbitWeapon>()
            .iter(app.world())
            .map(|o| o.orbit_angle)
            .collect();

        assert_eq!(angles.len(), 2);
        let diff = (angles[1] - angles[0]).abs();
        let expected = TAU / 2.0; // π
        assert!(
            (diff - expected).abs() < 1e-5,
            "two orbs should be π apart, got {diff}"
        );
    }

    /// Level 5 orbs start at evenly spaced angles (0, 2π/3, 4π/3).
    #[test]
    fn level_5_orbs_are_evenly_spaced() {
        let mut app = build_app();
        let player = spawn_player(&mut app);

        fire_once(&mut app, player, WeaponType::Bible, 5);

        let mut angles: Vec<f32> = app
            .world_mut()
            .query::<&OrbitWeapon>()
            .iter(app.world())
            .map(|o| o.orbit_angle)
            .collect();
        angles.sort_by(|a, b| a.partial_cmp(b).unwrap());

        assert_eq!(angles.len(), 3);
        let expected_step = TAU / 3.0;
        assert!(
            (angles[1] - angles[0] - expected_step).abs() < 1e-5,
            "orbs should be 2π/3 apart"
        );
        assert!(
            (angles[2] - angles[1] - expected_step).abs() < 1e-5,
            "orbs should be 2π/3 apart"
        );
    }

    /// Damage increases with level.
    #[test]
    fn bible_damage_increases_with_level() {
        let lv1 = DEFAULT_BIBLE_DAMAGE_BY_LEVEL[0];
        let lv8 = DEFAULT_BIBLE_DAMAGE_BY_LEVEL[7];
        assert!(lv8 > lv1, "Lv8 damage ({lv8}) should exceed Lv1 ({lv1})");
    }

    /// Orbit radius increases with level.
    #[test]
    fn bible_radius_increases_with_level() {
        let lv1 = DEFAULT_BIBLE_ORBIT_RADIUS_BY_LEVEL[0];
        let lv8 = DEFAULT_BIBLE_ORBIT_RADIUS_BY_LEVEL[7];
        assert!(lv8 > lv1, "Lv8 radius ({lv8}) should exceed Lv1 ({lv1})");
    }

    /// `PlayerStats::area_multiplier` scales the effective orbit radius.
    #[test]
    fn bible_radius_scales_with_area_multiplier() {
        let mut app = build_app();
        let player = spawn_player(&mut app);
        app.world_mut()
            .get_mut::<PlayerStats>(player)
            .unwrap()
            .area_multiplier = 2.0;

        fire_once(&mut app, player, WeaponType::Bible, 1);

        let orb_radius = app
            .world_mut()
            .query::<&OrbitWeapon>()
            .iter(app.world())
            .next()
            .unwrap()
            .orbit_radius;
        // Default Lv1 radius is 80; doubled → 160.
        assert!(
            (orb_radius - 160.0).abs() < 1e-5,
            "area_multiplier=2 should double the orbit radius, got {orb_radius}"
        );
    }
}
