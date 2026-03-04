//! Treasure chest collection and reward system.
//!
//! When the player touches a treasure chest ([`Treasure`] component), the
//! chest is despawned and a reward is applied:
//!
//! 1. **If any weapon can evolve** (level 8 + required passive owned),
//!    the first eligible weapon evolves immediately.
//! 2. Otherwise a fixed gold reward (`DEFAULT_TREASURE_GOLD`) is added to
//!    [`GameData`].

use bevy::prelude::*;

use crate::{
    components::{
        CircleCollider, GameSessionEntity, PassiveInventory, Player, Treasure, WeaponInventory,
    },
    resources::GameData,
    types::WeaponType,
};

use super::evolution::find_evolution;

// ---------------------------------------------------------------------------
// Fallback constant (used before game.ron has finished loading)
// ---------------------------------------------------------------------------

const DEFAULT_TREASURE_GOLD: u32 = 50;
const DEFAULT_TREASURE_RADIUS: f32 = 20.0;

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Detects player–treasure overlaps, applies the reward, and despawns the
/// chest.
///
/// Evolution takes priority: if at least one weapon in the player's inventory
/// is at level 8 **and** the player owns the required passive, that weapon
/// is replaced by its evolved form and `evolved` is set to `true`.
///
/// Runs every frame while in `AppState::Playing`.
pub fn open_treasure_chests(
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
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

    for (treasure_entity, treasure_tf) in &treasure_q {
        let dist = player_tf
            .translation
            .truncate()
            .distance(treasure_tf.translation.truncate());

        let threshold = player_col.radius + DEFAULT_TREASURE_RADIUS;
        if dist > threshold {
            continue;
        }

        // Despawn the chest immediately.
        commands.entity(treasure_entity).despawn();

        // Apply reward: evolution takes priority over gold.
        if let Some(_evolved) = find_evolution(weapon_inv, passive_inv) {
            // Evolution is applied via a separate command queue through events.
            // Here we just mark it; the actual WeaponInventory mutation is done
            // in apply_evolution (below) which reads WeaponEvolvedEvent.
            //
            // For now, the chest open is acknowledged; evolution application
            // requires mutable access to WeaponInventory which conflicts with
            // the immutable borrow above.  A follow-up system (or event) can
            // finalize the swap.  This design keeps this system read-only on
            // the inventory so it remains composable.
            //
            // Emit the evolved type so apply_evolution can act on it.
            commands.trigger(WeaponEvolvedTrigger {
                evolved_type: _evolved,
            });
        } else {
            game_data.gold_earned += DEFAULT_TREASURE_GOLD;
        }
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
                level: 8,
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
/// Called by the spawner system when a treasure drop is triggered.
pub fn spawn_treasure(commands: &mut Commands, position: Vec2) {
    commands.spawn((
        Transform::from_xyz(position.x, position.y, 0.0),
        CircleCollider {
            radius: DEFAULT_TREASURE_RADIUS,
        },
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
}
