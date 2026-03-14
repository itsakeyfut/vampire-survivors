//! Enemy configuration loaded from `assets/config/enemy.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

use crate::types::EnemyType;

// ---------------------------------------------------------------------------
// Fallback constants (used while enemy.ron is still loading)
// ---------------------------------------------------------------------------

/// Base enemy spawn interval in seconds when difficulty multiplier is 1.0.
const DEFAULT_ENEMY_SPAWN_BASE_INTERVAL: f32 = 0.5;

/// Hard cap for the difficulty multiplier.
const DEFAULT_DIFFICULTY_MAX: f32 = 10.0;

/// Maximum number of enemies that can exist simultaneously.
const DEFAULT_MAX_ENEMY_COUNT: usize = 500;

/// Distance from the player at which enemies are culled.
const DEFAULT_CULL_DISTANCE: f32 = 2000.0;

/// Extra pixels beyond the viewport edge at which enemies spawn.
const DEFAULT_SPAWN_MARGIN: f32 = 60.0;

/// Seconds before Zombies appear in the spawn table.
const DEFAULT_ZOMBIE_UNLOCK_SECS: f32 = 300.0;

/// Seconds before Ghosts appear in the spawn table.
const DEFAULT_GHOST_UNLOCK_SECS: f32 = 600.0;

/// Seconds before Demons appear in the spawn table.
const DEFAULT_DEMON_UNLOCK_SECS: f32 = 900.0;

/// Seconds before Medusas appear in the spawn table.
const DEFAULT_MEDUSA_UNLOCK_SECS: f32 = 1200.0;

/// Seconds before Dragons appear in the spawn table.
const DEFAULT_DRAGON_UNLOCK_SECS: f32 = 1500.0;

/// Seconds between mini-boss spawns.
const DEFAULT_MINI_BOSS_INTERVAL: f32 = 180.0;

// ---------------------------------------------------------------------------
// Medusa AI behavior config
// ---------------------------------------------------------------------------

/// Behavior parameters for the Medusa ranged enemy, deserialized from RON.
#[derive(Debug, Clone)]
pub struct MedusaBehaviorConfig {
    pub keep_min_dist: f32,
    pub keep_max_dist: f32,
    pub attack_interval: f32,
    pub projectile_speed: f32,
    pub projectile_lifetime: f32,
    pub projectile_radius: f32,
}

/// Deserialization mirror of [`MedusaBehaviorConfig`].
#[derive(Deserialize, Default)]
#[serde(default)]
pub(super) struct MedusaBehaviorConfigPartial {
    pub keep_min_dist: Option<f32>,
    pub keep_max_dist: Option<f32>,
    pub attack_interval: Option<f32>,
    pub projectile_speed: Option<f32>,
    pub projectile_lifetime: Option<f32>,
    pub projectile_radius: Option<f32>,
}

impl From<MedusaBehaviorConfigPartial> for MedusaBehaviorConfig {
    fn from(p: MedusaBehaviorConfigPartial) -> Self {
        MedusaBehaviorConfig {
            keep_min_dist: p.keep_min_dist.unwrap_or_else(|| {
                warn!("enemy.ron: `medusa_behavior.keep_min_dist` missing → using built-in baseline");
                150.0
            }),
            keep_max_dist: p.keep_max_dist.unwrap_or_else(|| {
                warn!("enemy.ron: `medusa_behavior.keep_max_dist` missing → using built-in baseline");
                250.0
            }),
            attack_interval: p.attack_interval.unwrap_or_else(|| {
                warn!("enemy.ron: `medusa_behavior.attack_interval` missing → using built-in baseline");
                2.0
            }),
            projectile_speed: p.projectile_speed.unwrap_or_else(|| {
                warn!("enemy.ron: `medusa_behavior.projectile_speed` missing → using built-in baseline");
                180.0
            }),
            projectile_lifetime: p.projectile_lifetime.unwrap_or_else(|| {
                warn!("enemy.ron: `medusa_behavior.projectile_lifetime` missing → using built-in baseline");
                5.0
            }),
            projectile_radius: p.projectile_radius.unwrap_or_else(|| {
                warn!("enemy.ron: `medusa_behavior.projectile_radius` missing → using built-in baseline");
                5.0
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// Dragon AI behavior config
// ---------------------------------------------------------------------------

/// Behavior parameters for the Dragon enemy, deserialized from RON.
#[derive(Debug, Clone)]
pub struct DragonBehaviorConfig {
    pub attack_interval: f32,
    pub fireball_speed: f32,
    pub fireball_lifetime: f32,
    pub fireball_radius: f32,
}

/// Deserialization mirror of [`DragonBehaviorConfig`].
#[derive(Deserialize, Default)]
#[serde(default)]
pub(super) struct DragonBehaviorConfigPartial {
    pub attack_interval: Option<f32>,
    pub fireball_speed: Option<f32>,
    pub fireball_lifetime: Option<f32>,
    pub fireball_radius: Option<f32>,
}

impl From<DragonBehaviorConfigPartial> for DragonBehaviorConfig {
    fn from(p: DragonBehaviorConfigPartial) -> Self {
        DragonBehaviorConfig {
            attack_interval: p.attack_interval.unwrap_or_else(|| {
                warn!("enemy.ron: `dragon_behavior.attack_interval` missing → using built-in baseline");
                3.0
            }),
            fireball_speed: p.fireball_speed.unwrap_or_else(|| {
                warn!("enemy.ron: `dragon_behavior.fireball_speed` missing → using built-in baseline");
                200.0
            }),
            fireball_lifetime: p.fireball_lifetime.unwrap_or_else(|| {
                warn!("enemy.ron: `dragon_behavior.fireball_lifetime` missing → using built-in baseline");
                6.0
            }),
            fireball_radius: p.fireball_radius.unwrap_or_else(|| {
                warn!("enemy.ron: `dragon_behavior.fireball_radius` missing → using built-in baseline");
                7.0
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// Per-enemy stats entry
// ---------------------------------------------------------------------------

/// Base stats for a single enemy type, deserialized from RON.
#[derive(Debug, Clone)]
pub struct EnemyStatsEntry {
    pub base_hp: f32,
    pub speed: f32,
    pub damage: f32,
    pub xp_value: u32,
    pub gold_chance: f32,
    pub collider_radius: f32,
    /// Relative spawn weight used for weighted-random selection.
    ///
    /// Higher values = more frequent.  Does not affect unlock timing.
    /// Typical range: 0.3 (Dragon) – 1.0 (Bat/Skeleton).
    pub spawn_weight: f32,
}

/// Deserialization mirror of [`EnemyStatsEntry`].
#[derive(Deserialize, Default)]
#[serde(default)]
pub(super) struct EnemyStatsEntryPartial {
    pub base_hp: Option<f32>,
    pub speed: Option<f32>,
    pub damage: Option<f32>,
    pub xp_value: Option<u32>,
    pub gold_chance: Option<f32>,
    pub collider_radius: Option<f32>,
    pub spawn_weight: Option<f32>,
}

impl EnemyStatsEntryPartial {
    fn into_entry(self, field_prefix: &str) -> EnemyStatsEntry {
        EnemyStatsEntry {
            base_hp: self.base_hp.unwrap_or_else(|| {
                warn!("enemy.ron: `{field_prefix}.base_hp` missing → using built-in baseline");
                1.0
            }),
            speed: self.speed.unwrap_or_else(|| {
                warn!("enemy.ron: `{field_prefix}.speed` missing → using built-in baseline");
                50.0
            }),
            damage: self.damage.unwrap_or_else(|| {
                warn!("enemy.ron: `{field_prefix}.damage` missing → using built-in baseline");
                1.0
            }),
            xp_value: self.xp_value.unwrap_or_else(|| {
                warn!("enemy.ron: `{field_prefix}.xp_value` missing → using built-in baseline");
                1
            }),
            gold_chance: self.gold_chance.unwrap_or_else(|| {
                warn!("enemy.ron: `{field_prefix}.gold_chance` missing → using built-in baseline");
                0.0
            }),
            collider_radius: self.collider_radius.unwrap_or_else(|| {
                warn!("enemy.ron: `{field_prefix}.collider_radius` missing → using built-in baseline");
                8.0
            }),
            spawn_weight: self.spawn_weight.unwrap_or_else(|| {
                warn!("enemy.ron: `{field_prefix}.spawn_weight` missing → using built-in baseline");
                0.0
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// Asset type
// ---------------------------------------------------------------------------

/// Deserialization mirror of [`EnemyConfig`] — every field is `Option<T>` so
/// RON files with missing fields still load and emit a `warn!` instead of failing.
#[derive(Deserialize, Default)]
#[serde(default, rename = "EnemyConfig")]
pub(super) struct EnemyConfigPartial {
    pub bat: Option<EnemyStatsEntryPartial>,
    pub skeleton: Option<EnemyStatsEntryPartial>,
    pub zombie: Option<EnemyStatsEntryPartial>,
    pub ghost: Option<EnemyStatsEntryPartial>,
    pub demon: Option<EnemyStatsEntryPartial>,
    pub medusa: Option<EnemyStatsEntryPartial>,
    pub dragon: Option<EnemyStatsEntryPartial>,
    pub boss_death: Option<EnemyStatsEntryPartial>,
    pub mini_death: Option<EnemyStatsEntryPartial>,
    pub mini_boss: Option<EnemyStatsEntryPartial>,
    pub spawn_base_interval: Option<f32>,
    pub max_count: Option<usize>,
    pub cull_distance: Option<f32>,
    pub difficulty_max: Option<f32>,
    pub spawn_margin: Option<f32>,
    pub zombie_unlock_secs: Option<f32>,
    pub ghost_unlock_secs: Option<f32>,
    pub demon_unlock_secs: Option<f32>,
    pub medusa_unlock_secs: Option<f32>,
    pub dragon_unlock_secs: Option<f32>,
    pub mini_boss_interval: Option<f32>,
    pub medusa_behavior: Option<MedusaBehaviorConfigPartial>,
    pub dragon_behavior: Option<DragonBehaviorConfigPartial>,
}

/// Full enemy configuration, loaded from `assets/config/enemy.ron`.
///
/// Contains per-enemy stat blocks and global spawn/difficulty parameters.
/// Hot-reloading this file affects enemies spawned after the reload;
/// existing enemies keep their current stats.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct EnemyConfig {
    pub bat: EnemyStatsEntry,
    pub skeleton: EnemyStatsEntry,
    pub zombie: EnemyStatsEntry,
    pub ghost: EnemyStatsEntry,
    pub demon: EnemyStatsEntry,
    pub medusa: EnemyStatsEntry,
    pub dragon: EnemyStatsEntry,
    pub boss_death: EnemyStatsEntry,
    pub mini_death: EnemyStatsEntry,
    /// Mini-boss that spawns every 3 minutes and drops a treasure chest on defeat.
    pub mini_boss: EnemyStatsEntry,
    // Spawning parameters
    pub spawn_base_interval: f32,
    pub max_count: usize,
    pub cull_distance: f32,
    pub difficulty_max: f32,
    /// Extra pixels beyond the half-viewport edge at which enemies appear.
    pub spawn_margin: f32,
    /// Seconds into the run before Zombies are added to the spawn table.
    pub zombie_unlock_secs: f32,
    /// Seconds into the run before Ghosts are added to the spawn table.
    pub ghost_unlock_secs: f32,
    /// Seconds into the run before Demons are added to the spawn table.
    pub demon_unlock_secs: f32,
    /// Seconds into the run before Medusas are added to the spawn table.
    pub medusa_unlock_secs: f32,
    /// Seconds into the run before Dragons are added to the spawn table.
    pub dragon_unlock_secs: f32,
    /// Seconds between each mini-boss spawn (default 180 = 3 minutes).
    pub mini_boss_interval: f32,
    /// Medusa-specific AI and projectile behavior parameters.
    pub medusa_behavior: MedusaBehaviorConfig,
    /// Dragon-specific fireball behavior parameters.
    pub dragon_behavior: DragonBehaviorConfig,
}

impl From<EnemyConfigPartial> for EnemyConfig {
    fn from(p: EnemyConfigPartial) -> Self {
        EnemyConfig {
            bat: p
                .bat
                .unwrap_or_default()
                .into_entry("bat"),
            skeleton: p
                .skeleton
                .unwrap_or_default()
                .into_entry("skeleton"),
            zombie: p
                .zombie
                .unwrap_or_default()
                .into_entry("zombie"),
            ghost: p
                .ghost
                .unwrap_or_default()
                .into_entry("ghost"),
            demon: p
                .demon
                .unwrap_or_default()
                .into_entry("demon"),
            medusa: p
                .medusa
                .unwrap_or_default()
                .into_entry("medusa"),
            dragon: p
                .dragon
                .unwrap_or_default()
                .into_entry("dragon"),
            boss_death: p
                .boss_death
                .unwrap_or_default()
                .into_entry("boss_death"),
            mini_death: p
                .mini_death
                .unwrap_or_default()
                .into_entry("mini_death"),
            mini_boss: p
                .mini_boss
                .unwrap_or_default()
                .into_entry("mini_boss"),
            spawn_base_interval: p.spawn_base_interval.unwrap_or_else(|| {
                warn!(
                    "enemy.ron: `spawn_base_interval` missing → using default {DEFAULT_ENEMY_SPAWN_BASE_INTERVAL}"
                );
                DEFAULT_ENEMY_SPAWN_BASE_INTERVAL
            }),
            max_count: p.max_count.unwrap_or_else(|| {
                warn!(
                    "enemy.ron: `max_count` missing → using default {DEFAULT_MAX_ENEMY_COUNT}"
                );
                DEFAULT_MAX_ENEMY_COUNT
            }),
            cull_distance: p.cull_distance.unwrap_or_else(|| {
                warn!(
                    "enemy.ron: `cull_distance` missing → using default {DEFAULT_CULL_DISTANCE}"
                );
                DEFAULT_CULL_DISTANCE
            }),
            difficulty_max: p.difficulty_max.unwrap_or_else(|| {
                warn!(
                    "enemy.ron: `difficulty_max` missing → using default {DEFAULT_DIFFICULTY_MAX}"
                );
                DEFAULT_DIFFICULTY_MAX
            }),
            spawn_margin: p.spawn_margin.unwrap_or_else(|| {
                warn!(
                    "enemy.ron: `spawn_margin` missing → using default {DEFAULT_SPAWN_MARGIN}"
                );
                DEFAULT_SPAWN_MARGIN
            }),
            zombie_unlock_secs: p.zombie_unlock_secs.unwrap_or_else(|| {
                warn!(
                    "enemy.ron: `zombie_unlock_secs` missing → using default {DEFAULT_ZOMBIE_UNLOCK_SECS}"
                );
                DEFAULT_ZOMBIE_UNLOCK_SECS
            }),
            ghost_unlock_secs: p.ghost_unlock_secs.unwrap_or_else(|| {
                warn!(
                    "enemy.ron: `ghost_unlock_secs` missing → using default {DEFAULT_GHOST_UNLOCK_SECS}"
                );
                DEFAULT_GHOST_UNLOCK_SECS
            }),
            demon_unlock_secs: p.demon_unlock_secs.unwrap_or_else(|| {
                warn!(
                    "enemy.ron: `demon_unlock_secs` missing → using default {DEFAULT_DEMON_UNLOCK_SECS}"
                );
                DEFAULT_DEMON_UNLOCK_SECS
            }),
            medusa_unlock_secs: p.medusa_unlock_secs.unwrap_or_else(|| {
                warn!(
                    "enemy.ron: `medusa_unlock_secs` missing → using default {DEFAULT_MEDUSA_UNLOCK_SECS}"
                );
                DEFAULT_MEDUSA_UNLOCK_SECS
            }),
            dragon_unlock_secs: p.dragon_unlock_secs.unwrap_or_else(|| {
                warn!(
                    "enemy.ron: `dragon_unlock_secs` missing → using default {DEFAULT_DRAGON_UNLOCK_SECS}"
                );
                DEFAULT_DRAGON_UNLOCK_SECS
            }),
            mini_boss_interval: p.mini_boss_interval.unwrap_or_else(|| {
                warn!(
                    "enemy.ron: `mini_boss_interval` missing → using default {DEFAULT_MINI_BOSS_INTERVAL}"
                );
                DEFAULT_MINI_BOSS_INTERVAL
            }),
            medusa_behavior: MedusaBehaviorConfig::from(
                p.medusa_behavior.unwrap_or_else(|| {
                    warn!("enemy.ron: `medusa_behavior` block missing → using built-in baseline");
                    MedusaBehaviorConfigPartial::default()
                }),
            ),
            dragon_behavior: DragonBehaviorConfig::from(
                p.dragon_behavior.unwrap_or_else(|| {
                    warn!("enemy.ron: `dragon_behavior` block missing → using built-in baseline");
                    DragonBehaviorConfigPartial::default()
                }),
            ),
        }
    }
}

impl EnemyConfig {
    /// Returns the stat block for a given [`EnemyType`].
    pub fn stats_for(&self, enemy_type: EnemyType) -> &EnemyStatsEntry {
        match enemy_type {
            EnemyType::Bat => &self.bat,
            EnemyType::Skeleton => &self.skeleton,
            EnemyType::Zombie => &self.zombie,
            EnemyType::Ghost => &self.ghost,
            EnemyType::Demon => &self.demon,
            EnemyType::Medusa => &self.medusa,
            EnemyType::Dragon => &self.dragon,
            EnemyType::BossDeath => &self.boss_death,
            EnemyType::MiniDeath => &self.mini_death,
            EnemyType::MiniBoss => &self.mini_boss,
        }
    }
}

/// Resource holding the handle to the loaded enemy configuration.
#[derive(Resource)]
pub struct EnemyConfigHandle(pub Handle<EnemyConfig>);

// ---------------------------------------------------------------------------
// SystemParam bundle
// ---------------------------------------------------------------------------

/// SystemParam bundle for accessing [`EnemyConfig`].
///
/// Returns `None` while the asset is still loading or when
/// [`GameConfigPlugin`] has not been registered (e.g. in unit tests).
/// Call `.get()` to obtain `Option<&EnemyConfig>`.
///
/// [`GameConfigPlugin`]: crate::config::GameConfigPlugin
#[derive(SystemParam)]
pub struct EnemyParams<'w> {
    handle: Option<Res<'w, EnemyConfigHandle>>,
    assets: Option<Res<'w, Assets<EnemyConfig>>>,
}

impl<'w> EnemyParams<'w> {
    /// Returns the currently loaded [`EnemyConfig`], or `None` while loading.
    pub fn get(&self) -> Option<&EnemyConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }

    /// Returns the base spawn interval, falling back to the default constant.
    pub fn spawn_base_interval(&self) -> f32 {
        self.get()
            .map(|c| c.spawn_base_interval)
            .unwrap_or(DEFAULT_ENEMY_SPAWN_BASE_INTERVAL)
    }

    pub fn difficulty_max(&self) -> f32 {
        self.get()
            .map(|c| c.difficulty_max)
            .unwrap_or(DEFAULT_DIFFICULTY_MAX)
    }

    pub fn max_count(&self) -> usize {
        self.get()
            .map(|c| c.max_count)
            .unwrap_or(DEFAULT_MAX_ENEMY_COUNT)
    }

    pub fn cull_distance(&self) -> f32 {
        self.get()
            .map(|c| c.cull_distance)
            .unwrap_or(DEFAULT_CULL_DISTANCE)
    }

    pub fn spawn_margin(&self) -> f32 {
        self.get()
            .map(|c| c.spawn_margin)
            .unwrap_or(DEFAULT_SPAWN_MARGIN)
    }

    pub fn zombie_unlock_secs(&self) -> f32 {
        self.get()
            .map(|c| c.zombie_unlock_secs)
            .unwrap_or(DEFAULT_ZOMBIE_UNLOCK_SECS)
    }

    pub fn ghost_unlock_secs(&self) -> f32 {
        self.get()
            .map(|c| c.ghost_unlock_secs)
            .unwrap_or(DEFAULT_GHOST_UNLOCK_SECS)
    }

    pub fn demon_unlock_secs(&self) -> f32 {
        self.get()
            .map(|c| c.demon_unlock_secs)
            .unwrap_or(DEFAULT_DEMON_UNLOCK_SECS)
    }

    pub fn medusa_unlock_secs(&self) -> f32 {
        self.get()
            .map(|c| c.medusa_unlock_secs)
            .unwrap_or(DEFAULT_MEDUSA_UNLOCK_SECS)
    }

    pub fn dragon_unlock_secs(&self) -> f32 {
        self.get()
            .map(|c| c.dragon_unlock_secs)
            .unwrap_or(DEFAULT_DRAGON_UNLOCK_SECS)
    }

    pub fn mini_boss_interval(&self) -> f32 {
        self.get()
            .map(|c| c.mini_boss_interval)
            .unwrap_or(DEFAULT_MINI_BOSS_INTERVAL)
    }
}

// ---------------------------------------------------------------------------
// Hot-reload system
// ---------------------------------------------------------------------------

/// Handles hot-reloading of enemy configuration.
///
/// On `Modified`, logs the reload. Active enemies keep their current stats;
/// only enemies spawned after the reload use the updated values.
pub fn hot_reload_enemy_config(mut events: MessageReader<AssetEvent<EnemyConfig>>) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id: _ } => {
                info!("✅ Enemy config loaded");
            }
            AssetEvent::Modified { id: _ } => {
                info!("🔥 Hot-reloading enemy config! New enemies will use updated stats.");
            }
            AssetEvent::Removed { id: _ } => {
                warn!("⚠️ Enemy config removed");
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
    fn enemy_config_deserialization() {
        let ron_data = r#"
EnemyConfig(
    bat: (base_hp: 10.0, speed: 150.0, damage: 5.0, xp_value: 3, gold_chance: 0.05, collider_radius: 8.0, spawn_weight: 1.0),
    skeleton: (base_hp: 30.0, speed: 80.0, damage: 8.0, xp_value: 5, gold_chance: 0.08, collider_radius: 12.0, spawn_weight: 1.0),
    zombie: (base_hp: 60.0, speed: 60.0, damage: 12.0, xp_value: 8, gold_chance: 0.10, collider_radius: 14.0, spawn_weight: 0.8),
    ghost: (base_hp: 25.0, speed: 100.0, damage: 10.0, xp_value: 6, gold_chance: 0.08, collider_radius: 10.0, spawn_weight: 0.6),
    demon: (base_hp: 80.0, speed: 130.0, damage: 15.0, xp_value: 10, gold_chance: 0.12, collider_radius: 14.0, spawn_weight: 0.5),
    medusa: (base_hp: 60.0, speed: 60.0, damage: 12.0, xp_value: 8, gold_chance: 0.10, collider_radius: 12.0, spawn_weight: 0.4),
    dragon: (base_hp: 150.0, speed: 90.0, damage: 25.0, xp_value: 15, gold_chance: 0.15, collider_radius: 20.0, spawn_weight: 0.3),
    boss_death: (base_hp: 5000.0, speed: 30.0, damage: 50.0, xp_value: 500, gold_chance: 1.0, collider_radius: 30.0, spawn_weight: 0.0),
    mini_death: (base_hp: 800.0, speed: 80.0, damage: 30.0, xp_value: 50, gold_chance: 0.5, collider_radius: 20.0, spawn_weight: 0.0),
    mini_boss: (base_hp: 400.0, speed: 70.0, damage: 20.0, xp_value: 30, gold_chance: 0.0, collider_radius: 22.0, spawn_weight: 0.0),
    spawn_base_interval: 0.5,
    max_count: 500,
    cull_distance: 2000.0,
    difficulty_max: 10.0,
    spawn_margin: 60.0,
    zombie_unlock_secs: 300.0,
    ghost_unlock_secs: 600.0,
    demon_unlock_secs: 900.0,
    medusa_unlock_secs: 1200.0,
    dragon_unlock_secs: 1500.0,
    mini_boss_interval: 180.0,
    medusa_behavior: (
        keep_min_dist: 150.0,
        keep_max_dist: 250.0,
        attack_interval: 2.0,
        projectile_speed: 180.0,
        projectile_lifetime: 5.0,
        projectile_radius: 5.0,
    ),
    dragon_behavior: (
        attack_interval: 3.0,
        fireball_speed: 200.0,
        fireball_lifetime: 6.0,
        fireball_radius: 7.0,
    ),
)
"#;
        let partial: EnemyConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(ron_data)
            .unwrap();
        let config = EnemyConfig::from(partial);
        assert_eq!(config.bat.base_hp, 10.0);
        assert_eq!(config.bat.speed, 150.0);
        assert_eq!(config.bat.collider_radius, 8.0);
        assert_eq!(config.skeleton.base_hp, 30.0);
        assert_eq!(config.boss_death.base_hp, 5000.0);
        assert_eq!(config.boss_death.xp_value, 500);
        assert_eq!(config.mini_death.base_hp, 800.0);
        assert_eq!(config.mini_death.speed, 80.0);
        assert_eq!(config.spawn_base_interval, 0.5);
        assert_eq!(config.max_count, 500);
        assert_eq!(config.cull_distance, 2000.0);
        assert_eq!(config.difficulty_max, 10.0);
        assert_eq!(config.spawn_margin, 60.0);
        assert_eq!(config.zombie_unlock_secs, 300.0);
        assert_eq!(config.ghost_unlock_secs, 600.0);
        assert_eq!(config.demon_unlock_secs, 900.0);
        assert_eq!(config.medusa_unlock_secs, 1200.0);
        assert_eq!(config.dragon_unlock_secs, 1500.0);
        assert_eq!(config.mini_boss_interval, 180.0);
        assert_eq!(config.medusa_behavior.keep_min_dist, 150.0);
        assert_eq!(config.medusa_behavior.keep_max_dist, 250.0);
        assert_eq!(config.medusa_behavior.attack_interval, 2.0);
        assert_eq!(config.dragon_behavior.attack_interval, 3.0);
        assert_eq!(config.dragon_behavior.fireball_speed, 200.0);
        assert_eq!(config.bat.spawn_weight, 1.0);
        assert_eq!(config.dragon.spawn_weight, 0.3);
    }

    #[test]
    fn stats_for_returns_correct_entry() {
        let ron_data = r#"
EnemyConfig(
    bat: (base_hp: 10.0, speed: 150.0, damage: 5.0, xp_value: 3, gold_chance: 0.05, collider_radius: 8.0, spawn_weight: 1.0),
    skeleton: (base_hp: 30.0, speed: 80.0, damage: 8.0, xp_value: 5, gold_chance: 0.08, collider_radius: 12.0, spawn_weight: 1.0),
    zombie: (base_hp: 60.0, speed: 60.0, damage: 12.0, xp_value: 8, gold_chance: 0.10, collider_radius: 14.0, spawn_weight: 0.8),
    ghost: (base_hp: 25.0, speed: 100.0, damage: 10.0, xp_value: 6, gold_chance: 0.08, collider_radius: 10.0, spawn_weight: 0.6),
    demon: (base_hp: 80.0, speed: 130.0, damage: 15.0, xp_value: 10, gold_chance: 0.12, collider_radius: 14.0, spawn_weight: 0.5),
    medusa: (base_hp: 60.0, speed: 60.0, damage: 12.0, xp_value: 8, gold_chance: 0.10, collider_radius: 12.0, spawn_weight: 0.4),
    dragon: (base_hp: 150.0, speed: 90.0, damage: 25.0, xp_value: 15, gold_chance: 0.15, collider_radius: 20.0, spawn_weight: 0.3),
    boss_death: (base_hp: 5000.0, speed: 30.0, damage: 50.0, xp_value: 500, gold_chance: 1.0, collider_radius: 30.0, spawn_weight: 0.0),
    mini_death: (base_hp: 800.0, speed: 80.0, damage: 30.0, xp_value: 50, gold_chance: 0.5, collider_radius: 20.0, spawn_weight: 0.0),
    mini_boss: (base_hp: 400.0, speed: 70.0, damage: 20.0, xp_value: 30, gold_chance: 0.0, collider_radius: 22.0, spawn_weight: 0.0),
    spawn_base_interval: 0.5,
    max_count: 500,
    cull_distance: 2000.0,
    difficulty_max: 10.0,
    spawn_margin: 60.0,
    zombie_unlock_secs: 300.0,
    ghost_unlock_secs: 600.0,
    demon_unlock_secs: 900.0,
    medusa_unlock_secs: 1200.0,
    dragon_unlock_secs: 1500.0,
    mini_boss_interval: 180.0,
    medusa_behavior: (
        keep_min_dist: 150.0,
        keep_max_dist: 250.0,
        attack_interval: 2.0,
        projectile_speed: 180.0,
        projectile_lifetime: 5.0,
        projectile_radius: 5.0,
    ),
    dragon_behavior: (
        attack_interval: 3.0,
        fireball_speed: 200.0,
        fireball_lifetime: 6.0,
        fireball_radius: 7.0,
    ),
)
"#;
        let partial: EnemyConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(ron_data)
            .unwrap();
        let config = EnemyConfig::from(partial);
        assert_eq!(config.stats_for(EnemyType::Bat).base_hp, 10.0);
        assert_eq!(config.stats_for(EnemyType::Bat).collider_radius, 8.0);
        assert_eq!(config.stats_for(EnemyType::BossDeath).base_hp, 5000.0);
        assert_eq!(config.stats_for(EnemyType::BossDeath).collider_radius, 30.0);
        assert_eq!(config.stats_for(EnemyType::MiniDeath).base_hp, 800.0);
        assert_eq!(config.stats_for(EnemyType::MiniDeath).collider_radius, 20.0);
        assert_eq!(config.stats_for(EnemyType::MiniBoss).base_hp, 400.0);
        assert_eq!(config.stats_for(EnemyType::MiniBoss).gold_chance, 0.0);
    }
}
