//! Treasure chest collection and reward system.
//!
//! When the player touches a treasure chest ([`Treasure`] component), the
//! chest is despawned and a reward is applied:
//!
//! 1. **If any weapon can evolve** (at max level + required passive owned),
//!    the first eligible weapon evolves immediately.
//! 2. Otherwise the gold reward from config (`GameConfig::treasure_gold_reward`)
//!    is added to [`GameData`].
//!
//! At most **one chest is collected per frame** to keep inventory state
//! consistent: `apply_evolution` runs deferred via a trigger, so collecting a
//! second chest in the same frame would see a stale (pre-evolution) inventory.
//! In practice, simultaneous chest overlaps are extremely rare and this
//! restriction has no gameplay impact.

use bevy::prelude::*;

use crate::{
    components::{
        CircleCollider, GameSessionEntity, PassiveInventory, Player, Treasure, WeaponInventory,
    },
    config::GameParams,
    events::TreasureOpenedEvent,
    resources::GameData,
    types::WeaponType,
};

use super::evolution::find_evolution;

// ---------------------------------------------------------------------------
// Fallback constants (used before game.ron has finished loading)
// ---------------------------------------------------------------------------

const DEFAULT_TREASURE_GOLD: u32 = 50;
const DEFAULT_TREASURE_RADIUS: f32 = 20.0;
const DEFAULT_MAX_WEAPON_LEVEL: u8 = 8;

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Detects player–treasure overlaps, applies the reward, and despawns the
/// chest.
///
/// Evolution takes priority: if at least one weapon in the player's inventory
/// is at `max_weapon_level` **and** the player owns the required passive, that
/// weapon is replaced by its evolved form and `evolved` is set to `true`.
///
/// At most one chest is processed per frame (see module-level note).
///
/// Proximity check: `player.radius + treasure_radius`.  Treasure chests are
/// sparse (a handful at most), so an O(n) scan is acceptable here — the same
/// pattern used for un-attracted XP gems.  A SpatialGrid optimisation can be
/// added if chest counts ever become significant.
///
/// Runs every frame while in `AppState::Playing`.
pub fn open_treasure_chests(
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
    mut opened_events: MessageWriter<TreasureOpenedEvent>,
    game_cfg: GameParams,
    player_q: Query<
        (
            &Transform,
            &CircleCollider,
            &WeaponInventory,
            &PassiveInventory,
        ),
        With<Player>,
    >,
    treasure_q: Query<(Entity, &Transform), With<Treasure>>,
) {
    let Ok((player_tf, player_col, weapon_inv, passive_inv)) = player_q.single() else {
        return;
    };

    let cfg = game_cfg.get();
    let treasure_radius = cfg.map_or(DEFAULT_TREASURE_RADIUS, |c| c.treasure_radius);
    let gold_reward = cfg.map_or(DEFAULT_TREASURE_GOLD, |c| c.treasure_gold_reward);
    let max_weapon_level = cfg.map_or(DEFAULT_MAX_WEAPON_LEVEL, |c| c.max_weapon_level);

    for (treasure_entity, treasure_tf) in &treasure_q {
        let dist = player_tf
            .translation
            .truncate()
            .distance(treasure_tf.translation.truncate());

        let threshold = player_col.radius + treasure_radius;
        if dist > threshold {
            continue;
        }

        let chest_pos = treasure_tf.translation.truncate();

        // Notify listeners (audio, HUD, etc.) before the entity is gone.
        opened_events.write(TreasureOpenedEvent {
            position: chest_pos,
        });

        // Despawn the chest immediately.
        commands.entity(treasure_entity).despawn();

        // Apply reward: evolution takes priority over gold.
        if let Some(evolved) = find_evolution(weapon_inv, passive_inv, max_weapon_level) {
            // Mutation happens in apply_evolution (an observer) which runs
            // after this system.  Emitting a trigger keeps this system
            // read-only on the inventory, avoiding borrow conflicts.
            commands.trigger(WeaponEvolvedTrigger {
                evolved_type: evolved,
            });
        } else {
            game_data.gold_earned += gold_reward;
        }

        // Process at most one chest per frame to avoid stale-inventory decisions.
        break;
    }
}

/// Observer trigger fired when a treasure chest evolution is detected.
///
/// [`apply_evolution`] reacts to this trigger and mutates the
/// [`WeaponInventory`] to replace the base weapon with its evolved form.
#[derive(Event, Debug)]
pub struct WeaponEvolvedTrigger {
    pub evolved_type: WeaponType,
}

/// Reacts to [`WeaponEvolvedTrigger`] and replaces the base weapon entry in
/// [`WeaponInventory`] with the evolved form.
///
/// Registered as a global observer on `App`.
pub fn apply_evolution(
    trigger: On<WeaponEvolvedTrigger>,
    mut player_q: Query<(&mut WeaponInventory, &PassiveInventory), With<Player>>,
) {
    use super::evolution::{get_evolution_requirement, get_evolved_weapon};
    use crate::types::WeaponState;

    let evolved_type = trigger.event().evolved_type;
    let Ok((mut weapon_inv, passive_inv)) = player_q.single_mut() else {
        return;
    };

    // Find the base weapon that produces this evolved form.
    for ws in weapon_inv.weapons.iter_mut() {
        if ws.evolved {
            continue;
        }
        let Some(required) = get_evolution_requirement(ws.weapon_type) else {
            continue;
        };
        let has_passive = passive_inv.items.iter().any(|p| p.item_type == required);
        if has_passive && get_evolved_weapon(ws.weapon_type) == evolved_type {
            *ws = WeaponState {
                weapon_type: evolved_type,
                level: ws.level, // preserve the earned level (always max for evolution)
                cooldown_timer: 0.0,
                evolved: true,
            };
            return;
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin helper — wired by XpPlugin
// ---------------------------------------------------------------------------

/// Spawns a treasure chest entity at the given world position.
///
/// `radius` is the chest's collision radius in pixels; callers should pass
/// `GameConfig::treasure_radius` (falling back to `DEFAULT_TREASURE_RADIUS`
/// when config is not yet loaded).
///
/// The chest uses a yellow square placeholder sprite (replace with pixel-art
/// chest sprite in Phase 17).  z = 6.0 places it above enemies (z = 5.0) so
/// it is always visible.
///
/// Called by the spawner system when a treasure drop is triggered.
pub fn spawn_treasure(commands: &mut Commands, position: Vec2, radius: f32) {
    commands.spawn((
        // Yellow square placeholder; Phase 17 will replace with a real sprite.
        Sprite {
            color: Color::srgb(1.0, 0.85, 0.1),
            custom_size: Some(Vec2::splat(radius * 2.0)),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, 6.0),
        CircleCollider { radius },
        Treasure,
        GameSessionEntity,
    ));
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        components::{CircleCollider, PassiveInventory, WeaponInventory},
        events::TreasureOpenedEvent,
        resources::GameData,
        states::AppState,
        types::{PassiveItemType, PassiveState, WeaponState, WeaponType},
    };
    use bevy::prelude::*;
    use bevy::state::app::StatesPlugin;

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<AppState>()
            .insert_resource(GameData::default())
            .add_message::<TreasureOpenedEvent>()
            .add_observer(apply_evolution)
            .add_systems(
                Update,
                open_treasure_chests.run_if(in_state(AppState::Playing)),
            );

        // Transition into Playing state.
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Playing);
        app.update();

        app
    }

    fn spawn_player_at(app: &mut App, pos: Vec2) -> Entity {
        app.world_mut()
            .spawn((
                Player,
                WeaponInventory { weapons: vec![] },
                PassiveInventory::default(),
                Transform::from_xyz(pos.x, pos.y, 0.0),
                CircleCollider {
                    radius: DEFAULT_TREASURE_RADIUS, // same size is fine for tests
                },
            ))
            .id()
    }

    fn spawn_treasure_at(app: &mut App, pos: Vec2) -> Entity {
        app.world_mut()
            .spawn((Treasure, Transform::from_xyz(pos.x, pos.y, 0.0)))
            .id()
    }

    /// When player is out of range, the treasure should not be collected.
    #[test]
    fn treasure_not_collected_when_out_of_range() {
        let mut app = build_app();
        spawn_player_at(&mut app, Vec2::ZERO);
        let t = spawn_treasure_at(&mut app, Vec2::new(500.0, 0.0));

        app.update();

        // Treasure entity still exists.
        assert!(app.world().get_entity(t).is_ok(), "chest should survive");
    }

    /// When player overlaps treasure with no evolvable weapon, gold is awarded.
    #[test]
    fn treasure_awards_gold_when_no_evolution_possible() {
        let mut app = build_app();
        spawn_player_at(&mut app, Vec2::ZERO);
        spawn_treasure_at(&mut app, Vec2::ZERO); // overlapping

        app.update();

        let gold = app.world().resource::<GameData>().gold_earned;
        assert_eq!(gold, DEFAULT_TREASURE_GOLD, "gold should be awarded");
    }

    /// When an evolvable weapon is present, the evolution triggers and no gold
    /// is awarded.
    #[test]
    fn treasure_triggers_evolution_when_weapon_qualifies() {
        let mut app = build_app();

        // Spawn player with Whip at level 8 + HollowHeart passive.
        let mut whip = WeaponState::new(WeaponType::Whip);
        whip.level = 8;
        let passive = PassiveInventory {
            items: vec![PassiveState {
                item_type: PassiveItemType::HollowHeart,
                level: 1,
            }],
        };
        app.world_mut().spawn((
            Player,
            WeaponInventory {
                weapons: vec![whip],
            },
            passive,
            Transform::from_xyz(0.0, 0.0, 0.0),
            CircleCollider {
                radius: DEFAULT_TREASURE_RADIUS,
            },
        ));
        spawn_treasure_at(&mut app, Vec2::ZERO);

        app.update();

        // No gold awarded.
        let gold = app.world().resource::<GameData>().gold_earned;
        assert_eq!(gold, 0, "no gold should be awarded when evolving");

        // Weapon should now be BloodyTear.
        let mut q = app
            .world_mut()
            .query_filtered::<&WeaponInventory, With<Player>>();
        let inv = q.single(app.world()).expect("player should exist");
        assert_eq!(
            inv.weapons[0].weapon_type,
            WeaponType::BloodyTear,
            "Whip should have evolved to BloodyTear"
        );
        assert!(inv.weapons[0].evolved, "evolved flag should be set");
    }

    /// Opening a chest emits exactly one [`TreasureOpenedEvent`] at the chest
    /// position.
    #[test]
    fn treasure_emits_opened_event_on_collection() {
        let mut app = build_app();
        spawn_player_at(&mut app, Vec2::ZERO);
        let chest_pos = Vec2::new(5.0, 5.0); // inside overlap range
        spawn_treasure_at(&mut app, chest_pos);

        app.update();

        let messages = app.world().resource::<Messages<TreasureOpenedEvent>>();
        let events: Vec<_> = messages.get_cursor().read(messages).cloned().collect();
        assert_eq!(
            events.len(),
            1,
            "exactly one TreasureOpenedEvent should fire"
        );
        assert!(
            (events[0].position - chest_pos).length() < 0.001,
            "event position must match chest position"
        );
    }

    /// When player is out of range, no [`TreasureOpenedEvent`] is emitted.
    #[test]
    fn no_event_when_treasure_out_of_range() {
        let mut app = build_app();
        spawn_player_at(&mut app, Vec2::ZERO);
        spawn_treasure_at(&mut app, Vec2::new(500.0, 0.0));

        app.update();

        let messages = app.world().resource::<Messages<TreasureOpenedEvent>>();
        assert_eq!(
            messages.get_cursor().read(messages).count(),
            0,
            "no TreasureOpenedEvent when chest is out of range"
        );
    }

    /// `spawn_treasure` must produce a yellow `Sprite` of the correct size.
    #[test]
    fn spawn_treasure_has_yellow_sprite() {
        use bevy::ecs::system::RunSystemOnce as _;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let radius = DEFAULT_TREASURE_RADIUS;
        app.world_mut()
            .run_system_once(move |mut commands: Commands| {
                spawn_treasure(&mut commands, Vec2::ZERO, radius);
            })
            .unwrap();

        app.update();

        let mut q = app
            .world_mut()
            .query_filtered::<(&Sprite, &CircleCollider), With<Treasure>>();
        let (sprite, collider) = q.single(app.world()).expect("treasure entity must exist");

        assert_eq!(
            collider.radius, radius,
            "CircleCollider radius must match spawn argument"
        );
        assert_eq!(
            sprite.custom_size,
            Some(Vec2::splat(radius * 2.0)),
            "sprite size must be diameter"
        );
        // Yellow: R ≈ 1.0, G ≈ 0.85, B ≈ 0.1.
        let [r, g, b, _] = sprite.color.to_srgba().to_f32_array();
        assert!(r > 0.9, "red channel must be high for yellow sprite");
        assert!(g > 0.7, "green channel must be moderate for yellow sprite");
        assert!(b < 0.3, "blue channel must be low for yellow sprite");
    }

    /// `spawn_treasure` z-coordinate must be above enemies (z > 5.0).
    #[test]
    fn spawn_treasure_z_above_enemies() {
        use bevy::ecs::system::RunSystemOnce as _;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        app.world_mut()
            .run_system_once(|mut commands: Commands| {
                spawn_treasure(
                    &mut commands,
                    Vec2::new(10.0, 20.0),
                    DEFAULT_TREASURE_RADIUS,
                );
            })
            .unwrap();

        app.update();

        let mut q = app
            .world_mut()
            .query_filtered::<&Transform, With<Treasure>>();
        let tf = q.single(app.world()).expect("treasure entity must exist");

        assert!(
            tf.translation.z > 5.0,
            "treasure z ({}) must be above enemies (z = 5.0)",
            tf.translation.z
        );
        assert_eq!(tf.translation.x, 10.0, "x position must match");
        assert_eq!(tf.translation.y, 20.0, "y position must match");
    }
}
