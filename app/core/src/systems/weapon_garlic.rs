//! Garlic weapon — a persistent damage aura centred on the player.
//!
//! The Garlic surrounds the player with a circular aura that damages all
//! enemies within its radius each time the weapon "fires" (i.e. on each
//! cooldown tick).  At higher levels both the damage and the radius increase.
//!
//! ## Level progression
//!
//! | Level | Damage/tick | Radius (px) |
//! |-------|-------------|-------------|
//! | 1     | 5           | 80          |
//! | 2     | 5           | 90          |
//! | 3     | 8           | 90          |
//! | 4     | 8           | 100         |
//! | 5     | 10          | 110         |
//! | 6     | 12          | 120         |
//! | 7     | 15          | 130         |
//! | 8     | 20          | 150         |
//!
//! ## Aura entity
//!
//! A [`GarlicAura`] entity is spawned as a child of the player on the first
//! [`WeaponFiredEvent`] for [`WeaponType::Garlic`] (or [`WeaponType::SoulEater`]).
//! It carries an [`AuraWeapon`] component that holds the current damage and
//! radius.  The circle visual is added by [`spawn_garlic_visual`] (via
//! `Mesh2d(Circle::new(1.0))`) and kept in sync by [`update_garlic_visual`]
//! (via `Transform::scale`).
//!
//! As a child entity its [`Transform`] is relative to the player, so it tracks
//! the player position automatically via Bevy's transform propagation.
//!
//! ## area_multiplier scaling
//!
//! The effective radius is `base_radius × player.area_multiplier`, matching
//! the behaviour of other area weapons (e.g. Whip).
//!
//! ## Tick interval
//!
//! The aura fires every time its weapon cooldown expires (driven by
//! `tick_weapon_cooldowns`), using the base-cooldown table in
//! `types/weapon.rs`.

use std::collections::HashSet;

use bevy::prelude::*;

use crate::{
    components::{AuraWeapon, Enemy, GameSessionEntity, Player, PlayerStats},
    config::weapon::garlic::GarlicParams,
    events::{DamageEnemyEvent, WeaponFiredEvent},
    resources::SpatialGrid,
    types::WeaponType,
};

// ---------------------------------------------------------------------------
// Fallback constants (used while RON config is still loading)
// ---------------------------------------------------------------------------

/// Damage per aura tick at each weapon level (index 0 = level 1).
const DEFAULT_GARLIC_DAMAGE_BY_LEVEL: [f32; 8] = [5.0, 5.0, 8.0, 8.0, 10.0, 12.0, 15.0, 20.0];

/// Aura radius in pixels at each weapon level (index 0 = level 1).
const DEFAULT_GARLIC_RADIUS_BY_LEVEL: [f32; 8] =
    [80.0, 90.0, 90.0, 100.0, 110.0, 120.0, 130.0, 150.0];

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/// Marks the persistent aura entity for Garlic and SoulEater.
///
/// Spawned as a child of the player entity on the first [`WeaponFiredEvent`]
/// for [`WeaponType::Garlic`] or [`WeaponType::SoulEater`].  Stores the
/// owning player entity so that [`fire_garlic`] can match events to their
/// existing aura.
#[derive(Component, Debug)]
pub struct GarlicAura {
    /// Player entity that owns this aura.
    pub player: Entity,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Activates the Garlic aura when a [`WeaponFiredEvent`] arrives for
/// [`WeaponType::Garlic`] or [`WeaponType::SoulEater`].
///
/// - On the first activation a [`GarlicAura`] child entity is spawned under
///   the player (game-logic components only; the circle visual is added by
///   [`spawn_garlic_visual`] on the following frame).
/// - On every subsequent activation the [`AuraWeapon`] stats are updated;
///   [`update_garlic_visual`] then syncs the visual radius automatically.
/// - A [`DamageEnemyEvent`] is emitted for each enemy within the
///   area-scaled radius (found via [`SpatialGrid`]).
/// - A [`HashSet`] guards against duplicate spawns when multiple
///   same-type events arrive in the same system run (commands are deferred,
///   so the freshly-queued entity is not yet visible to `aura_q`).
///
/// Must run after [`super::spatial::update_spatial_grid`] so the grid
/// reflects the current frame's enemy positions.
#[allow(clippy::too_many_arguments)]
pub fn fire_garlic(
    mut fired_events: MessageReader<WeaponFiredEvent>,
    mut damage_events: MessageWriter<DamageEnemyEvent>,
    player_q: Query<(&Transform, &PlayerStats), With<Player>>,
    enemy_q: Query<&Transform, With<Enemy>>,
    mut aura_q: Query<(Entity, &GarlicAura, &mut AuraWeapon), Without<Player>>,
    spatial_grid: Res<SpatialGrid>,
    garlic_cfg: GarlicParams,
    mut commands: Commands,
) {
    let cfg = garlic_cfg.get();
    // Track players for which a spawn was already queued this run, so that
    // two same-frame Garlic events cannot produce duplicate aura entities.
    let mut spawn_scheduled_for: HashSet<Entity> = HashSet::new();

    for event in fired_events.read() {
        let is_soul_eater = event.weapon_type == WeaponType::SoulEater;
        if event.weapon_type != WeaponType::Garlic && !is_soul_eater {
            continue;
        }

        let Ok((player_tf, stats)) = player_q.get(event.player) else {
            continue;
        };

        let player_pos = player_tf.translation.truncate();
        let level = event.level.clamp(1, 8) as usize;

        // --- Per-level stats (with config fallback) ---
        let base_damage = cfg
            .and_then(|c| c.damage_by_level.get(level - 1).copied())
            .unwrap_or(DEFAULT_GARLIC_DAMAGE_BY_LEVEL[level - 1]);
        let base_radius = cfg
            .and_then(|c| c.radius_by_level.get(level - 1).copied())
            .unwrap_or(DEFAULT_GARLIC_RADIUS_BY_LEVEL[level - 1]);

        let damage = base_damage * stats.damage_multiplier;
        let radius = base_radius * stats.area_multiplier;

        // --- Spawn or update the persistent GarlicAura entity ---
        // Find any existing aura belonging to this player.
        let existing_entity = aura_q
            .iter()
            .find_map(|(entity, aura, _)| (aura.player == event.player).then_some(entity));

        if let Some(entity) = existing_entity {
            // Update stats on level-up or player-stat change.
            // update_garlic_visual will sync the visual radius automatically
            // via Changed<AuraWeapon>.
            if let Ok((_, _, mut aura_weapon)) = aura_q.get_mut(entity) {
                aura_weapon.damage = damage;
                aura_weapon.radius = radius;
            }
        } else if spawn_scheduled_for.insert(event.player) {
            // First activation (and not already queued this run): spawn the
            // aura entity as a child of the player so its transform tracks the
            // player automatically.  The circle visual is inserted by
            // spawn_garlic_visual on the next frame.
            commands.entity(event.player).with_children(|parent| {
                parent.spawn((
                    GarlicAura {
                        player: event.player,
                    },
                    AuraWeapon {
                        damage,
                        radius,
                        tick_timer: 0.0,
                        tick_interval: 0.0, // scheduling is driven by weapon cooldown
                    },
                    // Scale encodes the aura radius (unit circle scaled to radius).
                    Transform::from_xyz(0.0, 0.0, 3.0).with_scale(Vec3::splat(radius)),
                    GameSessionEntity,
                ));
            });
        }

        // --- Deal damage to all enemies within the aura radius ---
        for enemy_entity in spatial_grid.get_nearby(player_pos, radius) {
            let Ok(enemy_tf) = enemy_q.get(enemy_entity) else {
                continue;
            };
            let dist = (enemy_tf.translation.truncate() - player_pos).length();
            if dist <= radius {
                damage_events.write(DamageEnemyEvent {
                    entity: enemy_entity,
                    damage,
                    weapon_type: event.weapon_type,
                });
            }
        }
    }
}

/// Adds a circle visual to newly spawned [`GarlicAura`] entities.
///
/// Runs whenever `Added<GarlicAura>` is detected (the frame after the
/// entity is flushed from [`fire_garlic`]'s deferred commands).
/// Uses a unit [`Circle`] mesh scaled by [`Transform::scale`] so that
/// [`update_garlic_visual`] can adjust the displayed radius cheaply.
pub fn spawn_garlic_visual(
    mut commands: Commands,
    query: Query<Entity, Added<GarlicAura>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for entity in query.iter() {
        let mesh = meshes.add(Circle::new(1.0));
        let material = materials.add(ColorMaterial::from_color(Color::srgba(0.7, 0.3, 1.0, 0.25)));
        commands
            .entity(entity)
            .insert((Mesh2d(mesh), MeshMaterial2d(material)));
    }
}

/// Keeps the aura circle visual in sync with the current [`AuraWeapon::radius`].
///
/// Runs whenever [`AuraWeapon`] is mutated on a [`GarlicAura`] entity
/// (including when it is first added).  Updates `Transform::scale` so the
/// unit circle mesh reflects the true radius.
#[allow(clippy::type_complexity)]
pub fn update_garlic_visual(
    mut aura_q: Query<(&AuraWeapon, &mut Transform), (With<GarlicAura>, Changed<AuraWeapon>)>,
) {
    for (aura, mut transform) in aura_q.iter_mut() {
        transform.scale = Vec3::splat(aura.radius);
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
        let mut weapon = WeaponState::new(WeaponType::Garlic);
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
            .run_system_once(fire_garlic)
            .expect("fire_garlic should run");
        app.world_mut().flush();
    }

    fn tick_and_fire(app: &mut App) {
        use crate::systems::{
            spatial::update_spatial_grid, weapon_cooldown::tick_weapon_cooldowns,
        };
        advance(app);
        app.world_mut()
            .run_system_once(tick_weapon_cooldowns)
            .expect("tick_weapon_cooldowns should run");
        app.world_mut()
            .run_system_once(update_spatial_grid)
            .expect("update_spatial_grid should run");
        app.world_mut()
            .run_system_once(fire_garlic)
            .expect("fire_garlic should run");
        app.world_mut().flush();
    }

    fn damage_events(app: &App) -> Vec<DamageEnemyEvent> {
        let messages = app.world().resource::<Messages<DamageEnemyEvent>>();
        let mut cursor = messages.get_cursor();
        cursor.read(messages).cloned().collect()
    }

    // -----------------------------------------------------------------------
    // Tests
    // -----------------------------------------------------------------------

    /// Enemy within aura radius receives a DamageEnemyEvent.
    #[test]
    fn garlic_hits_enemy_in_range() {
        let mut app = build_app();
        let player = spawn_player(&mut app);
        spawn_enemy(&mut app, Vec2::new(50.0, 0.0)); // within default Lv1 radius (80 px)

        fire_once(&mut app, player, WeaponType::Garlic, 1);

        let events = damage_events(&app);
        assert_eq!(events.len(), 1, "enemy in range should be hit");
        assert!(events[0].damage > 0.0);
        assert_eq!(events[0].weapon_type, WeaponType::Garlic);
    }

    /// Enemy outside aura radius is not hit.
    #[test]
    fn garlic_does_not_hit_enemy_out_of_range() {
        let mut app = build_app();
        let player = spawn_player(&mut app);
        spawn_enemy(&mut app, Vec2::new(200.0, 0.0)); // beyond 80 px default radius

        fire_once(&mut app, player, WeaponType::Garlic, 1);

        assert_eq!(
            damage_events(&app).len(),
            0,
            "enemy beyond radius should not be hit"
        );
    }

    /// The GarlicAura child entity is spawned on the first fire.
    #[test]
    fn garlic_aura_entity_spawned_on_first_fire() {
        let mut app = build_app();
        spawn_player(&mut app);
        tick_and_fire(&mut app);

        let count = app
            .world_mut()
            .query::<&GarlicAura>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "one GarlicAura entity should exist after firing");
    }

    /// Firing twice does not duplicate the GarlicAura entity.
    #[test]
    fn garlic_aura_not_duplicated_on_second_fire() {
        let mut app = build_app();
        let player = spawn_player(&mut app);

        fire_once(&mut app, player, WeaponType::Garlic, 1);
        fire_once(&mut app, player, WeaponType::Garlic, 1);

        let count = app
            .world_mut()
            .query::<&GarlicAura>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "GarlicAura must not be duplicated on second fire");
    }

    /// Two same-frame Garlic events for the same player do not produce two auras.
    #[test]
    fn garlic_aura_not_duplicated_on_same_frame_events() {
        let mut app = build_app();
        let player = spawn_player(&mut app);

        // Queue two events before running the system.
        app.world_mut().write_message(WeaponFiredEvent {
            player,
            weapon_type: WeaponType::Garlic,
            level: 1,
        });
        app.world_mut().write_message(WeaponFiredEvent {
            player,
            weapon_type: WeaponType::Garlic,
            level: 1,
        });
        app.world_mut()
            .run_system_once(crate::systems::spatial::update_spatial_grid)
            .unwrap();
        app.world_mut()
            .run_system_once(fire_garlic)
            .expect("fire_garlic should run");
        app.world_mut().flush();

        let count = app
            .world_mut()
            .query::<&GarlicAura>()
            .iter(app.world())
            .count();
        assert_eq!(
            count, 1,
            "two same-frame events must not produce two GarlicAura entities"
        );
    }

    /// Level 3 deals more damage per tick than level 1.
    #[test]
    fn garlic_damage_increases_with_level() {
        let lv1 = DEFAULT_GARLIC_DAMAGE_BY_LEVEL[0];
        let lv3 = DEFAULT_GARLIC_DAMAGE_BY_LEVEL[2];
        assert!(lv3 > lv1, "Lv3 damage ({lv3}) should exceed Lv1 ({lv1})");
    }

    /// Level 8 has a larger radius than level 1.
    #[test]
    fn garlic_radius_increases_with_level() {
        let lv1 = DEFAULT_GARLIC_RADIUS_BY_LEVEL[0];
        let lv8 = DEFAULT_GARLIC_RADIUS_BY_LEVEL[7];
        assert!(lv8 > lv1, "Lv8 radius ({lv8}) should exceed Lv1 ({lv1})");
    }

    /// `PlayerStats::area_multiplier` scales the effective aura radius.
    #[test]
    fn garlic_radius_scales_with_area_multiplier() {
        let mut app = build_app();
        let player = spawn_player(&mut app);
        // Double the area → default Lv1 radius 80 px becomes 160 px.
        app.world_mut()
            .get_mut::<PlayerStats>(player)
            .unwrap()
            .area_multiplier = 2.0;

        // Enemy just outside default radius but inside doubled radius.
        spawn_enemy(&mut app, Vec2::new(120.0, 0.0)); // 120 < 160

        fire_once(&mut app, player, WeaponType::Garlic, 1);

        assert_eq!(
            damage_events(&app).len(),
            1,
            "doubled radius should reach the enemy at 120 px"
        );
    }

    /// SoulEater (Garlic evolution) also activates the aura.
    #[test]
    fn soul_eater_fires_garlic_aura() {
        let mut app = build_app();
        let player = spawn_player(&mut app);
        spawn_enemy(&mut app, Vec2::new(50.0, 0.0));

        fire_once(&mut app, player, WeaponType::SoulEater, 1);

        assert_eq!(
            damage_events(&app).len(),
            1,
            "SoulEater should deal aura damage"
        );
    }

    /// Non-Garlic weapon events are ignored by fire_garlic.
    #[test]
    fn other_weapons_do_not_fire_garlic() {
        let mut app = build_app();
        let player = spawn_player(&mut app);
        spawn_enemy(&mut app, Vec2::new(50.0, 0.0));

        fire_once(&mut app, player, WeaponType::MagicWand, 1);

        assert_eq!(
            damage_events(&app).len(),
            0,
            "fire_garlic should ignore non-Garlic events"
        );
    }

    /// Multiple enemies in range all receive damage.
    #[test]
    fn garlic_hits_multiple_enemies_in_range() {
        let mut app = build_app();
        let player = spawn_player(&mut app);
        spawn_enemy(&mut app, Vec2::new(30.0, 0.0));
        spawn_enemy(&mut app, Vec2::new(-40.0, 20.0));
        spawn_enemy(&mut app, Vec2::new(0.0, 60.0));

        fire_once(&mut app, player, WeaponType::Garlic, 1);

        assert_eq!(
            damage_events(&app).len(),
            3,
            "all three enemies in range should be hit"
        );
    }
}
