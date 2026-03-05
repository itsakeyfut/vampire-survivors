//! Thunder Ring weapon — random lightning strikes against on-screen enemies.
//!
//! The Thunder Ring selects up to `count` random enemies anywhere on screen
//! and instantly damages them each time the weapon fires.  A short-lived
//! [`ThunderStrikeEffect`] flash is spawned at each struck enemy's position
//! as visual feedback.
//!
//! ## Level progression
//!
//! | Level | Damage | Strikes |
//! |-------|--------|---------|
//! | 1     | 40     | 1       |
//! | 2     | 50     | 1       |
//! | 3     | 60     | 2       |
//! | 4     | 60     | 2       |
//! | 5     | 70     | 3       |
//! | 6     | 80     | 3       |
//! | 7     | 90     | 3       |
//! | 8     | 100    | 4       |
//!
//! ## Strike selection
//!
//! All living enemy entities are collected into a candidate list and partially
//! shuffled via Fisher-Yates.  The first `count + extra_projectiles` entries
//! become targets, ensuring no enemy is struck twice per activation.  If fewer
//! enemies exist than `count`, all of them are hit.
//!
//! ## Visual effect
//!
//! Each strike spawns a [`ThunderStrikeEffect`] entity at the enemy's
//! world position.  [`despawn_thunder_effects`] removes it after
//! `effect_duration` seconds.

use rand::RngExt;

use bevy::prelude::*;

use crate::{
    components::{Enemy, GameSessionEntity, Player, PlayerStats},
    config::weapon::thunder_ring::ThunderRingParams,
    events::{DamageEnemyEvent, WeaponFiredEvent},
    resources::SpatialGrid,
    types::WeaponType,
};

// ---------------------------------------------------------------------------
// Fallback constants (used while RON config is still loading)
// ---------------------------------------------------------------------------

/// Damage per strike at each weapon level (index 0 = level 1).
const DEFAULT_THUNDER_RING_DAMAGE_BY_LEVEL: [f32; 8] =
    [40.0, 50.0, 60.0, 60.0, 70.0, 80.0, 90.0, 100.0];

/// Number of strikes per activation at each weapon level.
const DEFAULT_THUNDER_RING_COUNT_BY_LEVEL: [u32; 8] = [1, 1, 2, 2, 3, 3, 3, 4];

/// Fallback visual effect duration while RON config is still loading.
const DEFAULT_THUNDER_RING_EFFECT_DURATION: f32 = 0.2;

/// Fallback flash sprite side length (pixels) while RON config is still loading.
const DEFAULT_THUNDER_RING_VISUAL_SIZE: f32 = 24.0;

/// Fallback flash sprite RGBA color while RON config is still loading.
const DEFAULT_THUNDER_RING_VISUAL_COLOR: (f32, f32, f32, f32) = (0.9, 1.0, 0.2, 0.85);

/// Fallback strike sprite z-depth while RON config is still loading.
const DEFAULT_THUNDER_RING_STRIKE_Z: f32 = 6.0;

/// Fallback maximum targeting range in pixels while RON config is still loading.
/// Approximates the visible screen radius so only on-screen enemies are struck.
const DEFAULT_THUNDER_RING_TARGET_RANGE: f32 = 800.0;

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/// Marks a short-lived lightning flash entity spawned by [`fire_thunder_ring`].
///
/// [`despawn_thunder_effects`] removes these entities once `remaining` reaches
/// zero.
#[derive(Component, Debug)]
pub struct ThunderStrikeEffect {
    /// Seconds remaining before this entity is despawned.
    pub remaining: f32,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Fires the Thunder Ring by striking random enemies on the current frame.
///
/// On each [`WeaponFiredEvent`] for [`WeaponType::ThunderRing`] or
/// [`WeaponType::LightningRing`]:
/// 1. Collects all living enemy entities with their world positions.
/// 2. Partially shuffles the list (Fisher-Yates) to pick `count +
///    extra_projectiles` unique targets (capped at the total enemy count).
/// 3. Emits a [`DamageEnemyEvent`] for each target.
/// 4. Spawns a [`ThunderStrikeEffect`] sprite at each target's position.
///
/// Every event is processed independently — multiple events in the same frame
/// (e.g. catch-up ticks from [`super::weapon_cooldown::tick_weapon_cooldowns`]
/// after a frame hitch) each trigger a full activation.
pub fn fire_thunder_ring(
    mut fired_events: MessageReader<WeaponFiredEvent>,
    player_q: Query<(&Transform, &PlayerStats), With<Player>>,
    enemy_q: Query<&Transform, With<Enemy>>,
    spatial_grid: Res<SpatialGrid>,
    thunder_cfg: ThunderRingParams,
    mut damage_events: MessageWriter<DamageEnemyEvent>,
    mut commands: Commands,
) {
    let cfg = thunder_cfg.get();

    for event in fired_events.read() {
        let is_lightning_ring = event.weapon_type == WeaponType::LightningRing;
        if event.weapon_type != WeaponType::ThunderRing && !is_lightning_ring {
            continue;
        }

        let Ok((player_tf, stats)) = player_q.get(event.player) else {
            continue;
        };

        let player_pos = player_tf.translation.truncate();
        let level = event.level.clamp(1, 8) as usize;

        let base_damage = cfg
            .and_then(|c| c.damage_by_level.get(level - 1).copied())
            .unwrap_or(DEFAULT_THUNDER_RING_DAMAGE_BY_LEVEL[level - 1]);
        let base_count = cfg
            .and_then(|c| c.count_by_level.get(level - 1).copied())
            .unwrap_or(DEFAULT_THUNDER_RING_COUNT_BY_LEVEL[level - 1]);
        let effect_duration = cfg
            .map(|c| c.effect_duration)
            .unwrap_or(DEFAULT_THUNDER_RING_EFFECT_DURATION);
        let visual_size = cfg
            .map(|c| c.visual_size)
            .unwrap_or(DEFAULT_THUNDER_RING_VISUAL_SIZE);
        let visual_color = cfg
            .map(|c| c.visual_color)
            .unwrap_or(DEFAULT_THUNDER_RING_VISUAL_COLOR);
        let strike_z = cfg
            .map(|c| c.strike_z)
            .unwrap_or(DEFAULT_THUNDER_RING_STRIKE_Z);

        let damage = base_damage * stats.damage_multiplier;
        let count = (base_count + stats.extra_projectiles) as usize;
        let target_range = cfg
            .map(|c| c.target_range)
            .unwrap_or(DEFAULT_THUNDER_RING_TARGET_RANGE);

        // Collect enemies within target_range using SpatialGrid so that only
        // on-screen (or near-screen) enemies are considered as targets.
        // Must run after update_spatial_grid.
        let mut candidates: Vec<(Entity, Vec2)> = Vec::new();
        for e in spatial_grid.get_nearby(player_pos, target_range) {
            if let Ok(tf) = enemy_q.get(e) {
                candidates.push((e, tf.translation.truncate()));
            }
        }

        // LightningRing (evolved form) hits every enemy on screen; ThunderRing
        // is capped to `count` targets.
        let pick_count = if is_lightning_ring {
            candidates.len()
        } else {
            count.min(candidates.len())
        };

        // Fisher-Yates partial shuffle: move `pick_count` random entries to
        // the front of `candidates` so they can be taken as unique targets.
        let mut rng = rand::rng();
        for i in 0..pick_count {
            let j = i + rng.random_range(0..(candidates.len() - i));
            candidates.swap(i, j);
        }

        for (enemy_entity, enemy_pos) in &candidates[..pick_count] {
            damage_events.write(DamageEnemyEvent {
                entity: *enemy_entity,
                damage,
                weapon_type: event.weapon_type,
            });
            let (r, g, b, a) = visual_color;
            commands.spawn((
                ThunderStrikeEffect {
                    remaining: effect_duration,
                },
                Sprite {
                    color: Color::srgba(r, g, b, a),
                    custom_size: Some(Vec2::splat(visual_size)),
                    ..default()
                },
                Transform::from_xyz(enemy_pos.x, enemy_pos.y, strike_z),
                GameSessionEntity,
            ));
        }
    }
}

/// Ticks down [`ThunderStrikeEffect`] lifetimes and despawns expired entities.
pub fn despawn_thunder_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut effect_q: Query<(Entity, &mut ThunderStrikeEffect)>,
) {
    let delta = time.delta_secs();
    for (entity, mut effect) in effect_q.iter_mut() {
        effect.remaining -= delta;
        if effect.remaining <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
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
        let mut weapon = WeaponState::new(WeaponType::ThunderRing);
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
            .run_system_once(crate::systems::spatial::update_spatial_grid)
            .expect("update_spatial_grid should run");
        app.world_mut()
            .run_system_once(fire_thunder_ring)
            .expect("fire_thunder_ring should run");
        app.world_mut().flush();
    }

    fn damage_events(app: &App) -> Vec<DamageEnemyEvent> {
        let messages = app.world().resource::<Messages<DamageEnemyEvent>>();
        let mut cursor = messages.get_cursor();
        cursor.read(messages).cloned().collect()
    }

    fn count_effects(app: &mut App) -> usize {
        app.world_mut()
            .query::<&ThunderStrikeEffect>()
            .iter(app.world())
            .count()
    }

    // -----------------------------------------------------------------------
    // fire_thunder_ring tests
    // -----------------------------------------------------------------------

    /// Non-ThunderRing weapon events are ignored.
    #[test]
    fn other_weapons_do_not_fire_thunder_ring() {
        let mut app = build_app();
        let player = spawn_player(&mut app);
        spawn_enemy(&mut app, Vec2::new(100.0, 0.0));

        fire_once(&mut app, player, WeaponType::MagicWand, 1);

        assert_eq!(
            damage_events(&app).len(),
            0,
            "fire_thunder_ring should ignore non-ThunderRing events"
        );
    }

    /// Thunder Ring fires at a single enemy at level 1.
    #[test]
    fn fire_thunder_ring_hits_one_enemy_at_level_1() {
        let mut app = build_app();
        let player = spawn_player(&mut app);
        spawn_enemy(&mut app, Vec2::new(100.0, 0.0));

        fire_once(&mut app, player, WeaponType::ThunderRing, 1);

        let events = damage_events(&app);
        assert_eq!(events.len(), 1, "level 1 should hit exactly one enemy");
        assert!(events[0].damage > 0.0);
        assert_eq!(events[0].weapon_type, WeaponType::ThunderRing);
    }

    /// No enemies → no damage events.
    #[test]
    fn fire_thunder_ring_no_enemies_no_events() {
        let mut app = build_app();
        let player = spawn_player(&mut app);

        fire_once(&mut app, player, WeaponType::ThunderRing, 1);

        assert_eq!(
            damage_events(&app).len(),
            0,
            "no enemies should produce no damage events"
        );
    }

    /// Level 3 (count=2) hits two different enemies when two enemies exist.
    #[test]
    fn fire_thunder_ring_level_3_hits_two_enemies() {
        let mut app = build_app();
        let player = spawn_player(&mut app);
        let e1 = spawn_enemy(&mut app, Vec2::new(50.0, 0.0));
        let e2 = spawn_enemy(&mut app, Vec2::new(-50.0, 0.0));

        fire_once(&mut app, player, WeaponType::ThunderRing, 3);

        let events = damage_events(&app);
        assert_eq!(events.len(), 2, "level 3 should hit two enemies");
        // Both enemies must be different entities.
        assert_ne!(
            events[0].entity, events[1].entity,
            "strikes must target different enemies"
        );
        let targets: HashSet<_> = events.iter().map(|e| e.entity).collect();
        assert!(targets.contains(&e1));
        assert!(targets.contains(&e2));
    }

    /// When fewer enemies exist than the strike count, all enemies are hit once.
    #[test]
    fn fire_thunder_ring_count_capped_by_enemy_count() {
        let mut app = build_app();
        let player = spawn_player(&mut app);
        // Only 1 enemy, but level 3 wants 2 strikes.
        spawn_enemy(&mut app, Vec2::new(100.0, 0.0));

        fire_once(&mut app, player, WeaponType::ThunderRing, 3);

        assert_eq!(
            damage_events(&app).len(),
            1,
            "only 1 enemy available — should hit exactly 1"
        );
    }

    /// LightningRing (evolution) hits ALL enemies regardless of count.
    #[test]
    fn lightning_ring_hits_all_enemies() {
        let mut app = build_app();
        let player = spawn_player(&mut app);
        // Spawn more enemies than level-1 ThunderRing count (1).
        spawn_enemy(&mut app, Vec2::new(100.0, 0.0));
        spawn_enemy(&mut app, Vec2::new(-100.0, 0.0));
        spawn_enemy(&mut app, Vec2::new(0.0, 100.0));
        spawn_enemy(&mut app, Vec2::new(0.0, -100.0));
        spawn_enemy(&mut app, Vec2::new(50.0, 50.0));

        fire_once(&mut app, player, WeaponType::LightningRing, 1);

        let events = damage_events(&app);
        assert_eq!(
            events.len(),
            5,
            "LightningRing should hit all 5 enemies, got {}",
            events.len()
        );
        for e in &events {
            assert_eq!(e.weapon_type, WeaponType::LightningRing);
        }
    }

    /// `extra_projectiles` increases the number of simultaneous strikes.
    #[test]
    fn fire_thunder_ring_respects_extra_projectiles() {
        let mut app = build_app();
        let player = spawn_player(&mut app);
        // Lv1 count=1, but extra_projectiles=2 → 3 strikes total.
        app.world_mut()
            .get_mut::<PlayerStats>(player)
            .unwrap()
            .extra_projectiles = 2;
        spawn_enemy(&mut app, Vec2::new(50.0, 0.0));
        spawn_enemy(&mut app, Vec2::new(-50.0, 0.0));
        spawn_enemy(&mut app, Vec2::new(0.0, 100.0));

        fire_once(&mut app, player, WeaponType::ThunderRing, 1);

        assert_eq!(
            damage_events(&app).len(),
            3,
            "extra_projectiles=2 with count=1 should hit 3 enemies"
        );
    }

    /// Damage scales with `damage_multiplier`.
    #[test]
    fn fire_thunder_ring_damage_scales_with_multiplier() {
        let mut app = build_app();
        let player = spawn_player(&mut app);
        app.world_mut()
            .get_mut::<PlayerStats>(player)
            .unwrap()
            .damage_multiplier = 2.0;
        spawn_enemy(&mut app, Vec2::new(100.0, 0.0));

        fire_once(&mut app, player, WeaponType::ThunderRing, 1);

        let events = damage_events(&app);
        // Default Lv1 damage is 40 × 2.0 = 80.
        assert!(
            (events[0].damage - 80.0).abs() < 1e-5,
            "damage_multiplier=2 should double the damage, got {}",
            events[0].damage
        );
    }

    /// Damage increases with level.
    #[test]
    fn thunder_ring_damage_increases_with_level() {
        let lv1 = DEFAULT_THUNDER_RING_DAMAGE_BY_LEVEL[0];
        let lv8 = DEFAULT_THUNDER_RING_DAMAGE_BY_LEVEL[7];
        assert!(lv8 > lv1, "Lv8 damage ({lv8}) should exceed Lv1 ({lv1})");
    }

    /// Strike count increases at higher levels.
    #[test]
    fn thunder_ring_count_increases_with_level() {
        let lv1 = DEFAULT_THUNDER_RING_COUNT_BY_LEVEL[0];
        let lv8 = DEFAULT_THUNDER_RING_COUNT_BY_LEVEL[7];
        assert!(lv8 > lv1, "Lv8 count ({lv8}) should exceed Lv1 ({lv1})");
    }

    /// A ThunderStrikeEffect entity is spawned at the enemy's position.
    #[test]
    fn strike_effect_spawned_at_enemy_position() {
        let mut app = build_app();
        let player = spawn_player(&mut app);
        let enemy_pos = Vec2::new(123.0, 45.0);
        spawn_enemy(&mut app, enemy_pos);

        fire_once(&mut app, player, WeaponType::ThunderRing, 1);

        let mut q = app
            .world_mut()
            .query::<(&ThunderStrikeEffect, &Transform)>();
        let effects: Vec<_> = q.iter(app.world()).collect();
        assert_eq!(
            effects.len(),
            1,
            "one ThunderStrikeEffect should be spawned"
        );
        let (_, tf) = effects[0];
        assert!(
            (tf.translation.x - enemy_pos.x).abs() < 1e-3,
            "effect x should match enemy x"
        );
        assert!(
            (tf.translation.y - enemy_pos.y).abs() < 1e-3,
            "effect y should match enemy y"
        );
    }

    /// Two same-frame events produce two independent activations.
    ///
    /// `tick_weapon_cooldowns` intentionally emits multiple events in one
    /// frame to catch up on missed ticks (e.g. after a frame hitch).  Each
    /// event must fire independently so no activations are silently dropped.
    #[test]
    fn thunder_ring_fires_all_same_frame_events() {
        let mut app = build_app();
        let player = spawn_player(&mut app);
        spawn_enemy(&mut app, Vec2::new(100.0, 0.0));

        app.world_mut().write_message(WeaponFiredEvent {
            player,
            weapon_type: WeaponType::ThunderRing,
            level: 1,
        });
        app.world_mut().write_message(WeaponFiredEvent {
            player,
            weapon_type: WeaponType::ThunderRing,
            level: 1,
        });
        app.world_mut()
            .run_system_once(crate::systems::spatial::update_spatial_grid)
            .expect("update_spatial_grid should run");
        app.world_mut()
            .run_system_once(fire_thunder_ring)
            .expect("fire_thunder_ring should run");
        app.world_mut().flush();

        assert_eq!(
            damage_events(&app).len(),
            2,
            "two same-frame events must each produce an independent activation"
        );
    }

    // -----------------------------------------------------------------------
    // despawn_thunder_effects tests
    // -----------------------------------------------------------------------

    /// Effects that still have remaining time are not despawned.
    #[test]
    fn despawn_thunder_effects_keeps_active_effects() {
        let mut app = build_app();
        let player = spawn_player(&mut app);
        spawn_enemy(&mut app, Vec2::new(100.0, 0.0));

        fire_once(&mut app, player, WeaponType::ThunderRing, 1);
        assert_eq!(count_effects(&mut app), 1);

        // Only a tiny delta — effect should still be alive.
        advance(&mut app);
        app.world_mut()
            .run_system_once(despawn_thunder_effects)
            .unwrap();

        assert_eq!(
            count_effects(&mut app),
            1,
            "effect should survive a single short frame"
        );
    }

    /// Effects are despawned after their lifetime expires.
    #[test]
    fn despawn_thunder_effects_removes_expired_effects() {
        let mut app = build_app();
        let player = spawn_player(&mut app);
        spawn_enemy(&mut app, Vec2::new(100.0, 0.0));

        fire_once(&mut app, player, WeaponType::ThunderRing, 1);
        assert_eq!(count_effects(&mut app), 1);

        // Advance past the effect duration (0.2 s default).
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(0.5));
        app.world_mut()
            .run_system_once(despawn_thunder_effects)
            .unwrap();

        assert_eq!(
            count_effects(&mut app),
            0,
            "effect should be despawned after its lifetime expires"
        );
    }
}
