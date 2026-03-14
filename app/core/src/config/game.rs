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
const DEFAULT_MAX_WEAPON_LEVEL: u8 = 8;
const DEFAULT_MAX_PASSIVE_LEVEL: u8 = 5;
const DEFAULT_MAX_WEAPONS: usize = 6;
const DEFAULT_MAX_PASSIVES: usize = 6;

// --- XP / levelling ---
const DEFAULT_XP_LEVEL_BASE: u32 = 20;
const DEFAULT_XP_LEVEL_MULTIPLIER: f32 = 1.2;
const DEFAULT_CHOICE_COUNT: usize = 3;
const DEFAULT_LUCK_BONUS_CHOICE_THRESHOLD: f32 = 1.5;

// --- spatial grid ---
const DEFAULT_SPATIAL_GRID_CELL_SIZE: f32 = 64.0;

// --- treasure chests ---
const DEFAULT_TREASURE_RADIUS: f32 = 20.0;
const DEFAULT_TREASURE_GOLD: u32 = 50;
const DEFAULT_TREASURE_HP_RECOVERY_PCT: f32 = 0.3;
const DEFAULT_TREASURE_GLOW_DISTANCE: f32 = 150.0;
const DEFAULT_TREASURE_SPAWN_FLASH_DURATION: f32 = 0.35;

// --- boss and game timing ---
const DEFAULT_BOSS_SPAWN_TIME: f32 = 1800.0;
const DEFAULT_TREASURE_SPAWN_INTERVAL: f32 = 180.0;
const DEFAULT_CAMERA_LERP_SPEED: f32 = 10.0;

// --- meta shop costs ---
const DEFAULT_SHOP_UPGRADE_COST_HP: u32 = 300;
const DEFAULT_SHOP_UPGRADE_COST_SPEED: u32 = 300;
const DEFAULT_SHOP_UPGRADE_COST_DAMAGE: u32 = 300;
const DEFAULT_SHOP_UPGRADE_COST_XP: u32 = 300;
const DEFAULT_SHOP_UPGRADE_COST_WEAPON: u32 = 500;

// --- meta upgrade stat bonuses ---
/// Flat HP bonus per BonusHp purchase (same magnitude as HollowHeart per level).
const DEFAULT_META_UPGRADE_HP_BONUS: f32 = 20.0;
/// Flat speed bonus (px/s) per BonusSpeed purchase (same magnitude as Wings per level).
const DEFAULT_META_UPGRADE_SPEED_BONUS: f32 = 20.0;
/// Damage multiplier added per BonusDamage purchase (same magnitude as Spinach per level).
const DEFAULT_META_UPGRADE_DAMAGE_BONUS: f32 = 0.1;
/// XP multiplier added per BonusXp purchase (10 % more XP per upgrade).
const DEFAULT_META_UPGRADE_XP_BONUS: f32 = 0.1;

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

/// Deserialization mirror of [`GameConfig`] — every field is `Option<T>` so
/// RON files with missing fields still load and emit a `warn!` instead of failing.
#[derive(Deserialize, Default)]
#[serde(default, rename = "GameConfig")]
pub(super) struct GameConfigPartial {
    pub window_width: Option<u32>,
    pub window_height: Option<u32>,
    pub max_weapons: Option<usize>,
    pub max_passives: Option<usize>,
    pub max_weapon_level: Option<u8>,
    pub max_passive_level: Option<u8>,
    pub boss_spawn_time: Option<f32>,
    pub treasure_spawn_interval: Option<f32>,
    pub treasure_radius: Option<f32>,
    pub treasure_gold_reward: Option<u32>,
    pub treasure_hp_recovery_pct: Option<f32>,
    pub treasure_glow_distance: Option<f32>,
    pub treasure_spawn_flash_duration: Option<f32>,
    pub xp_level_base: Option<u32>,
    pub xp_level_multiplier: Option<f32>,
    pub level_up_choice_count: Option<usize>,
    pub luck_bonus_choice_threshold: Option<f32>,
    pub camera_lerp_speed: Option<f32>,
    pub spatial_grid_cell_size: Option<f32>,
    pub base_projectile_speed: Option<f32>,
    pub base_projectile_lifetime: Option<f32>,
    pub boss_phase2_hp_threshold: Option<f32>,
    pub boss_phase3_hp_threshold: Option<f32>,
    pub boss_phase2_speed_multiplier: Option<f32>,
    pub mini_death_spawn_count: Option<usize>,
    pub mini_death_spawn_radius: Option<f32>,
    pub boss_phase3_speed_multiplier: Option<f32>,
    pub mini_death_spawn_count_phase3: Option<usize>,
    pub boss_scythe_interval: Option<f32>,
    pub boss_scythe_speed: Option<f32>,
    pub boss_scythe_lifetime: Option<f32>,
    pub boss_scythe_damage: Option<f32>,
    pub boss_scythe_radius: Option<f32>,
    pub shop_upgrade_cost_hp: Option<u32>,
    pub shop_upgrade_cost_speed: Option<u32>,
    pub shop_upgrade_cost_damage: Option<u32>,
    pub shop_upgrade_cost_xp: Option<u32>,
    pub shop_upgrade_cost_weapon: Option<u32>,
    pub meta_upgrade_hp_bonus: Option<f32>,
    pub meta_upgrade_speed_bonus: Option<f32>,
    pub meta_upgrade_damage_bonus: Option<f32>,
    pub meta_upgrade_xp_bonus: Option<f32>,
}

// ---------------------------------------------------------------------------
// Fallback constants for game fields without existing DEFAULT_* constants
// ---------------------------------------------------------------------------

const DEFAULT_WINDOW_WIDTH: u32 = 1280;
const DEFAULT_WINDOW_HEIGHT: u32 = 720;
const DEFAULT_BASE_PROJECTILE_SPEED: f32 = 300.0;
const DEFAULT_BASE_PROJECTILE_LIFETIME: f32 = 5.0;
const DEFAULT_BOSS_PHASE2_HP_THRESHOLD: f32 = 0.6;
const DEFAULT_BOSS_PHASE3_HP_THRESHOLD: f32 = 0.3;
const DEFAULT_BOSS_PHASE2_SPEED_MULTIPLIER: f32 = 1.5;
const DEFAULT_MINI_DEATH_SPAWN_COUNT: usize = 3;
const DEFAULT_MINI_DEATH_SPAWN_RADIUS: f32 = 80.0;
const DEFAULT_BOSS_PHASE3_SPEED_MULTIPLIER: f32 = 2.0;
const DEFAULT_MINI_DEATH_SPAWN_COUNT_PHASE3: usize = 5;
const DEFAULT_BOSS_SCYTHE_INTERVAL: f32 = 3.0;
const DEFAULT_BOSS_SCYTHE_SPEED: f32 = 250.0;
const DEFAULT_BOSS_SCYTHE_LIFETIME: f32 = 8.0;
const DEFAULT_BOSS_SCYTHE_DAMAGE: f32 = 80.0;
const DEFAULT_BOSS_SCYTHE_RADIUS: f32 = 15.0;

/// Game-wide rules and settings, loaded from `assets/config/game.ron`.
///
/// Covers viewport size, inventory caps, XP levelling curve, camera feel,
/// and spatial-grid tuning. Systems that read via [`GameParams`] pick up
/// hot-reloaded values automatically on the next frame.
#[derive(Asset, TypePath, Debug, Clone)]
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

impl From<GameConfigPartial> for GameConfig {
    fn from(p: GameConfigPartial) -> Self {
        GameConfig {
            window_width: p.window_width.unwrap_or_else(|| {
                warn!(
                    "game.ron: `window_width` missing → using default {DEFAULT_WINDOW_WIDTH}"
                );
                DEFAULT_WINDOW_WIDTH
            }),
            window_height: p.window_height.unwrap_or_else(|| {
                warn!(
                    "game.ron: `window_height` missing → using default {DEFAULT_WINDOW_HEIGHT}"
                );
                DEFAULT_WINDOW_HEIGHT
            }),
            max_weapons: p.max_weapons.unwrap_or_else(|| {
                warn!(
                    "game.ron: `max_weapons` missing → using default {DEFAULT_MAX_WEAPONS}"
                );
                DEFAULT_MAX_WEAPONS
            }),
            max_passives: p.max_passives.unwrap_or_else(|| {
                warn!(
                    "game.ron: `max_passives` missing → using default {DEFAULT_MAX_PASSIVES}"
                );
                DEFAULT_MAX_PASSIVES
            }),
            max_weapon_level: p.max_weapon_level.unwrap_or_else(|| {
                warn!(
                    "game.ron: `max_weapon_level` missing → using default {DEFAULT_MAX_WEAPON_LEVEL}"
                );
                DEFAULT_MAX_WEAPON_LEVEL
            }),
            max_passive_level: p.max_passive_level.unwrap_or_else(|| {
                warn!(
                    "game.ron: `max_passive_level` missing → using default {DEFAULT_MAX_PASSIVE_LEVEL}"
                );
                DEFAULT_MAX_PASSIVE_LEVEL
            }),
            boss_spawn_time: p.boss_spawn_time.unwrap_or_else(|| {
                warn!(
                    "game.ron: `boss_spawn_time` missing → using default {DEFAULT_BOSS_SPAWN_TIME}"
                );
                DEFAULT_BOSS_SPAWN_TIME
            }),
            treasure_spawn_interval: p.treasure_spawn_interval.unwrap_or_else(|| {
                warn!(
                    "game.ron: `treasure_spawn_interval` missing → using default {DEFAULT_TREASURE_SPAWN_INTERVAL}"
                );
                DEFAULT_TREASURE_SPAWN_INTERVAL
            }),
            treasure_radius: p.treasure_radius.unwrap_or_else(|| {
                warn!(
                    "game.ron: `treasure_radius` missing → using default {DEFAULT_TREASURE_RADIUS}"
                );
                DEFAULT_TREASURE_RADIUS
            }),
            treasure_gold_reward: p.treasure_gold_reward.unwrap_or_else(|| {
                warn!(
                    "game.ron: `treasure_gold_reward` missing → using default {DEFAULT_TREASURE_GOLD}"
                );
                DEFAULT_TREASURE_GOLD
            }),
            treasure_hp_recovery_pct: p.treasure_hp_recovery_pct.unwrap_or_else(|| {
                warn!(
                    "game.ron: `treasure_hp_recovery_pct` missing → using default {DEFAULT_TREASURE_HP_RECOVERY_PCT}"
                );
                DEFAULT_TREASURE_HP_RECOVERY_PCT
            }),
            treasure_glow_distance: p.treasure_glow_distance.unwrap_or_else(|| {
                warn!(
                    "game.ron: `treasure_glow_distance` missing → using default {DEFAULT_TREASURE_GLOW_DISTANCE}"
                );
                DEFAULT_TREASURE_GLOW_DISTANCE
            }),
            treasure_spawn_flash_duration: p.treasure_spawn_flash_duration.unwrap_or_else(|| {
                warn!(
                    "game.ron: `treasure_spawn_flash_duration` missing → using default {DEFAULT_TREASURE_SPAWN_FLASH_DURATION}"
                );
                DEFAULT_TREASURE_SPAWN_FLASH_DURATION
            }),
            xp_level_base: p.xp_level_base.unwrap_or_else(|| {
                warn!(
                    "game.ron: `xp_level_base` missing → using default {DEFAULT_XP_LEVEL_BASE}"
                );
                DEFAULT_XP_LEVEL_BASE
            }),
            xp_level_multiplier: p.xp_level_multiplier.unwrap_or_else(|| {
                warn!(
                    "game.ron: `xp_level_multiplier` missing → using default {DEFAULT_XP_LEVEL_MULTIPLIER}"
                );
                DEFAULT_XP_LEVEL_MULTIPLIER
            }),
            level_up_choice_count: p.level_up_choice_count.unwrap_or_else(|| {
                warn!(
                    "game.ron: `level_up_choice_count` missing → using default {DEFAULT_CHOICE_COUNT}"
                );
                DEFAULT_CHOICE_COUNT
            }),
            luck_bonus_choice_threshold: p.luck_bonus_choice_threshold.unwrap_or_else(|| {
                warn!(
                    "game.ron: `luck_bonus_choice_threshold` missing → using default {DEFAULT_LUCK_BONUS_CHOICE_THRESHOLD}"
                );
                DEFAULT_LUCK_BONUS_CHOICE_THRESHOLD
            }),
            camera_lerp_speed: p.camera_lerp_speed.unwrap_or_else(|| {
                warn!(
                    "game.ron: `camera_lerp_speed` missing → using default {DEFAULT_CAMERA_LERP_SPEED}"
                );
                DEFAULT_CAMERA_LERP_SPEED
            }),
            spatial_grid_cell_size: p.spatial_grid_cell_size.unwrap_or_else(|| {
                warn!(
                    "game.ron: `spatial_grid_cell_size` missing → using default {DEFAULT_SPATIAL_GRID_CELL_SIZE}"
                );
                DEFAULT_SPATIAL_GRID_CELL_SIZE
            }),
            base_projectile_speed: p.base_projectile_speed.unwrap_or_else(|| {
                warn!(
                    "game.ron: `base_projectile_speed` missing → using default {DEFAULT_BASE_PROJECTILE_SPEED}"
                );
                DEFAULT_BASE_PROJECTILE_SPEED
            }),
            base_projectile_lifetime: p.base_projectile_lifetime.unwrap_or_else(|| {
                warn!(
                    "game.ron: `base_projectile_lifetime` missing → using default {DEFAULT_BASE_PROJECTILE_LIFETIME}"
                );
                DEFAULT_BASE_PROJECTILE_LIFETIME
            }),
            boss_phase2_hp_threshold: p.boss_phase2_hp_threshold.unwrap_or_else(|| {
                warn!(
                    "game.ron: `boss_phase2_hp_threshold` missing → using default {DEFAULT_BOSS_PHASE2_HP_THRESHOLD}"
                );
                DEFAULT_BOSS_PHASE2_HP_THRESHOLD
            }),
            boss_phase3_hp_threshold: p.boss_phase3_hp_threshold.unwrap_or_else(|| {
                warn!(
                    "game.ron: `boss_phase3_hp_threshold` missing → using default {DEFAULT_BOSS_PHASE3_HP_THRESHOLD}"
                );
                DEFAULT_BOSS_PHASE3_HP_THRESHOLD
            }),
            boss_phase2_speed_multiplier: p.boss_phase2_speed_multiplier.unwrap_or_else(|| {
                warn!(
                    "game.ron: `boss_phase2_speed_multiplier` missing → using default {DEFAULT_BOSS_PHASE2_SPEED_MULTIPLIER}"
                );
                DEFAULT_BOSS_PHASE2_SPEED_MULTIPLIER
            }),
            mini_death_spawn_count: p.mini_death_spawn_count.unwrap_or_else(|| {
                warn!(
                    "game.ron: `mini_death_spawn_count` missing → using default {DEFAULT_MINI_DEATH_SPAWN_COUNT}"
                );
                DEFAULT_MINI_DEATH_SPAWN_COUNT
            }),
            mini_death_spawn_radius: p.mini_death_spawn_radius.unwrap_or_else(|| {
                warn!(
                    "game.ron: `mini_death_spawn_radius` missing → using default {DEFAULT_MINI_DEATH_SPAWN_RADIUS}"
                );
                DEFAULT_MINI_DEATH_SPAWN_RADIUS
            }),
            boss_phase3_speed_multiplier: p.boss_phase3_speed_multiplier.unwrap_or_else(|| {
                warn!(
                    "game.ron: `boss_phase3_speed_multiplier` missing → using default {DEFAULT_BOSS_PHASE3_SPEED_MULTIPLIER}"
                );
                DEFAULT_BOSS_PHASE3_SPEED_MULTIPLIER
            }),
            mini_death_spawn_count_phase3: p.mini_death_spawn_count_phase3.unwrap_or_else(|| {
                warn!(
                    "game.ron: `mini_death_spawn_count_phase3` missing → using default {DEFAULT_MINI_DEATH_SPAWN_COUNT_PHASE3}"
                );
                DEFAULT_MINI_DEATH_SPAWN_COUNT_PHASE3
            }),
            boss_scythe_interval: p.boss_scythe_interval.unwrap_or_else(|| {
                warn!(
                    "game.ron: `boss_scythe_interval` missing → using default {DEFAULT_BOSS_SCYTHE_INTERVAL}"
                );
                DEFAULT_BOSS_SCYTHE_INTERVAL
            }),
            boss_scythe_speed: p.boss_scythe_speed.unwrap_or_else(|| {
                warn!(
                    "game.ron: `boss_scythe_speed` missing → using default {DEFAULT_BOSS_SCYTHE_SPEED}"
                );
                DEFAULT_BOSS_SCYTHE_SPEED
            }),
            boss_scythe_lifetime: p.boss_scythe_lifetime.unwrap_or_else(|| {
                warn!(
                    "game.ron: `boss_scythe_lifetime` missing → using default {DEFAULT_BOSS_SCYTHE_LIFETIME}"
                );
                DEFAULT_BOSS_SCYTHE_LIFETIME
            }),
            boss_scythe_damage: p.boss_scythe_damage.unwrap_or_else(|| {
                warn!(
                    "game.ron: `boss_scythe_damage` missing → using default {DEFAULT_BOSS_SCYTHE_DAMAGE}"
                );
                DEFAULT_BOSS_SCYTHE_DAMAGE
            }),
            boss_scythe_radius: p.boss_scythe_radius.unwrap_or_else(|| {
                warn!(
                    "game.ron: `boss_scythe_radius` missing → using default {DEFAULT_BOSS_SCYTHE_RADIUS}"
                );
                DEFAULT_BOSS_SCYTHE_RADIUS
            }),
            shop_upgrade_cost_hp: p.shop_upgrade_cost_hp.unwrap_or_else(|| {
                warn!(
                    "game.ron: `shop_upgrade_cost_hp` missing → using default {DEFAULT_SHOP_UPGRADE_COST_HP}"
                );
                DEFAULT_SHOP_UPGRADE_COST_HP
            }),
            shop_upgrade_cost_speed: p.shop_upgrade_cost_speed.unwrap_or_else(|| {
                warn!(
                    "game.ron: `shop_upgrade_cost_speed` missing → using default {DEFAULT_SHOP_UPGRADE_COST_SPEED}"
                );
                DEFAULT_SHOP_UPGRADE_COST_SPEED
            }),
            shop_upgrade_cost_damage: p.shop_upgrade_cost_damage.unwrap_or_else(|| {
                warn!(
                    "game.ron: `shop_upgrade_cost_damage` missing → using default {DEFAULT_SHOP_UPGRADE_COST_DAMAGE}"
                );
                DEFAULT_SHOP_UPGRADE_COST_DAMAGE
            }),
            shop_upgrade_cost_xp: p.shop_upgrade_cost_xp.unwrap_or_else(|| {
                warn!(
                    "game.ron: `shop_upgrade_cost_xp` missing → using default {DEFAULT_SHOP_UPGRADE_COST_XP}"
                );
                DEFAULT_SHOP_UPGRADE_COST_XP
            }),
            shop_upgrade_cost_weapon: p.shop_upgrade_cost_weapon.unwrap_or_else(|| {
                warn!(
                    "game.ron: `shop_upgrade_cost_weapon` missing → using default {DEFAULT_SHOP_UPGRADE_COST_WEAPON}"
                );
                DEFAULT_SHOP_UPGRADE_COST_WEAPON
            }),
            meta_upgrade_hp_bonus: p.meta_upgrade_hp_bonus.unwrap_or_else(|| {
                warn!(
                    "game.ron: `meta_upgrade_hp_bonus` missing → using default {DEFAULT_META_UPGRADE_HP_BONUS}"
                );
                DEFAULT_META_UPGRADE_HP_BONUS
            }),
            meta_upgrade_speed_bonus: p.meta_upgrade_speed_bonus.unwrap_or_else(|| {
                warn!(
                    "game.ron: `meta_upgrade_speed_bonus` missing → using default {DEFAULT_META_UPGRADE_SPEED_BONUS}"
                );
                DEFAULT_META_UPGRADE_SPEED_BONUS
            }),
            meta_upgrade_damage_bonus: p.meta_upgrade_damage_bonus.unwrap_or_else(|| {
                warn!(
                    "game.ron: `meta_upgrade_damage_bonus` missing → using default {DEFAULT_META_UPGRADE_DAMAGE_BONUS}"
                );
                DEFAULT_META_UPGRADE_DAMAGE_BONUS
            }),
            meta_upgrade_xp_bonus: p.meta_upgrade_xp_bonus.unwrap_or_else(|| {
                warn!(
                    "game.ron: `meta_upgrade_xp_bonus` missing → using default {DEFAULT_META_UPGRADE_XP_BONUS}"
                );
                DEFAULT_META_UPGRADE_XP_BONUS
            }),
        }
    }
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

    pub fn boss_spawn_time(&self) -> f32 {
        self.get()
            .map(|c| c.boss_spawn_time)
            .unwrap_or(DEFAULT_BOSS_SPAWN_TIME)
    }

    pub fn treasure_spawn_interval(&self) -> f32 {
        self.get()
            .map(|c| c.treasure_spawn_interval)
            .unwrap_or(DEFAULT_TREASURE_SPAWN_INTERVAL)
    }

    pub fn camera_lerp_speed(&self) -> f32 {
        self.get()
            .map(|c| c.camera_lerp_speed)
            .unwrap_or(DEFAULT_CAMERA_LERP_SPEED)
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
        let partial: GameConfigPartial = ron::Options::default().with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME).from_str(ron_data).unwrap();
        let config = GameConfig::from(partial);
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
