//! Game configuration loaded from `assets/config/game.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

use crate::types::MetaUpgradeType;

// ---------------------------------------------------------------------------
// Fallback constants (used while game.ron is still loading)
//
// These are the single source of truth for every game.ron field's fallback
// value.  All system files that need a fallback import or use GameParams
// methods (defined below) rather than redeclaring their own copies.
// ---------------------------------------------------------------------------

// --- inventory caps ---
pub(crate) const DEFAULT_MAX_WEAPON_LEVEL: u8 = 8;
pub(crate) const DEFAULT_MAX_PASSIVE_LEVEL: u8 = 5;
pub(crate) const DEFAULT_MAX_WEAPONS: usize = 6;
pub(crate) const DEFAULT_MAX_PASSIVES: usize = 6;

// --- XP / levelling ---
pub(crate) const DEFAULT_XP_LEVEL_BASE: u32 = 20;
pub(crate) const DEFAULT_XP_LEVEL_MULTIPLIER: f32 = 1.2;
pub(crate) const DEFAULT_CHOICE_COUNT: usize = 3;
pub(crate) const DEFAULT_LUCK_BONUS_CHOICE_THRESHOLD: f32 = 1.5;

// --- spatial grid ---
pub(crate) const DEFAULT_SPATIAL_GRID_CELL_SIZE: f32 = 64.0;

// --- treasure chests ---
pub(crate) const DEFAULT_TREASURE_RADIUS: f32 = 20.0;
pub(crate) const DEFAULT_TREASURE_GOLD: u32 = 50;
pub(crate) const DEFAULT_TREASURE_HP_RECOVERY_PCT: f32 = 0.3;
pub(crate) const DEFAULT_TREASURE_GLOW_DISTANCE: f32 = 150.0;
pub(crate) const DEFAULT_TREASURE_SPAWN_FLASH_DURATION: f32 = 0.35;

// --- meta shop costs ---
const DEFAULT_SHOP_UPGRADE_COST_HP: u32 = 300;
const DEFAULT_SHOP_UPGRADE_COST_SPEED: u32 = 300;
const DEFAULT_SHOP_UPGRADE_COST_DAMAGE: u32 = 300;
const DEFAULT_SHOP_UPGRADE_COST_XP: u32 = 300;
const DEFAULT_SHOP_UPGRADE_COST_WEAPON: u32 = 500;

// --- meta upgrade stat bonuses ---
/// Flat HP bonus per BonusHp purchase (same magnitude as HollowHeart per level).
pub(crate) const DEFAULT_META_UPGRADE_HP_BONUS: f32 = 20.0;
/// Flat speed bonus (px/s) per BonusSpeed purchase (same magnitude as Wings per level).
pub(crate) const DEFAULT_META_UPGRADE_SPEED_BONUS: f32 = 20.0;
/// Damage multiplier added per BonusDamage purchase (same magnitude as Spinach per level).
pub(crate) const DEFAULT_META_UPGRADE_DAMAGE_BONUS: f32 = 0.1;
/// XP multiplier added per BonusXp purchase (10 % more XP per upgrade).
pub(crate) const DEFAULT_META_UPGRADE_XP_BONUS: f32 = 0.1;

fn default_upgrade_cost(upgrade: MetaUpgradeType) -> u32 {
    match upgrade {
        MetaUpgradeType::BonusHp => DEFAULT_SHOP_UPGRADE_COST_HP,
        MetaUpgradeType::BonusSpeed => DEFAULT_SHOP_UPGRADE_COST_SPEED,
        MetaUpgradeType::BonusDamage => DEFAULT_SHOP_UPGRADE_COST_DAMAGE,
        MetaUpgradeType::BonusXp => DEFAULT_SHOP_UPGRADE_COST_XP,
        MetaUpgradeType::StartingWeapon => DEFAULT_SHOP_UPGRADE_COST_WEAPON,
    }
}

fn default_upgrade_stat_bonus(upgrade: MetaUpgradeType) -> f32 {
    match upgrade {
        MetaUpgradeType::BonusHp => DEFAULT_META_UPGRADE_HP_BONUS,
        MetaUpgradeType::BonusSpeed => DEFAULT_META_UPGRADE_SPEED_BONUS,
        MetaUpgradeType::BonusDamage => DEFAULT_META_UPGRADE_DAMAGE_BONUS,
        MetaUpgradeType::BonusXp => DEFAULT_META_UPGRADE_XP_BONUS,
        MetaUpgradeType::StartingWeapon => 0.0,
    }
}

// ---------------------------------------------------------------------------
// Asset type
// ---------------------------------------------------------------------------

/// Game-wide rules and settings, loaded from `assets/config/game.ron`.
///
/// Covers viewport size, inventory caps, XP levelling curve, camera feel,
/// and spatial-grid tuning. Systems that read via [`GameParams`] pick up
/// hot-reloaded values automatically on the next frame.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct GameConfig {
    // Window / viewport
    pub window_width: u32,
    pub window_height: u32,
    // Inventory caps
    pub max_weapons: usize,
    pub max_passives: usize,
    pub max_weapon_level: u8,
    pub max_passive_level: u8,
    // Game rules
    pub boss_spawn_time: f32,
    pub treasure_spawn_interval: f32,
    /// Collision radius of treasure chests in pixels.
    pub treasure_radius: f32,
    /// Gold awarded when a chest reward rolls gold (one of three possible rewards).
    pub treasure_gold_reward: u32,
    /// Fraction of max HP restored when a chest reward rolls HP recovery (0.0–1.0).
    pub treasure_hp_recovery_pct: f32,
    /// Player distance (pixels) within which the radial-glow highlight becomes
    /// visible on a treasure chest.
    pub treasure_glow_distance: f32,
    /// Duration in seconds of the white-flash-to-yellow spawn animation.
    pub treasure_spawn_flash_duration: f32,
    // XP / levelling
    pub xp_level_base: u32,
    pub xp_level_multiplier: f32,
    pub level_up_choice_count: usize,
    /// Luck value at or above which the player receives one extra upgrade card.
    ///
    /// Base luck is 1.0; Clover adds +0.10 per level.  The default (1.5) means
    /// five levels of Clover unlock a 4th card.
    pub luck_bonus_choice_threshold: f32,
    // Camera
    pub camera_lerp_speed: f32,
    // Spatial partitioning
    pub spatial_grid_cell_size: f32,
    // Projectile defaults
    pub base_projectile_speed: f32,
    pub base_projectile_lifetime: f32,
    // Boss Phase2 behavior
    /// HP fraction (inclusive) at which Phase1 → Phase2 triggers.
    pub boss_phase2_hp_threshold: f32,
    /// HP fraction (inclusive) at which Phase2 → Phase3 triggers.
    pub boss_phase3_hp_threshold: f32,
    /// Speed multiplier applied to the boss's base move speed in Phase2.
    pub boss_phase2_speed_multiplier: f32,
    /// Number of Mini Deaths summoned at the Phase2 transition.
    pub mini_death_spawn_count: usize,
    /// Radial distance from the boss center when placing Mini Deaths (pixels).
    pub mini_death_spawn_radius: f32,
    // Boss Phase3 behavior
    /// Speed multiplier applied to the boss's base move speed in Phase3.
    pub boss_phase3_speed_multiplier: f32,
    /// Number of Mini Deaths summoned at the Phase3 transition.
    pub mini_death_spawn_count_phase3: usize,
    /// Seconds between scythe projectile shots in Phase3.
    pub boss_scythe_interval: f32,
    /// Scythe projectile travel speed in pixels per second.
    pub boss_scythe_speed: f32,
    /// Scythe projectile lifetime in seconds before despawn.
    pub boss_scythe_lifetime: f32,
    /// Damage dealt to the player on scythe hit.
    pub boss_scythe_damage: f32,
    /// Scythe projectile collider radius in pixels.
    pub boss_scythe_radius: f32,
    // Shop upgrade costs
    /// Gold cost to purchase the max-HP permanent upgrade.
    pub shop_upgrade_cost_hp: u32,
    /// Gold cost to purchase the move-speed permanent upgrade.
    pub shop_upgrade_cost_speed: u32,
    /// Gold cost to purchase the damage permanent upgrade.
    pub shop_upgrade_cost_damage: u32,
    /// Gold cost to purchase the XP-gain permanent upgrade.
    pub shop_upgrade_cost_xp: u32,
    /// Gold cost to purchase the starting-weapon permanent upgrade.
    pub shop_upgrade_cost_weapon: u32,
    // Meta upgrade bonus values (applied to PlayerStats at run start)
    /// Flat HP added to max_hp per BonusHp purchase.
    pub meta_upgrade_hp_bonus: f32,
    /// Flat speed (px/s) added to move_speed per BonusSpeed purchase.
    pub meta_upgrade_speed_bonus: f32,
    /// Amount added to damage_multiplier per BonusDamage purchase.
    pub meta_upgrade_damage_bonus: f32,
    /// Amount added to xp_multiplier per BonusXp purchase.
    pub meta_upgrade_xp_bonus: f32,
}

impl GameConfig {
    /// Returns the gold cost for a given permanent upgrade.
    pub fn upgrade_cost(&self, upgrade: MetaUpgradeType) -> u32 {
        match upgrade {
            MetaUpgradeType::BonusHp => self.shop_upgrade_cost_hp,
            MetaUpgradeType::BonusSpeed => self.shop_upgrade_cost_speed,
            MetaUpgradeType::BonusDamage => self.shop_upgrade_cost_damage,
            MetaUpgradeType::BonusXp => self.shop_upgrade_cost_xp,
            MetaUpgradeType::StartingWeapon => self.shop_upgrade_cost_weapon,
        }
    }

    /// Returns the stat bonus applied per purchase of the given upgrade type.
    ///
    /// The meaning of the returned `f32` depends on the upgrade:
    /// - `BonusHp`/`BonusSpeed` — flat additive amount
    /// - `BonusDamage`/`BonusXp` — multiplier delta (added to a factor that starts at 1.0)
    /// - `StartingWeapon` — returns `0.0` (no stat effect; it is a selection unlock)
    pub fn upgrade_stat_bonus(&self, upgrade: MetaUpgradeType) -> f32 {
        match upgrade {
            MetaUpgradeType::BonusHp => self.meta_upgrade_hp_bonus,
            MetaUpgradeType::BonusSpeed => self.meta_upgrade_speed_bonus,
            MetaUpgradeType::BonusDamage => self.meta_upgrade_damage_bonus,
            MetaUpgradeType::BonusXp => self.meta_upgrade_xp_bonus,
            MetaUpgradeType::StartingWeapon => 0.0,
        }
    }
}

/// Resource holding the handle to the loaded game configuration.
#[derive(Resource)]
pub struct GameConfigHandle(pub Handle<GameConfig>);

// ---------------------------------------------------------------------------
// SystemParam bundle
// ---------------------------------------------------------------------------

/// SystemParam bundle for accessing [`GameConfig`].
///
/// Returns `None` while the asset is still loading or when
/// [`GameConfigPlugin`] has not been registered (e.g. in unit tests).
/// Call `.get()` to obtain `Option<&GameConfig>`.
///
/// [`GameConfigPlugin`]: crate::config::GameConfigPlugin
#[derive(SystemParam)]
pub struct GameParams<'w> {
    handle: Option<Res<'w, GameConfigHandle>>,
    assets: Option<Res<'w, Assets<GameConfig>>>,
}

impl<'w> GameParams<'w> {
    /// Returns the currently loaded [`GameConfig`], or `None` while loading.
    pub fn get(&self) -> Option<&GameConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }

    /// Returns the gold cost for a given permanent upgrade.
    ///
    /// Falls back to the hardcoded value from [`crate::types::upgrade_cost`]
    /// while the config asset is still loading.
    pub fn upgrade_cost(&self, upgrade: MetaUpgradeType) -> u32 {
        self.get()
            .map(|c| c.upgrade_cost(upgrade))
            .unwrap_or_else(|| default_upgrade_cost(upgrade))
    }

    /// Returns the stat bonus applied per purchase of the given upgrade type.
    ///
    /// Falls back to `DEFAULT_META_UPGRADE_*` constants while the config asset
    /// is still loading.
    pub fn upgrade_stat_bonus(&self, upgrade: MetaUpgradeType) -> f32 {
        self.get()
            .map(|c| c.upgrade_stat_bonus(upgrade))
            .unwrap_or_else(|| default_upgrade_stat_bonus(upgrade))
    }

    // --- Inventory caps ---

    pub fn max_weapon_level(&self) -> u8 {
        self.get()
            .map(|c| c.max_weapon_level)
            .unwrap_or(DEFAULT_MAX_WEAPON_LEVEL)
    }

    pub fn max_passive_level(&self) -> u8 {
        self.get()
            .map(|c| c.max_passive_level)
            .unwrap_or(DEFAULT_MAX_PASSIVE_LEVEL)
    }

    pub fn max_weapons(&self) -> usize {
        self.get()
            .map(|c| c.max_weapons)
            .unwrap_or(DEFAULT_MAX_WEAPONS)
    }

    pub fn max_passives(&self) -> usize {
        self.get()
            .map(|c| c.max_passives)
            .unwrap_or(DEFAULT_MAX_PASSIVES)
    }

    // --- XP / levelling ---

    pub fn xp_level_base(&self) -> u32 {
        self.get()
            .map(|c| c.xp_level_base)
            .unwrap_or(DEFAULT_XP_LEVEL_BASE)
    }

    pub fn xp_level_multiplier(&self) -> f32 {
        self.get()
            .map(|c| c.xp_level_multiplier)
            .unwrap_or(DEFAULT_XP_LEVEL_MULTIPLIER)
    }

    pub fn choice_count(&self) -> usize {
        self.get()
            .map(|c| c.level_up_choice_count)
            .unwrap_or(DEFAULT_CHOICE_COUNT)
    }

    pub fn luck_bonus_choice_threshold(&self) -> f32 {
        self.get()
            .map(|c| c.luck_bonus_choice_threshold)
            .unwrap_or(DEFAULT_LUCK_BONUS_CHOICE_THRESHOLD)
    }

    // --- Treasure chests ---

    pub fn treasure_radius(&self) -> f32 {
        self.get()
            .map(|c| c.treasure_radius)
            .unwrap_or(DEFAULT_TREASURE_RADIUS)
    }

    pub fn treasure_gold(&self) -> u32 {
        self.get()
            .map(|c| c.treasure_gold_reward)
            .unwrap_or(DEFAULT_TREASURE_GOLD)
    }

    pub fn treasure_hp_recovery_pct(&self) -> f32 {
        self.get()
            .map(|c| c.treasure_hp_recovery_pct)
            .unwrap_or(DEFAULT_TREASURE_HP_RECOVERY_PCT)
    }

    pub fn treasure_glow_distance(&self) -> f32 {
        self.get()
            .map(|c| c.treasure_glow_distance)
            .unwrap_or(DEFAULT_TREASURE_GLOW_DISTANCE)
    }

    pub fn treasure_spawn_flash_duration(&self) -> f32 {
        self.get()
            .map(|c| c.treasure_spawn_flash_duration)
            .unwrap_or(DEFAULT_TREASURE_SPAWN_FLASH_DURATION)
    }

    pub fn spatial_grid_cell_size(&self) -> f32 {
        self.get()
            .map(|c| c.spatial_grid_cell_size)
            .unwrap_or(DEFAULT_SPATIAL_GRID_CELL_SIZE)
    }
}

// ---------------------------------------------------------------------------
// Hot-reload system
// ---------------------------------------------------------------------------

/// Handles hot-reloading of game configuration.
///
/// On `Modified`, logs the reload. Systems that read via [`GameParams`]
/// will automatically pick up the new values on the next frame.
pub fn hot_reload_game_config(mut events: MessageReader<AssetEvent<GameConfig>>) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id: _ } => {
                info!("✅ Game config loaded");
            }
            AssetEvent::Modified { id: _ } => {
                info!("🔥 Hot-reloading game config!");
            }
            AssetEvent::Removed { id: _ } => {
                warn!("⚠️ Game config removed");
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_config_deserialization() {
        let ron_data = r#"
GameConfig(
    window_width: 1280,
    window_height: 720,
    max_weapons: 6,
    max_passives: 6,
    max_weapon_level: 8,
    max_passive_level: 5,
    boss_spawn_time: 1800.0,
    treasure_spawn_interval: 180.0,
    treasure_radius: 20.0,
    treasure_gold_reward: 50,
    treasure_hp_recovery_pct: 0.3,
    treasure_glow_distance: 150.0,
    treasure_spawn_flash_duration: 0.35,
    xp_level_base: 20,
    xp_level_multiplier: 1.2,
    level_up_choice_count: 3,
    luck_bonus_choice_threshold: 1.5,
    camera_lerp_speed: 10.0,
    spatial_grid_cell_size: 64.0,
    base_projectile_speed: 300.0,
    base_projectile_lifetime: 5.0,
    boss_phase2_hp_threshold: 0.6,
    boss_phase3_hp_threshold: 0.3,
    boss_phase2_speed_multiplier: 1.5,
    mini_death_spawn_count: 3,
    mini_death_spawn_radius: 80.0,
    boss_phase3_speed_multiplier: 2.0,
    mini_death_spawn_count_phase3: 5,
    boss_scythe_interval: 3.0,
    boss_scythe_speed: 250.0,
    boss_scythe_lifetime: 8.0,
    boss_scythe_damage: 80.0,
    boss_scythe_radius: 15.0,
    shop_upgrade_cost_hp: 300,
    shop_upgrade_cost_speed: 300,
    shop_upgrade_cost_damage: 300,
    shop_upgrade_cost_xp: 300,
    shop_upgrade_cost_weapon: 500,
    meta_upgrade_hp_bonus: 20.0,
    meta_upgrade_speed_bonus: 20.0,
    meta_upgrade_damage_bonus: 0.1,
    meta_upgrade_xp_bonus: 0.1,
)
"#;
        let config: GameConfig = ron::de::from_str(ron_data).unwrap();
        assert_eq!(config.window_width, 1280);
        assert_eq!(config.window_height, 720);
        assert_eq!(config.max_weapons, 6);
        assert_eq!(config.max_passives, 6);
        assert_eq!(config.max_weapon_level, 8);
        assert_eq!(config.max_passive_level, 5);
        assert_eq!(config.boss_spawn_time, 1800.0);
        assert_eq!(config.treasure_spawn_interval, 180.0);
        assert_eq!(config.treasure_radius, 20.0);
        assert_eq!(config.treasure_gold_reward, 50);
        assert_eq!(config.treasure_hp_recovery_pct, 0.3);
        assert_eq!(config.treasure_glow_distance, 150.0);
        assert_eq!(config.treasure_spawn_flash_duration, 0.35);
        assert_eq!(config.xp_level_base, 20);
        assert_eq!(config.level_up_choice_count, 3);
        assert_eq!(config.spatial_grid_cell_size, 64.0);
        assert_eq!(config.base_projectile_speed, 300.0);
        assert_eq!(config.base_projectile_lifetime, 5.0);
        assert_eq!(config.boss_phase2_hp_threshold, 0.6);
        assert_eq!(config.boss_phase3_hp_threshold, 0.3);
        assert_eq!(config.boss_phase2_speed_multiplier, 1.5);
        assert_eq!(config.mini_death_spawn_count, 3);
        assert_eq!(config.mini_death_spawn_radius, 80.0);
        assert_eq!(config.boss_phase3_speed_multiplier, 2.0);
        assert_eq!(config.mini_death_spawn_count_phase3, 5);
        assert_eq!(config.boss_scythe_interval, 3.0);
        assert_eq!(config.boss_scythe_speed, 250.0);
        assert_eq!(config.boss_scythe_lifetime, 8.0);
        assert_eq!(config.boss_scythe_damage, 80.0);
        assert_eq!(config.boss_scythe_radius, 15.0);
        assert_eq!(config.shop_upgrade_cost_hp, 300);
        assert_eq!(config.shop_upgrade_cost_speed, 300);
        assert_eq!(config.shop_upgrade_cost_damage, 300);
        assert_eq!(config.shop_upgrade_cost_xp, 300);
        assert_eq!(config.shop_upgrade_cost_weapon, 500);
        assert_eq!(config.meta_upgrade_hp_bonus, DEFAULT_META_UPGRADE_HP_BONUS);
        assert_eq!(
            config.meta_upgrade_speed_bonus,
            DEFAULT_META_UPGRADE_SPEED_BONUS
        );
        assert_eq!(
            config.meta_upgrade_damage_bonus,
            DEFAULT_META_UPGRADE_DAMAGE_BONUS
        );
        assert_eq!(config.meta_upgrade_xp_bonus, DEFAULT_META_UPGRADE_XP_BONUS);
    }
}
