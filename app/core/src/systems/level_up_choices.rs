//! Level-up upgrade choice generation.
//!
//! [`generate_level_up_choices`] runs once on entering [`AppState::LevelUp`].
//! It inspects the player's current [`WeaponInventory`] and
//! [`PassiveInventory`], builds a pool of every valid [`UpgradeChoice`], and
//! randomly selects up to [`CHOICE_COUNT`] options to present on the UI.
//!
//! ## Selection rules
//!
//! | Source | Condition |
//! |--------|-----------|
//! | [`UpgradeChoice::WeaponUpgrade`] | Weapon owned, `level < max_weapon_level`, not evolved |
//! | [`UpgradeChoice::NewWeapon`] | Base weapon not owned, weapon slot available |
//! | [`UpgradeChoice::PassiveUpgrade`] | Passive owned, `level < max_passive_level` |
//! | [`UpgradeChoice::PassiveItem`] | Passive not owned, passive slot available |
//!
//! Choices are selected by a Fisher-Yates shuffle of the full candidate pool,
//! so each eligible item has an equal chance of appearing.

use std::collections::HashSet;

use bevy::prelude::*;
use rand::RngExt;

use crate::{
    components::{PassiveInventory, Player, WeaponInventory},
    config::GameParams,
    resources::LevelUpChoices,
    types::{PassiveItemType, UpgradeChoice, WeaponType},
};

// ---------------------------------------------------------------------------
// Fallback constants
// ---------------------------------------------------------------------------

const DEFAULT_MAX_WEAPON_LEVEL: u8 = 8;
const DEFAULT_MAX_PASSIVE_LEVEL: u8 = 5;
const DEFAULT_MAX_WEAPONS: usize = 6;
const DEFAULT_MAX_PASSIVES: usize = 6;

// ---------------------------------------------------------------------------
// Static item lists
// ---------------------------------------------------------------------------

/// All base (non-evolved) weapon types eligible to appear as new-weapon choices.
const BASE_WEAPONS: [WeaponType; 8] = [
    WeaponType::Whip,
    WeaponType::MagicWand,
    WeaponType::Knife,
    WeaponType::Garlic,
    WeaponType::Bible,
    WeaponType::ThunderRing,
    WeaponType::Cross,
    WeaponType::FireWand,
];

/// All passive item types eligible to appear as new-passive choices.
const ALL_PASSIVES: [PassiveItemType; 9] = [
    PassiveItemType::Spinach,
    PassiveItemType::Wings,
    PassiveItemType::HollowHeart,
    PassiveItemType::Clover,
    PassiveItemType::EmptyTome,
    PassiveItemType::Bracer,
    PassiveItemType::Spellbinder,
    PassiveItemType::Duplicator,
    PassiveItemType::Pummarola,
];

/// Number of upgrade cards shown to the player on each level-up.
pub const CHOICE_COUNT: usize = 3;

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Generates [`CHOICE_COUNT`] random upgrade choices and stores them in
/// [`LevelUpChoices`].
///
/// Runs on [`OnEnter(AppState::LevelUp)`](crate::states::AppState::LevelUp).
/// When fewer than [`CHOICE_COUNT`] valid choices exist (e.g. all items are
/// maxed), all remaining options are returned.
pub fn generate_level_up_choices(
    player_q: Query<(&WeaponInventory, &PassiveInventory), With<Player>>,
    mut level_up_choices: ResMut<LevelUpChoices>,
    game_cfg: GameParams,
) {
    let Ok((weapon_inv, passive_inv)) = player_q.single() else {
        level_up_choices.choices.clear();
        return;
    };

    let cfg = game_cfg.get();
    let max_weapon_level = cfg
        .map(|c| c.max_weapon_level)
        .unwrap_or(DEFAULT_MAX_WEAPON_LEVEL);
    let max_passive_level = cfg
        .map(|c| c.max_passive_level)
        .unwrap_or(DEFAULT_MAX_PASSIVE_LEVEL);
    let max_weapons = cfg.map(|c| c.max_weapons).unwrap_or(DEFAULT_MAX_WEAPONS);
    let max_passives = cfg.map(|c| c.max_passives).unwrap_or(DEFAULT_MAX_PASSIVES);

    let mut pool: Vec<UpgradeChoice> = Vec::new();

    // 1. Upgradeable owned weapons (level < max, not evolved).
    for weapon in &weapon_inv.weapons {
        if weapon.level < max_weapon_level && !weapon.evolved {
            pool.push(UpgradeChoice::WeaponUpgrade(weapon.weapon_type));
        }
    }

    // 2. New base weapons (not owned, slot available).
    if weapon_inv.weapons.len() < max_weapons {
        let owned_weapons: HashSet<WeaponType> =
            weapon_inv.weapons.iter().map(|w| w.weapon_type).collect();
        for &weapon_type in &BASE_WEAPONS {
            if !owned_weapons.contains(&weapon_type) {
                pool.push(UpgradeChoice::NewWeapon(weapon_type));
            }
        }
    }

    // 3. Upgradeable owned passives (level < max).
    for passive in &passive_inv.items {
        if passive.level < max_passive_level {
            pool.push(UpgradeChoice::PassiveUpgrade(passive.item_type));
        }
    }

    // 4. New passives (not owned, slot available).
    if passive_inv.items.len() < max_passives {
        let owned_passives: HashSet<PassiveItemType> =
            passive_inv.items.iter().map(|p| p.item_type).collect();
        for &passive_type in &ALL_PASSIVES {
            if !owned_passives.contains(&passive_type) {
                pool.push(UpgradeChoice::PassiveItem(passive_type));
            }
        }
    }

    // Shuffle the pool and take up to CHOICE_COUNT choices.
    fisher_yates_shuffle(&mut pool);
    pool.truncate(CHOICE_COUNT);
    level_up_choices.choices = pool;
}

/// In-place Fisher-Yates shuffle using the thread-local RNG.
fn fisher_yates_shuffle<T>(items: &mut [T]) {
    let mut rng = rand::rng();
    for i in (1..items.len()).rev() {
        let j = rng.random_range(0..i + 1);
        items.swap(i, j);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        components::{PassiveInventory, Player, WeaponInventory},
        resources::LevelUpChoices,
        types::{PassiveState, WeaponState, WeaponType},
    };

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(LevelUpChoices::default());
        app
    }

    fn spawn_player(app: &mut App, weapons: Vec<WeaponState>, passives: Vec<PassiveState>) {
        app.world_mut().spawn((
            Player,
            WeaponInventory { weapons },
            PassiveInventory { items: passives },
        ));
    }

    fn run(app: &mut App) {
        use bevy::ecs::system::RunSystemOnce as _;
        app.world_mut()
            .run_system_once(generate_level_up_choices)
            .unwrap();
    }

    fn choices(app: &App) -> Vec<UpgradeChoice> {
        app.world().resource::<LevelUpChoices>().choices.clone()
    }

    // --- No player ---

    /// No player entity → choices are cleared and system does not panic.
    #[test]
    fn no_player_clears_choices() {
        let mut app = build_app();
        app.world_mut()
            .resource_mut::<LevelUpChoices>()
            .choices
            .push(UpgradeChoice::NewWeapon(WeaponType::Whip));
        run(&mut app);
        assert!(
            choices(&app).is_empty(),
            "choices must be cleared when no player exists"
        );
    }

    // --- Choice count ---

    /// With a full pool, exactly CHOICE_COUNT choices are returned.
    #[test]
    fn returns_at_most_choice_count() {
        let mut app = build_app();
        // Player with one whip at level 1 — large pool available.
        spawn_player(&mut app, vec![WeaponState::new(WeaponType::Whip)], vec![]);
        run(&mut app);
        assert_eq!(
            choices(&app).len(),
            CHOICE_COUNT,
            "exactly {CHOICE_COUNT} choices expected with large pool"
        );
    }

    /// With a tiny pool (fewer than CHOICE_COUNT items), all are returned.
    #[test]
    fn returns_all_when_pool_smaller_than_choice_count() {
        let mut app = build_app();
        // All 8 weapons owned at max level, all 9 passives owned at max level
        // → pool is empty; no choices generated.
        let weapons: Vec<WeaponState> = BASE_WEAPONS
            .iter()
            .map(|&wt| {
                let mut ws = WeaponState::new(wt);
                ws.level = DEFAULT_MAX_WEAPON_LEVEL;
                ws
            })
            .collect();
        let passives: Vec<PassiveState> = ALL_PASSIVES
            .iter()
            .map(|&pt| PassiveState {
                item_type: pt,
                level: DEFAULT_MAX_PASSIVE_LEVEL,
            })
            .collect();
        spawn_player(&mut app, weapons, passives);
        run(&mut app);
        assert!(
            choices(&app).is_empty(),
            "no choices expected when all items are maxed"
        );
    }

    // --- WeaponUpgrade eligibility ---

    /// Owned weapon below max level appears as WeaponUpgrade.
    #[test]
    fn owned_weapon_below_max_is_upgradeable() {
        let mut app = build_app();
        // Only weapon: Whip at level 1. All passive slots full (to limit other choices).
        let passives: Vec<PassiveState> = ALL_PASSIVES
            .iter()
            .map(|&pt| PassiveState {
                item_type: pt,
                level: DEFAULT_MAX_PASSIVE_LEVEL,
            })
            .collect();
        // Fill weapon slots so no NewWeapon choices appear.
        let mut weapons: Vec<WeaponState> = BASE_WEAPONS
            .iter()
            .map(|&wt| WeaponState::new(wt))
            .collect();
        // Only Whip upgradeable; rest at max.
        for w in weapons.iter_mut().skip(1) {
            w.level = DEFAULT_MAX_WEAPON_LEVEL;
        }
        spawn_player(&mut app, weapons, passives);
        run(&mut app);

        let c = choices(&app);
        assert_eq!(c.len(), 1, "only Whip upgrade should be in pool");
        assert!(
            matches!(c[0], UpgradeChoice::WeaponUpgrade(WeaponType::Whip)),
            "expected WeaponUpgrade(Whip), got {c:?}"
        );
    }

    /// Evolved weapon is excluded from upgrade choices.
    #[test]
    fn evolved_weapon_excluded() {
        let mut app = build_app();
        let mut whip = WeaponState::new(WeaponType::Whip);
        whip.evolved = true;
        whip.level = 1; // below max, but evolved
        let passives: Vec<PassiveState> = ALL_PASSIVES
            .iter()
            .map(|&pt| PassiveState {
                item_type: pt,
                level: DEFAULT_MAX_PASSIVE_LEVEL,
            })
            .collect();
        // Fill remaining weapon slots so pool stays small.
        let mut weapons = vec![whip];
        for &wt in BASE_WEAPONS.iter().skip(1) {
            let mut ws = WeaponState::new(wt);
            ws.level = DEFAULT_MAX_WEAPON_LEVEL;
            weapons.push(ws);
        }
        spawn_player(&mut app, weapons, passives);
        run(&mut app);

        assert!(
            choices(&app).is_empty(),
            "evolved weapon must not appear in upgrade pool"
        );
    }

    /// Weapon at max level is excluded.
    #[test]
    fn max_level_weapon_excluded() {
        let mut app = build_app();
        let mut whip = WeaponState::new(WeaponType::Whip);
        whip.level = DEFAULT_MAX_WEAPON_LEVEL;
        let passives: Vec<PassiveState> = ALL_PASSIVES
            .iter()
            .map(|&pt| PassiveState {
                item_type: pt,
                level: DEFAULT_MAX_PASSIVE_LEVEL,
            })
            .collect();
        // Fill remaining weapon slots.
        let mut weapons = vec![whip];
        for &wt in BASE_WEAPONS.iter().skip(1) {
            let mut ws = WeaponState::new(wt);
            ws.level = DEFAULT_MAX_WEAPON_LEVEL;
            weapons.push(ws);
        }
        spawn_player(&mut app, weapons, passives);
        run(&mut app);

        assert!(
            choices(&app).is_empty(),
            "max-level weapon must not appear in upgrade pool"
        );
    }

    // --- NewWeapon eligibility ---

    /// Unowned base weapon appears as NewWeapon when a slot is open.
    #[test]
    fn unowned_weapon_is_new_weapon_choice() {
        let mut app = build_app();
        // Only Whip owned; all passives maxed to keep pool focused.
        let passives: Vec<PassiveState> = ALL_PASSIVES
            .iter()
            .map(|&pt| PassiveState {
                item_type: pt,
                level: DEFAULT_MAX_PASSIVE_LEVEL,
            })
            .collect();
        // Whip maxed so it cannot be upgraded; remaining 7 base weapons are unowned.
        let mut whip = WeaponState::new(WeaponType::Whip);
        whip.level = DEFAULT_MAX_WEAPON_LEVEL;
        spawn_player(&mut app, vec![whip], passives);
        run(&mut app);

        let c = choices(&app);
        assert_eq!(
            c.len(),
            CHOICE_COUNT,
            "7 unowned weapons → at least {CHOICE_COUNT} choices"
        );
        assert!(
            c.iter().all(|ch| matches!(ch, UpgradeChoice::NewWeapon(_))),
            "all choices should be NewWeapon when passives are maxed"
        );
    }

    /// No NewWeapon choices when weapon slots are full.
    #[test]
    fn no_new_weapon_when_slots_full() {
        let mut app = build_app();
        // 6 weapons (full) at max level; all passives maxed.
        let weapons: Vec<WeaponState> = BASE_WEAPONS
            .iter()
            .take(DEFAULT_MAX_WEAPONS)
            .map(|&wt| {
                let mut ws = WeaponState::new(wt);
                ws.level = DEFAULT_MAX_WEAPON_LEVEL;
                ws
            })
            .collect();
        let passives: Vec<PassiveState> = ALL_PASSIVES
            .iter()
            .map(|&pt| PassiveState {
                item_type: pt,
                level: DEFAULT_MAX_PASSIVE_LEVEL,
            })
            .collect();
        spawn_player(&mut app, weapons, passives);
        run(&mut app);

        assert!(
            choices(&app)
                .iter()
                .all(|ch| !matches!(ch, UpgradeChoice::NewWeapon(_))),
            "NewWeapon must not appear when weapon slots are full"
        );
    }

    // --- Passive eligibility ---

    /// Owned passive below max level appears as PassiveUpgrade.
    #[test]
    fn owned_passive_below_max_is_upgradeable() {
        let mut app = build_app();
        // All weapons at max; only Spinach at level 1 (upgradeable).
        let weapons: Vec<WeaponState> = BASE_WEAPONS
            .iter()
            .take(DEFAULT_MAX_WEAPONS)
            .map(|&wt| {
                let mut ws = WeaponState::new(wt);
                ws.level = DEFAULT_MAX_WEAPON_LEVEL;
                ws
            })
            .collect();
        // Fill passive slots: Spinach at level 1, rest at max.
        let mut passives: Vec<PassiveState> = ALL_PASSIVES
            .iter()
            .take(DEFAULT_MAX_PASSIVES)
            .map(|&pt| PassiveState {
                item_type: pt,
                level: DEFAULT_MAX_PASSIVE_LEVEL,
            })
            .collect();
        passives[0].level = 1; // Spinach upgradeable
        spawn_player(&mut app, weapons, passives);
        run(&mut app);

        let c = choices(&app);
        assert_eq!(c.len(), 1);
        assert!(
            matches!(
                c[0],
                UpgradeChoice::PassiveUpgrade(PassiveItemType::Spinach)
            ),
            "expected PassiveUpgrade(Spinach), got {c:?}"
        );
    }

    /// Passive at max level is excluded.
    #[test]
    fn max_level_passive_excluded() {
        let mut app = build_app();
        let weapons: Vec<WeaponState> = BASE_WEAPONS
            .iter()
            .take(DEFAULT_MAX_WEAPONS)
            .map(|&wt| {
                let mut ws = WeaponState::new(wt);
                ws.level = DEFAULT_MAX_WEAPON_LEVEL;
                ws
            })
            .collect();
        // All passives at max level.
        let passives: Vec<PassiveState> = ALL_PASSIVES
            .iter()
            .take(DEFAULT_MAX_PASSIVES)
            .map(|&pt| PassiveState {
                item_type: pt,
                level: DEFAULT_MAX_PASSIVE_LEVEL,
            })
            .collect();
        spawn_player(&mut app, weapons, passives);
        run(&mut app);

        assert!(
            choices(&app)
                .iter()
                .all(|ch| !matches!(ch, UpgradeChoice::PassiveUpgrade(_))),
            "max-level passive must not appear as upgrade choice"
        );
    }

    /// No PassiveItem choices when passive slots are full.
    #[test]
    fn no_new_passive_when_slots_full() {
        let mut app = build_app();
        let weapons: Vec<WeaponState> = BASE_WEAPONS
            .iter()
            .take(DEFAULT_MAX_WEAPONS)
            .map(|&wt| {
                let mut ws = WeaponState::new(wt);
                ws.level = DEFAULT_MAX_WEAPON_LEVEL;
                ws
            })
            .collect();
        // 6 passives (full slots) at level 1 — all upgradeable but no room for new ones.
        let passives: Vec<PassiveState> = ALL_PASSIVES
            .iter()
            .take(DEFAULT_MAX_PASSIVES)
            .map(|&pt| PassiveState {
                item_type: pt,
                level: 1,
            })
            .collect();
        spawn_player(&mut app, weapons, passives);
        run(&mut app);

        assert!(
            choices(&app)
                .iter()
                .all(|ch| !matches!(ch, UpgradeChoice::PassiveItem(_))),
            "PassiveItem must not appear when passive slots are full"
        );
    }

    // --- Fisher-Yates ---

    /// `fisher_yates_shuffle` does not change the length of the slice.
    #[test]
    fn fisher_yates_preserves_length() {
        let mut v: Vec<i32> = (0..10).collect();
        fisher_yates_shuffle(&mut v);
        assert_eq!(v.len(), 10);
    }

    /// `fisher_yates_shuffle` preserves all elements (set equality).
    #[test]
    fn fisher_yates_preserves_elements() {
        use std::collections::HashSet;
        let original: Vec<i32> = (0..10).collect();
        let mut v = original.clone();
        fisher_yates_shuffle(&mut v);
        let orig_set: HashSet<_> = original.into_iter().collect();
        let shuf_set: HashSet<_> = v.into_iter().collect();
        assert_eq!(
            orig_set, shuf_set,
            "shuffle must not add or remove elements"
        );
    }
}
