//! Upgrade application system.
//!
//! [`apply_selected_upgrade`] runs on [`OnEnter(AppState::Playing)`].
//! When the player returns from the level-up card screen, this system:
//!
//! 1. Reads [`PendingUpgradeIndex`] to find which card was chosen.
//! 2. Looks up the corresponding [`UpgradeChoice`] in [`LevelUpChoices`].
//! 3. Applies the upgrade to the player's [`WeaponInventory`],
//!    [`PassiveInventory`], and [`PlayerStats`].
//! 4. Clears [`PendingUpgradeIndex`] so the system is a no-op on normal
//!    game-start re-entries (Title → Playing).
//!
//! ## Passive stat bonuses (applied per upgrade level)
//!
//! | Passive      | Stat modified           | Delta per level       |
//! |--------------|-------------------------|-----------------------|
//! | Spinach      | `damage_multiplier`     | +0.10                 |
//! | Wings        | `move_speed`            | +20 px/s (10 % base)  |
//! | HollowHeart  | `max_hp` / `current_hp` | +20 HP (20 % base)    |
//! | Clover       | `luck`                  | +0.10                 |
//! | EmptyTome    | `cooldown_reduction`    | +0.08                 |
//! | Bracer       | `projectile_speed_mult` | +0.10                 |
//! | Spellbinder  | `duration_multiplier`   | +0.10                 |
//! | Duplicator   | `extra_projectiles`     | +1                    |
//! | Pummarola    | `hp_regen`              | +0.5 HP/s             |

use bevy::prelude::*;

use crate::{
    components::{PassiveInventory, Player, PlayerStats, WeaponInventory},
    resources::{LevelUpChoices, PendingUpgradeIndex},
    types::{PassiveItemType, PassiveState, UpgradeChoice, WeaponState},
};

// ---------------------------------------------------------------------------
// Passive bonus constants (per level)
// ---------------------------------------------------------------------------

/// Move-speed bonus per Wings level (pixels/second).
/// Equals 10 % of the base move speed (200 px/s).
const WINGS_SPEED_PER_LEVEL: f32 = 20.0;

/// Max-HP bonus per HollowHeart level (absolute HP).
/// Equals 20 % of the base max HP (100 HP).
const HOLLOW_HEART_HP_PER_LEVEL: f32 = 20.0;

/// Damage multiplier bonus per Spinach level.
const SPINACH_DAMAGE_PER_LEVEL: f32 = 0.10;

/// Luck multiplier bonus per Clover level.
const CLOVER_LUCK_PER_LEVEL: f32 = 0.10;

/// Cooldown reduction bonus per EmptyTome level.
const EMPTY_TOME_CDR_PER_LEVEL: f32 = 0.08;

/// Projectile speed multiplier bonus per Bracer level.
const BRACER_PROJ_SPEED_PER_LEVEL: f32 = 0.10;

/// Duration multiplier bonus per Spellbinder level.
const SPELLBINDER_DURATION_PER_LEVEL: f32 = 0.10;

/// HP regeneration bonus per Pummarola level (HP/s).
const PUMMAROLA_REGEN_PER_LEVEL: f32 = 0.5;

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Applies the upgrade chosen by the player on the level-up card screen.
///
/// Runs on [`OnEnter(AppState::Playing)`].  When
/// [`PendingUpgradeIndex`] is `None` (e.g. normal game start), the system
/// returns immediately without modifying anything.
///
/// After applying the upgrade, [`PendingUpgradeIndex`] is cleared so
/// subsequent re-entries into `Playing` are no-ops.
pub fn apply_selected_upgrade(
    mut pending: ResMut<PendingUpgradeIndex>,
    choices: Res<LevelUpChoices>,
    mut player_q: Query<
        (
            &mut WeaponInventory,
            &mut PassiveInventory,
            &mut PlayerStats,
        ),
        With<Player>,
    >,
) {
    let Some(index) = pending.0.take() else {
        return;
    };

    let Some(choice) = choices.choices.get(index).cloned() else {
        warn!(
            "PendingUpgradeIndex {index} out of bounds (choices len={})",
            choices.choices.len()
        );
        return;
    };

    let Ok((mut weapon_inv, mut passive_inv, mut stats)) = player_q.single_mut() else {
        warn!("apply_selected_upgrade: no player entity found");
        return;
    };

    match choice {
        UpgradeChoice::NewWeapon(weapon_type) => {
            weapon_inv.weapons.push(WeaponState::new(weapon_type));
            info!("Acquired new weapon: {weapon_type:?}");
        }
        UpgradeChoice::WeaponUpgrade(weapon_type) => {
            if let Some(w) = weapon_inv
                .weapons
                .iter_mut()
                .find(|w| w.weapon_type == weapon_type)
            {
                w.level = (w.level + 1).min(8);
                info!("Upgraded {weapon_type:?} to level {}", w.level);
            } else {
                warn!("WeaponUpgrade for {weapon_type:?} but weapon not in inventory");
            }
        }
        UpgradeChoice::PassiveItem(passive_type) => {
            passive_inv.items.push(PassiveState {
                item_type: passive_type,
                level: 1,
            });
            apply_passive_bonus(&mut stats, passive_type);
            info!("Acquired new passive: {passive_type:?}");
        }
        UpgradeChoice::PassiveUpgrade(passive_type) => {
            if let Some(p) = passive_inv
                .items
                .iter_mut()
                .find(|p| p.item_type == passive_type)
            {
                p.level = (p.level + 1).min(5);
                apply_passive_bonus(&mut stats, passive_type);
                info!("Upgraded {passive_type:?} to level {}", p.level);
            } else {
                warn!("PassiveUpgrade for {passive_type:?} but passive not in inventory");
            }
        }
    }
}

/// Applies one level's worth of the stat bonus for `passive_type` to `stats`.
///
/// Called both when a passive is first acquired (level 1) and when an existing
/// passive is upgraded (level N → N+1), so the delta is always the per-level
/// bonus amount.
fn apply_passive_bonus(stats: &mut PlayerStats, passive_type: PassiveItemType) {
    match passive_type {
        PassiveItemType::Spinach => {
            stats.damage_multiplier += SPINACH_DAMAGE_PER_LEVEL;
        }
        PassiveItemType::Wings => {
            stats.move_speed += WINGS_SPEED_PER_LEVEL;
        }
        PassiveItemType::HollowHeart => {
            stats.max_hp += HOLLOW_HEART_HP_PER_LEVEL;
            stats.current_hp += HOLLOW_HEART_HP_PER_LEVEL;
        }
        PassiveItemType::Clover => {
            stats.luck += CLOVER_LUCK_PER_LEVEL;
        }
        PassiveItemType::EmptyTome => {
            stats.cooldown_reduction =
                (stats.cooldown_reduction + EMPTY_TOME_CDR_PER_LEVEL).min(0.9);
        }
        PassiveItemType::Bracer => {
            stats.projectile_speed_mult += BRACER_PROJ_SPEED_PER_LEVEL;
        }
        PassiveItemType::Spellbinder => {
            stats.duration_multiplier += SPELLBINDER_DURATION_PER_LEVEL;
        }
        PassiveItemType::Duplicator => {
            stats.extra_projectiles += 1;
        }
        PassiveItemType::Pummarola => {
            stats.hp_regen += PUMMAROLA_REGEN_PER_LEVEL;
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce as _;
    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::{
        resources::{LevelUpChoices, PendingUpgradeIndex},
        states::AppState,
        types::{PassiveState, WeaponState, WeaponType},
    };

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<AppState>();
        app.insert_resource(PendingUpgradeIndex::default());
        app.insert_resource(LevelUpChoices::default());
        app
    }

    fn spawn_player(app: &mut App) -> Entity {
        app.world_mut()
            .spawn((
                Player,
                WeaponInventory {
                    weapons: vec![WeaponState::new(WeaponType::Whip)],
                },
                PassiveInventory::default(),
                PlayerStats::default(),
            ))
            .id()
    }

    fn run(app: &mut App) {
        app.world_mut()
            .run_system_once(apply_selected_upgrade)
            .unwrap();
    }

    // --- No-op when pending is None ---

    #[test]
    fn no_pending_upgrade_is_noop() {
        let mut app = build_app();
        let entity = spawn_player(&mut app);

        run(&mut app);

        let inv = app.world().get::<WeaponInventory>(entity).unwrap();
        assert_eq!(inv.weapons.len(), 1, "inventory must not change");
    }

    // --- NewWeapon ---

    #[test]
    fn new_weapon_is_added_to_inventory() {
        let mut app = build_app();
        let entity = spawn_player(&mut app);
        app.world_mut()
            .resource_mut::<LevelUpChoices>()
            .choices
            .push(UpgradeChoice::NewWeapon(WeaponType::MagicWand));
        app.world_mut().resource_mut::<PendingUpgradeIndex>().0 = Some(0);

        run(&mut app);

        let inv = app.world().get::<WeaponInventory>(entity).unwrap();
        assert_eq!(inv.weapons.len(), 2);
        assert!(
            inv.weapons
                .iter()
                .any(|w| w.weapon_type == WeaponType::MagicWand)
        );
        assert!(app.world().resource::<PendingUpgradeIndex>().0.is_none());
    }

    #[test]
    fn new_weapon_starts_at_level_one() {
        let mut app = build_app();
        let entity = spawn_player(&mut app);
        app.world_mut()
            .resource_mut::<LevelUpChoices>()
            .choices
            .push(UpgradeChoice::NewWeapon(WeaponType::Knife));
        app.world_mut().resource_mut::<PendingUpgradeIndex>().0 = Some(0);

        run(&mut app);

        let inv = app.world().get::<WeaponInventory>(entity).unwrap();
        let knife = inv
            .weapons
            .iter()
            .find(|w| w.weapon_type == WeaponType::Knife)
            .unwrap();
        assert_eq!(knife.level, 1);
    }

    // --- WeaponUpgrade ---

    #[test]
    fn weapon_upgrade_increments_level() {
        let mut app = build_app();
        let entity = spawn_player(&mut app);
        app.world_mut()
            .resource_mut::<LevelUpChoices>()
            .choices
            .push(UpgradeChoice::WeaponUpgrade(WeaponType::Whip));
        app.world_mut().resource_mut::<PendingUpgradeIndex>().0 = Some(0);

        run(&mut app);

        let inv = app.world().get::<WeaponInventory>(entity).unwrap();
        let whip = inv
            .weapons
            .iter()
            .find(|w| w.weapon_type == WeaponType::Whip)
            .unwrap();
        assert_eq!(whip.level, 2, "Whip should be level 2 after upgrade");
    }

    #[test]
    fn weapon_upgrade_clamped_at_max_level() {
        let mut app = build_app();
        let entity = spawn_player(&mut app);
        // Set Whip to level 8 (max).
        app.world_mut()
            .get_mut::<WeaponInventory>(entity)
            .unwrap()
            .weapons[0]
            .level = 8;
        app.world_mut()
            .resource_mut::<LevelUpChoices>()
            .choices
            .push(UpgradeChoice::WeaponUpgrade(WeaponType::Whip));
        app.world_mut().resource_mut::<PendingUpgradeIndex>().0 = Some(0);

        run(&mut app);

        let inv = app.world().get::<WeaponInventory>(entity).unwrap();
        assert_eq!(inv.weapons[0].level, 8, "level must not exceed 8");
    }

    // --- PassiveItem (new) ---

    #[test]
    fn new_passive_added_to_inventory() {
        let mut app = build_app();
        let entity = spawn_player(&mut app);
        app.world_mut()
            .resource_mut::<LevelUpChoices>()
            .choices
            .push(UpgradeChoice::PassiveItem(PassiveItemType::Spinach));
        app.world_mut().resource_mut::<PendingUpgradeIndex>().0 = Some(0);

        run(&mut app);

        let inv = app.world().get::<PassiveInventory>(entity).unwrap();
        assert_eq!(inv.items.len(), 1);
        assert_eq!(inv.items[0].item_type, PassiveItemType::Spinach);
        assert_eq!(inv.items[0].level, 1);
    }

    #[test]
    fn spinach_increases_damage_multiplier() {
        let mut app = build_app();
        let entity = spawn_player(&mut app);
        let base_dmg = app
            .world()
            .get::<PlayerStats>(entity)
            .unwrap()
            .damage_multiplier;
        app.world_mut()
            .resource_mut::<LevelUpChoices>()
            .choices
            .push(UpgradeChoice::PassiveItem(PassiveItemType::Spinach));
        app.world_mut().resource_mut::<PendingUpgradeIndex>().0 = Some(0);

        run(&mut app);

        let stats = app.world().get::<PlayerStats>(entity).unwrap();
        assert!(
            (stats.damage_multiplier - (base_dmg + SPINACH_DAMAGE_PER_LEVEL)).abs() < 1e-6,
            "damage_multiplier should increase by {SPINACH_DAMAGE_PER_LEVEL}"
        );
    }

    #[test]
    fn wings_increases_move_speed() {
        let mut app = build_app();
        let entity = spawn_player(&mut app);
        let base_speed = app.world().get::<PlayerStats>(entity).unwrap().move_speed;
        app.world_mut()
            .resource_mut::<LevelUpChoices>()
            .choices
            .push(UpgradeChoice::PassiveItem(PassiveItemType::Wings));
        app.world_mut().resource_mut::<PendingUpgradeIndex>().0 = Some(0);

        run(&mut app);

        let stats = app.world().get::<PlayerStats>(entity).unwrap();
        assert!(
            (stats.move_speed - (base_speed + WINGS_SPEED_PER_LEVEL)).abs() < 1e-6,
            "move_speed should increase by {WINGS_SPEED_PER_LEVEL}"
        );
    }

    #[test]
    fn hollow_heart_increases_max_and_current_hp() {
        let mut app = build_app();
        let entity = spawn_player(&mut app);
        let (base_max_hp, base_cur_hp) = {
            let s = app.world().get::<PlayerStats>(entity).unwrap();
            (s.max_hp, s.current_hp)
        };
        app.world_mut()
            .resource_mut::<LevelUpChoices>()
            .choices
            .push(UpgradeChoice::PassiveItem(PassiveItemType::HollowHeart));
        app.world_mut().resource_mut::<PendingUpgradeIndex>().0 = Some(0);

        run(&mut app);

        let stats = app.world().get::<PlayerStats>(entity).unwrap();
        assert!(
            (stats.max_hp - (base_max_hp + HOLLOW_HEART_HP_PER_LEVEL)).abs() < 1e-6,
            "max_hp should increase by {HOLLOW_HEART_HP_PER_LEVEL}"
        );
        assert!(
            (stats.current_hp - (base_cur_hp + HOLLOW_HEART_HP_PER_LEVEL)).abs() < 1e-6,
            "current_hp should also increase by {HOLLOW_HEART_HP_PER_LEVEL}"
        );
    }

    #[test]
    fn empty_tome_increases_cooldown_reduction() {
        let mut app = build_app();
        let entity = spawn_player(&mut app);
        let base_cdr = app
            .world()
            .get::<PlayerStats>(entity)
            .unwrap()
            .cooldown_reduction;
        app.world_mut()
            .resource_mut::<LevelUpChoices>()
            .choices
            .push(UpgradeChoice::PassiveItem(PassiveItemType::EmptyTome));
        app.world_mut().resource_mut::<PendingUpgradeIndex>().0 = Some(0);

        run(&mut app);

        let stats = app.world().get::<PlayerStats>(entity).unwrap();
        assert!(
            (stats.cooldown_reduction - (base_cdr + EMPTY_TOME_CDR_PER_LEVEL)).abs() < 1e-6,
            "cooldown_reduction should increase by {EMPTY_TOME_CDR_PER_LEVEL}"
        );
    }

    #[test]
    fn empty_tome_cdr_capped_at_ninety_percent() {
        let mut app = build_app();
        let entity = spawn_player(&mut app);
        // Set CDR near the cap.
        app.world_mut()
            .get_mut::<PlayerStats>(entity)
            .unwrap()
            .cooldown_reduction = 0.88;
        app.world_mut()
            .resource_mut::<LevelUpChoices>()
            .choices
            .push(UpgradeChoice::PassiveItem(PassiveItemType::EmptyTome));
        app.world_mut().resource_mut::<PendingUpgradeIndex>().0 = Some(0);

        run(&mut app);

        let cdr = app
            .world()
            .get::<PlayerStats>(entity)
            .unwrap()
            .cooldown_reduction;
        assert!(cdr <= 0.9 + 1e-6, "CDR must not exceed 0.9, got {cdr}");
    }

    #[test]
    fn duplicator_increases_extra_projectiles() {
        let mut app = build_app();
        let entity = spawn_player(&mut app);
        let base_proj = app
            .world()
            .get::<PlayerStats>(entity)
            .unwrap()
            .extra_projectiles;
        app.world_mut()
            .resource_mut::<LevelUpChoices>()
            .choices
            .push(UpgradeChoice::PassiveItem(PassiveItemType::Duplicator));
        app.world_mut().resource_mut::<PendingUpgradeIndex>().0 = Some(0);

        run(&mut app);

        let stats = app.world().get::<PlayerStats>(entity).unwrap();
        assert_eq!(stats.extra_projectiles, base_proj + 1);
    }

    // --- PassiveUpgrade ---

    #[test]
    fn passive_upgrade_increments_level() {
        let mut app = build_app();
        let entity = spawn_player(&mut app);
        // Give player a Spinach at level 1.
        app.world_mut()
            .get_mut::<PassiveInventory>(entity)
            .unwrap()
            .items
            .push(PassiveState {
                item_type: PassiveItemType::Spinach,
                level: 1,
            });
        app.world_mut()
            .resource_mut::<LevelUpChoices>()
            .choices
            .push(UpgradeChoice::PassiveUpgrade(PassiveItemType::Spinach));
        app.world_mut().resource_mut::<PendingUpgradeIndex>().0 = Some(0);

        run(&mut app);

        let inv = app.world().get::<PassiveInventory>(entity).unwrap();
        let spinach = inv
            .items
            .iter()
            .find(|p| p.item_type == PassiveItemType::Spinach)
            .unwrap();
        assert_eq!(spinach.level, 2, "Spinach should be level 2");
    }

    #[test]
    fn passive_upgrade_also_applies_stat_bonus() {
        let mut app = build_app();
        let entity = spawn_player(&mut app);
        app.world_mut()
            .get_mut::<PassiveInventory>(entity)
            .unwrap()
            .items
            .push(PassiveState {
                item_type: PassiveItemType::Spinach,
                level: 1,
            });
        let base_dmg = app
            .world()
            .get::<PlayerStats>(entity)
            .unwrap()
            .damage_multiplier;
        app.world_mut()
            .resource_mut::<LevelUpChoices>()
            .choices
            .push(UpgradeChoice::PassiveUpgrade(PassiveItemType::Spinach));
        app.world_mut().resource_mut::<PendingUpgradeIndex>().0 = Some(0);

        run(&mut app);

        let stats = app.world().get::<PlayerStats>(entity).unwrap();
        assert!(
            (stats.damage_multiplier - (base_dmg + SPINACH_DAMAGE_PER_LEVEL)).abs() < 1e-6,
            "damage_multiplier must increase on PassiveUpgrade"
        );
    }

    #[test]
    fn passive_upgrade_clamped_at_max_level() {
        let mut app = build_app();
        let entity = spawn_player(&mut app);
        app.world_mut()
            .get_mut::<PassiveInventory>(entity)
            .unwrap()
            .items
            .push(PassiveState {
                item_type: PassiveItemType::Wings,
                level: 5,
            });
        app.world_mut()
            .resource_mut::<LevelUpChoices>()
            .choices
            .push(UpgradeChoice::PassiveUpgrade(PassiveItemType::Wings));
        app.world_mut().resource_mut::<PendingUpgradeIndex>().0 = Some(0);

        run(&mut app);

        let inv = app.world().get::<PassiveInventory>(entity).unwrap();
        let wings = inv
            .items
            .iter()
            .find(|p| p.item_type == PassiveItemType::Wings)
            .unwrap();
        assert_eq!(wings.level, 5, "passive level must not exceed 5");
    }

    // --- Out-of-bounds index is a no-op ---

    #[test]
    fn out_of_bounds_index_is_noop() {
        let mut app = build_app();
        let entity = spawn_player(&mut app);
        // No choices in the resource, but index 0 is pending.
        app.world_mut().resource_mut::<PendingUpgradeIndex>().0 = Some(0);

        run(&mut app);

        let inv = app.world().get::<WeaponInventory>(entity).unwrap();
        assert_eq!(inv.weapons.len(), 1, "inventory must not change");
        assert!(app.world().resource::<PendingUpgradeIndex>().0.is_none());
    }
}
