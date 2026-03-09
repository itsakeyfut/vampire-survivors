//! Enemy configuration loaded from `assets/config/enemy.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

use crate::types::EnemyType;

// ---------------------------------------------------------------------------
// Medusa AI behavior config
// ---------------------------------------------------------------------------

/// Behavior parameters for the Medusa ranged enemy, deserialized from RON.
#[derive(Deserialize, Debug, Clone)]
pub struct MedusaBehaviorConfig {
    /// Minimum keep distance from the player (pixels).  Medusa moves away
    /// from the player when closer than this value.
    pub keep_min_dist: f32,
    /// Maximum keep distance from the player (pixels).  Medusa moves toward
    /// the player when farther than this value.
    pub keep_max_dist: f32,
    /// Seconds between petrification-projectile shots.
    pub attack_interval: f32,
    /// Speed of the fired projectile (pixels/second).
    pub projectile_speed: f32,
    /// Lifetime of the projectile before it despawns (seconds).
    pub projectile_lifetime: f32,
    /// Collider radius of the projectile (pixels).
    pub projectile_radius: f32,
}

// ---------------------------------------------------------------------------
// Dragon AI behavior config
// ---------------------------------------------------------------------------

/// Behavior parameters for the Dragon enemy, deserialized from RON.
#[derive(Deserialize, Debug, Clone)]
pub struct DragonBehaviorConfig {
    /// Seconds between fireball shots.
    pub attack_interval: f32,
    /// Speed of the fired fireball (pixels/second).
    pub fireball_speed: f32,
    /// Lifetime of the fireball before it despawns (seconds).
    pub fireball_lifetime: f32,
    /// Collider radius of the fireball (pixels).
    pub fireball_radius: f32,
}

// ---------------------------------------------------------------------------
// Per-enemy stats entry
// ---------------------------------------------------------------------------

/// Base stats for a single enemy type, deserialized from RON.
#[derive(Deserialize, Debug, Clone)]
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

// ---------------------------------------------------------------------------
// Asset type
// ---------------------------------------------------------------------------

/// Full enemy configuration, loaded from `assets/config/enemy.ron`.
///
/// Contains per-enemy stat blocks and global spawn/difficulty parameters.
/// Hot-reloading this file affects enemies spawned after the reload;
/// existing enemies keep their current stats.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
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
    /// Medusa-specific AI and projectile behavior parameters.
    pub medusa_behavior: MedusaBehaviorConfig,
    /// Dragon-specific fireball behavior parameters.
    pub dragon_behavior: DragonBehaviorConfig,
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
        let config: EnemyConfig = ron::de::from_str(ron_data).unwrap();
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
        let config: EnemyConfig = ron::de::from_str(ron_data).unwrap();
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
