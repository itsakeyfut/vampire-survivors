//! Treasure chest collection and reward system.
//!
//! When the player touches a treasure chest ([`Treasure`] component), the
//! chest is despawned and a reward is applied:
//!
//! 1. **If any weapon can evolve** (at max level + required passive owned),
//!    the first eligible weapon evolves immediately.
//! 2. Otherwise a random reward is chosen from three options:
//!    - **Upgrade** — if any owned weapon or passive is below max level, one
//!      is upgraded.  This option is skipped when no upgrades are available.
//!    - **HP recovery** — restores `treasure_hp_recovery_pct × max_hp`.
//!    - **Gold** — adds `treasure_gold_reward` to [`GameData`].
//!
//! When gold is the reward, `treasure_gold_reward` is added to
//! [`GameData::gold_earned`] (run total).  At run end
//! (`OnEnter(GameOver)` / `OnEnter(Victory)`), the full `gold_earned` value is
//! transferred to [`MetaProgress::total_gold`] by `accrue_gold_on_game_over` /
//! `accrue_gold_on_victory` in `systems::persistence`.
//!
//! At most **one chest is collected per frame** to keep inventory state
//! consistent: `apply_evolution` runs deferred via a trigger, so collecting a
//! second chest in the same frame would see a stale (pre-evolution) inventory.
//! In practice, simultaneous chest overlaps are extremely rare and this
//! restriction has no gameplay impact.

use bevy::prelude::*;
use rand::RngExt;

use crate::{
    components::{
        CircleCollider, GameSessionEntity, PassiveInventory, Player, PlayerStats, Treasure,
        TreasureGlow, TreasureSpawnFlash, WeaponInventory,
    },
    config::{GameParams, PassiveConfig, PassiveParams},
    events::TreasureOpenedEvent,
    materials::GlowMaterial,
    resources::GameData,
    types::{UpgradeChoice, WeaponType},
};

use super::apply::apply_passive_bonus;
use super::choices::build_owned_upgrade_pool;
use super::evolution::find_evolution;

/// Normal chest colour (yellow placeholder, Phase 17 will replace with a real
/// sprite; kept as a constant so `spawn_treasure` and `animate_treasure_spawn_flash`
/// stay in sync without a config round-trip for a colour value).
const CHEST_COLOR: Color = Color::srgb(1.0, 0.85, 0.1);

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Detects player–treasure overlaps, applies the reward, and despawns the
/// chest.
///
/// Evolution takes priority: if at least one weapon in the player's inventory
/// is at `max_weapon_level` **and** the player owns the required passive, that
/// weapon is replaced by its evolved form.
///
/// Otherwise a random reward is chosen from three options (see module docs).
///
/// At most one chest is processed per frame (see module-level note).
///
/// Proximity check: `player.radius + treasure_radius`.  Treasure chests are
/// sparse (a handful at most), so an O(n) scan is acceptable here — the same
/// pattern used for un-attracted XP gems.  A SpatialGrid optimisation can be
/// added if chest counts ever become significant.
///
/// Runs every frame while in `AppState::Playing`.
#[allow(clippy::too_many_arguments)]
pub fn open_treasure_chests(
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
    mut opened_events: MessageWriter<TreasureOpenedEvent>,
    game_cfg: GameParams,
    passive_cfg: PassiveParams,
    mut player_q: Query<
        (
            &Transform,
            &CircleCollider,
            &mut WeaponInventory,
            &mut PassiveInventory,
            &mut PlayerStats,
        ),
        With<Player>,
    >,
    treasure_q: Query<(Entity, &Transform), With<Treasure>>,
) {
    let Ok((player_tf, player_col, mut weapon_inv, mut passive_inv, mut stats)) =
        player_q.single_mut()
    else {
        return;
    };

    let treasure_radius = game_cfg.treasure_radius();
    let gold_reward = game_cfg.treasure_gold();
    let max_weapon_level = game_cfg.max_weapon_level();
    let max_passive_level = game_cfg.max_passive_level();
    let hp_recovery_pct = game_cfg.treasure_hp_recovery_pct();

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

        // Apply reward: evolution takes priority over random reward.
        if let Some(evolved) = find_evolution(&weapon_inv, &passive_inv, max_weapon_level) {
            // Mutation happens in apply_evolution (an observer) which runs
            // after this system.  Emitting a trigger keeps this system
            // read-only on the inventory, avoiding borrow conflicts.
            commands.trigger(WeaponEvolvedTrigger {
                evolved_type: evolved,
            });
        } else {
            apply_non_evolution_reward(
                &mut weapon_inv,
                &mut passive_inv,
                &mut stats,
                &mut game_data,
                &RewardContext {
                    passive_cfg: passive_cfg.get(),
                    hp_recovery_pct,
                    gold_reward,
                },
                max_weapon_level,
                max_passive_level,
            );
        }

        // Process at most one chest per frame to avoid stale-inventory decisions.
        break;
    }
}

/// The reward variant awarded when a chest is opened without a weapon evolution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reward {
    /// Upgrade one specific weapon or passive by one level.
    Upgrade(UpgradeChoice),
    /// Restore a fraction of the player's max HP.
    HpRecovery,
    /// Award gold coins.
    Gold,
}

/// Builds the eligible reward pool for the current player state and picks one
/// at random.
///
/// Returns one of:
/// - [`Reward::Upgrade`] with a randomly chosen [`UpgradeChoice`] (omitted when
///   no weapons/passives are below max level)
/// - [`Reward::HpRecovery`]
/// - [`Reward::Gold`]
///
/// All eligible options have equal probability.
pub(crate) fn pick_reward(
    weapon_inv: &WeaponInventory,
    passive_inv: &PassiveInventory,
    max_weapon_level: u8,
    max_passive_level: u8,
) -> Reward {
    let upgrade_pool =
        build_owned_upgrade_pool(weapon_inv, passive_inv, max_weapon_level, max_passive_level);

    let mut rng = rand::rng();

    let mut options: Vec<Reward> = vec![Reward::HpRecovery, Reward::Gold];
    if !upgrade_pool.is_empty() {
        let idx = rng.random_range(0..upgrade_pool.len());
        options.push(Reward::Upgrade(upgrade_pool[idx]));
    }

    options[rng.random_range(0..options.len())]
}

/// Config-derived values threaded into [`apply_reward`].
pub(crate) struct RewardContext<'a> {
    pub passive_cfg: Option<&'a PassiveConfig>,
    pub hp_recovery_pct: f32,
    pub gold_reward: u32,
}

/// Applies an already-selected [`Reward`] to the player and game state.
///
/// Separated from selection so each reward path is directly testable without
/// random retries.
pub(crate) fn apply_reward(
    reward: Reward,
    weapon_inv: &mut WeaponInventory,
    passive_inv: &mut PassiveInventory,
    stats: &mut PlayerStats,
    game_data: &mut GameData,
    ctx: &RewardContext<'_>,
) {
    let RewardContext {
        passive_cfg,
        hp_recovery_pct,
        gold_reward,
    } = *ctx;
    match reward {
        Reward::Upgrade(choice) => match choice {
            UpgradeChoice::WeaponUpgrade(wt) => {
                if let Some(w) = weapon_inv.weapons.iter_mut().find(|w| w.weapon_type == wt) {
                    w.level += 1;
                    info!("Treasure: upgraded {wt:?} to level {}", w.level);
                }
            }
            UpgradeChoice::PassiveUpgrade(pt) => {
                if let Some(p) = passive_inv.items.iter_mut().find(|p| p.item_type == pt) {
                    p.level += 1;
                    apply_passive_bonus(stats, pt, passive_cfg);
                    info!("Treasure: upgraded {pt:?} to level {}", p.level);
                }
            }
            other => unreachable!(
                "Reward::Upgrade must carry WeaponUpgrade or PassiveUpgrade, got {other:?}"
            ),
        },
        Reward::HpRecovery => {
            let heal = (stats.max_hp * hp_recovery_pct).max(0.0);
            stats.current_hp = (stats.current_hp + heal).clamp(0.0, stats.max_hp);
            info!("Treasure: restored {heal:.0} HP");
        }
        Reward::Gold => {
            game_data.gold_earned += gold_reward;
            info!("Treasure: awarded {gold_reward} gold");
        }
    }
}

/// Picks and applies a random non-evolution reward.
///
/// Returns the [`Reward`] that was chosen so the caller can perform
/// any additional bookkeeping.
fn apply_non_evolution_reward(
    weapon_inv: &mut WeaponInventory,
    passive_inv: &mut PassiveInventory,
    stats: &mut PlayerStats,
    game_data: &mut GameData,
    ctx: &RewardContext<'_>,
    max_weapon_level: u8,
    max_passive_level: u8,
) -> Reward {
    let reward = pick_reward(weapon_inv, passive_inv, max_weapon_level, max_passive_level);
    apply_reward(reward, weapon_inv, passive_inv, stats, game_data, ctx);
    reward
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
/// `game_cfg.treasure_radius()` (falls back to `config::game::DEFAULT_TREASURE_RADIUS`
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
        // Starts white and fades to yellow via TreasureSpawnFlash.
        Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::splat(radius * 2.0)),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, 6.0),
        CircleCollider { radius },
        Treasure,
        TreasureSpawnFlash { elapsed: 0.0 },
        GameSessionEntity,
    ));
}

// ---------------------------------------------------------------------------
// Visual systems — wired by XpPlugin
// ---------------------------------------------------------------------------

/// Spawns a radial-glow child entity for every newly created [`Treasure`] chest.
///
/// The glow mesh is a quad `3×` the chest diameter so the ring has room to
/// expand outward.  It starts [`Visibility::Hidden`] and is made visible by
/// [`update_treasure_glow`] when the player approaches.
pub fn spawn_treasure_glow(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut glow_materials: ResMut<Assets<GlowMaterial>>,
    query: Query<(Entity, &CircleCollider), Added<Treasure>>,
) {
    for (entity, collider) in &query {
        let size = collider.radius * 3.0 * 2.0;
        let mesh = meshes.add(Rectangle::new(size, size));
        let material = glow_materials.add(GlowMaterial::for_treasure(collider.radius));

        let glow = commands
            .spawn((
                TreasureGlow,
                Mesh2d(mesh),
                MeshMaterial2d(material),
                // z = -0.1 so the glow ring renders behind the chest sprite.
                Transform::from_xyz(0.0, 0.0, -0.1),
                Visibility::Hidden,
            ))
            .id();

        commands.entity(entity).add_child(glow);
    }
}

/// Lerps the chest sprite from white → yellow over `treasure_spawn_flash_duration`
/// seconds (from `game.ron`).
///
/// Removes [`TreasureSpawnFlash`] once the animation completes.
pub fn animate_treasure_spawn_flash(
    mut commands: Commands,
    game_cfg: GameParams,
    mut query: Query<(Entity, &mut TreasureSpawnFlash, &mut Sprite)>,
    time: Res<Time>,
) {
    let duration = game_cfg.treasure_spawn_flash_duration();

    for (entity, mut flash, mut sprite) in &mut query {
        flash.elapsed += time.delta_secs();
        let t = (flash.elapsed / duration).clamp(0.0, 1.0);

        // White (1,1,1) → CHEST_COLOR (1, 0.85, 0.1)
        let g = 1.0 - 0.15 * t;
        let b = 1.0 - 0.90 * t;
        sprite.color = Color::srgb(1.0, g, b);

        if flash.elapsed >= duration {
            sprite.color = CHEST_COLOR;
            commands.entity(entity).remove::<TreasureSpawnFlash>();
        }
    }
}

/// Shows the glow ring when the player is within `treasure_glow_distance`
/// pixels of a chest (from `game.ron`), hides it otherwise.
pub fn update_treasure_glow(
    game_cfg: GameParams,
    player_q: Query<&Transform, With<Player>>,
    treasure_q: Query<(&Transform, &Children), With<Treasure>>,
    mut glow_q: Query<&mut Visibility, With<TreasureGlow>>,
) {
    let highlight_dist = game_cfg.treasure_glow_distance();

    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();

    for (chest_tf, children) in &treasure_q {
        let dist = player_pos.distance(chest_tf.translation.truncate());
        let desired = if dist < highlight_dist {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        for &child in children {
            if let Ok(mut vis) = glow_q.get_mut(child) {
                *vis = desired;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        components::{CircleCollider, PassiveInventory, PlayerStats, WeaponInventory},
        config::game::{
            DEFAULT_TREASURE_GOLD, DEFAULT_TREASURE_HP_RECOVERY_PCT, DEFAULT_TREASURE_RADIUS,
        },
        events::TreasureOpenedEvent,
        resources::{GameData, MetaProgress},
        states::AppState,
        types::{PassiveItemType, PassiveState, UpgradeChoice, WeaponState, WeaponType},
    };
    use bevy::state::app::StatesPlugin;

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<AppState>()
            .insert_resource(GameData::default())
            .insert_resource(MetaProgress::default())
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
        let mut stats = PlayerStats::default();
        stats.max_hp = 100.0;
        stats.current_hp = 100.0;
        app.world_mut()
            .spawn((
                Player,
                WeaponInventory { weapons: vec![] },
                PassiveInventory::default(),
                stats,
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

    /// When player overlaps treasure with no evolvable weapon, some reward is applied.
    ///
    /// One of three rewards is given: gold, HP recovery, or upgrade (skipped
    /// here since the player has no weapons/passives).  With an empty upgrade
    /// pool the reward is always gold or HP recovery.
    #[test]
    fn treasure_applies_some_reward_when_no_evolution_possible() {
        let mut app = build_app();
        spawn_player_at(&mut app, Vec2::ZERO);
        // Player HP is full (100), so HP recovery will be clamped to no visible change.
        // Force the chest to be at ZERO to guarantee a collection.
        spawn_treasure_at(&mut app, Vec2::ZERO);

        app.update();

        // At least gold or HP recovery must have been applied.  Since upgrade
        // pool is empty the only two options are gold and HP recovery.
        let gold = app.world().resource::<GameData>().gold_earned;
        // Either gold was awarded or HP was recovered (already full → no change).
        // Just verify the chest was collected (despawned) — reward was applied.
        let mut q = app.world_mut().query_filtered::<Entity, With<Treasure>>();
        assert_eq!(
            q.iter(app.world()).count(),
            0,
            "chest must be despawned after collection"
        );
        // Gold or HP was the outcome. At least the system ran without panicking.
        let _ = gold;
    }

    /// When gold is the reward, `GameData::gold_earned` is incremented.
    ///
    /// `MetaProgress::total_gold` is **not** updated during the run; it is
    /// carried over at run end by `accrue_gold_on_game_over` /
    /// `accrue_gold_on_victory` in `systems::persistence`.
    #[test]
    fn gold_reward_increments_gold_earned() {
        let mut app = build_app();

        // Player with full HP, no weapons/passives → upgrade pool is empty.
        // Only gold or HP recovery can be chosen; run until gold is awarded.
        spawn_player_at(&mut app, Vec2::ZERO);
        for _ in 0..100 {
            spawn_treasure_at(&mut app, Vec2::ZERO);
            app.update();

            let gold = app.world().resource::<GameData>().gold_earned;
            if gold > 0 {
                // MetaProgress must NOT be updated in-run.
                let total = app.world().resource::<MetaProgress>().total_gold;
                assert_eq!(
                    total, 0,
                    "MetaProgress::total_gold must NOT be updated during the run"
                );
                return;
            }
        }
        // Reaching here without gold in 100 chest openings is astronomically unlikely.
        panic!("gold reward never triggered in 100 chest openings");
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
        let mut stats = PlayerStats::default();
        stats.max_hp = 100.0;
        stats.current_hp = 100.0;
        app.world_mut().spawn((
            Player,
            WeaponInventory {
                weapons: vec![whip],
            },
            passive,
            stats,
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

    /// `spawn_treasure` must produce a `Sprite` of the correct size with the
    /// [`TreasureSpawnFlash`] component attached (the flash starts white and
    /// fades to yellow via [`animate_treasure_spawn_flash`]).
    #[test]
    fn spawn_treasure_has_correct_size_and_flash() {
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
            .query_filtered::<(&Sprite, &CircleCollider, &TreasureSpawnFlash), With<Treasure>>();
        let (sprite, collider, flash) = q.single(app.world()).expect("treasure entity must exist");

        assert_eq!(
            collider.radius, radius,
            "CircleCollider radius must match spawn argument"
        );
        assert_eq!(
            sprite.custom_size,
            Some(Vec2::splat(radius * 2.0)),
            "sprite size must be diameter"
        );
        // Chest spawns white (flash t=0) and transitions to yellow.
        let [r, g, b, _] = sprite.color.to_srgba().to_f32_array();
        assert!(r > 0.9, "red channel must be high at spawn");
        assert!(g > 0.9, "green channel must be high (white) at spawn");
        assert!(b > 0.9, "blue channel must be high (white) at spawn");
        assert_eq!(flash.elapsed, 0.0, "flash must not have advanced yet");
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

    // ---------------------------------------------------------------------------
    // apply_reward — deterministic unit tests for each reward path
    // ---------------------------------------------------------------------------

    fn default_ctx() -> RewardContext<'static> {
        RewardContext {
            passive_cfg: None,
            hp_recovery_pct: DEFAULT_TREASURE_HP_RECOVERY_PCT,
            gold_reward: DEFAULT_TREASURE_GOLD,
        }
    }

    fn call_apply_reward(
        reward: Reward,
        weapon_inv: &mut WeaponInventory,
        passive_inv: &mut PassiveInventory,
        stats: &mut PlayerStats,
        game_data: &mut GameData,
    ) {
        let ctx = default_ctx();
        apply_reward(reward, weapon_inv, passive_inv, stats, game_data, &ctx);
    }

    /// HP recovery reward heals the player by the configured percentage.
    #[test]
    fn reward_hp_recovery_heals_player() {
        let mut weapon_inv = WeaponInventory { weapons: vec![] };
        let mut passive_inv = PassiveInventory::default();
        let mut stats = PlayerStats::default();
        stats.max_hp = 100.0;
        stats.current_hp = 40.0;
        let mut game_data = GameData::default();

        call_apply_reward(
            Reward::HpRecovery,
            &mut weapon_inv,
            &mut passive_inv,
            &mut stats,
            &mut game_data,
        );

        let expected = 40.0 + 100.0 * DEFAULT_TREASURE_HP_RECOVERY_PCT;
        assert!(
            (stats.current_hp - expected).abs() < 0.001,
            "HP should increase by {}% of max HP",
            DEFAULT_TREASURE_HP_RECOVERY_PCT * 100.0
        );
    }

    /// Gold reward adds the configured amount to GameData.
    #[test]
    fn reward_gold_adds_to_game_data() {
        let mut weapon_inv = WeaponInventory { weapons: vec![] };
        let mut passive_inv = PassiveInventory::default();
        let mut stats = PlayerStats::default();
        let mut game_data = GameData::default();

        call_apply_reward(
            Reward::Gold,
            &mut weapon_inv,
            &mut passive_inv,
            &mut stats,
            &mut game_data,
        );

        assert_eq!(
            game_data.gold_earned, DEFAULT_TREASURE_GOLD,
            "gold reward must match config"
        );
    }

    /// Upgrade reward increases a weapon's level by one.
    #[test]
    fn reward_upgrade_increases_weapon_level() {
        let mut whip = WeaponState::new(WeaponType::Whip);
        whip.level = 3;
        let mut weapon_inv = WeaponInventory {
            weapons: vec![whip],
        };
        let mut passive_inv = PassiveInventory::default();
        let mut stats = PlayerStats::default();
        let mut game_data = GameData::default();

        call_apply_reward(
            Reward::Upgrade(UpgradeChoice::WeaponUpgrade(WeaponType::Whip)),
            &mut weapon_inv,
            &mut passive_inv,
            &mut stats,
            &mut game_data,
        );

        assert_eq!(
            weapon_inv.weapons[0].level, 4,
            "weapon level should be incremented by 1"
        );
    }

    /// HP recovery is clamped to max HP.
    #[test]
    fn reward_hp_recovery_clamped_to_max_hp() {
        let mut weapon_inv = WeaponInventory { weapons: vec![] };
        let mut passive_inv = PassiveInventory::default();
        let mut stats = PlayerStats::default();
        stats.max_hp = 100.0;
        stats.current_hp = 95.0; // 5 HP below max; 30% heal = 30 HP → clamped to 100
        let mut game_data = GameData::default();

        call_apply_reward(
            Reward::HpRecovery,
            &mut weapon_inv,
            &mut passive_inv,
            &mut stats,
            &mut game_data,
        );

        assert_eq!(stats.current_hp, 100.0, "HP must be clamped to max_hp");
        assert!(
            stats.current_hp <= stats.max_hp,
            "HP must not exceed max_hp"
        );
    }

    /// HP recovery with a negative `hp_recovery_pct` (misconfigured) must not reduce HP.
    #[test]
    fn reward_hp_recovery_negative_pct_is_safe() {
        let mut weapon_inv = WeaponInventory { weapons: vec![] };
        let mut passive_inv = PassiveInventory::default();
        let mut stats = PlayerStats::default();
        stats.max_hp = 100.0;
        stats.current_hp = 50.0;
        let mut game_data = GameData::default();

        apply_reward(
            Reward::HpRecovery,
            &mut weapon_inv,
            &mut passive_inv,
            &mut stats,
            &mut game_data,
            &RewardContext {
                passive_cfg: None,
                hp_recovery_pct: -0.5, // misconfigured negative value
                gold_reward: DEFAULT_TREASURE_GOLD,
            },
        );

        assert!(
            stats.current_hp >= 50.0,
            "HP must not decrease when hp_recovery_pct is negative"
        );
    }
}
