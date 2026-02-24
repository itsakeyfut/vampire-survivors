//! Enemy configuration loaded from `assets/config/enemy.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

use crate::types::EnemyType;

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
    // Spawning parameters
    pub spawn_base_interval: f32,
    pub max_count: usize,
    pub cull_distance: f32,
    pub difficulty_max: f32,
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
    bat: (base_hp: 10.0, speed: 150.0, damage: 5.0, xp_value: 3, gold_chance: 0.05, collider_radius: 8.0),
    skeleton: (base_hp: 30.0, speed: 80.0, damage: 8.0, xp_value: 5, gold_chance: 0.08, collider_radius: 12.0),
    zombie: (base_hp: 80.0, speed: 40.0, damage: 12.0, xp_value: 8, gold_chance: 0.10, collider_radius: 14.0),
    ghost: (base_hp: 40.0, speed: 70.0, damage: 10.0, xp_value: 6, gold_chance: 0.08, collider_radius: 10.0),
    demon: (base_hp: 100.0, speed: 120.0, damage: 15.0, xp_value: 10, gold_chance: 0.12, collider_radius: 14.0),
    medusa: (base_hp: 60.0, speed: 60.0, damage: 12.0, xp_value: 8, gold_chance: 0.10, collider_radius: 12.0),
    dragon: (base_hp: 200.0, speed: 80.0, damage: 20.0, xp_value: 15, gold_chance: 0.15, collider_radius: 20.0),
    boss_death: (base_hp: 5000.0, speed: 30.0, damage: 50.0, xp_value: 500, gold_chance: 1.0, collider_radius: 30.0),
    spawn_base_interval: 0.5,
    max_count: 500,
    cull_distance: 2000.0,
    difficulty_max: 10.0,
)
"#;
        let config: EnemyConfig = ron::de::from_str(ron_data).unwrap();
        assert_eq!(config.bat.base_hp, 10.0);
        assert_eq!(config.bat.speed, 150.0);
        assert_eq!(config.bat.collider_radius, 8.0);
        assert_eq!(config.skeleton.base_hp, 30.0);
        assert_eq!(config.boss_death.base_hp, 5000.0);
        assert_eq!(config.boss_death.xp_value, 500);
        assert_eq!(config.spawn_base_interval, 0.5);
        assert_eq!(config.max_count, 500);
        assert_eq!(config.cull_distance, 2000.0);
        assert_eq!(config.difficulty_max, 10.0);
    }

    #[test]
    fn stats_for_returns_correct_entry() {
        let ron_data = r#"
EnemyConfig(
    bat: (base_hp: 10.0, speed: 150.0, damage: 5.0, xp_value: 3, gold_chance: 0.05, collider_radius: 8.0),
    skeleton: (base_hp: 30.0, speed: 80.0, damage: 8.0, xp_value: 5, gold_chance: 0.08, collider_radius: 12.0),
    zombie: (base_hp: 80.0, speed: 40.0, damage: 12.0, xp_value: 8, gold_chance: 0.10, collider_radius: 14.0),
    ghost: (base_hp: 40.0, speed: 70.0, damage: 10.0, xp_value: 6, gold_chance: 0.08, collider_radius: 10.0),
    demon: (base_hp: 100.0, speed: 120.0, damage: 15.0, xp_value: 10, gold_chance: 0.12, collider_radius: 14.0),
    medusa: (base_hp: 60.0, speed: 60.0, damage: 12.0, xp_value: 8, gold_chance: 0.10, collider_radius: 12.0),
    dragon: (base_hp: 200.0, speed: 80.0, damage: 20.0, xp_value: 15, gold_chance: 0.15, collider_radius: 20.0),
    boss_death: (base_hp: 5000.0, speed: 30.0, damage: 50.0, xp_value: 500, gold_chance: 1.0, collider_radius: 30.0),
    spawn_base_interval: 0.5,
    max_count: 500,
    cull_distance: 2000.0,
    difficulty_max: 10.0,
)
"#;
        let config: EnemyConfig = ron::de::from_str(ron_data).unwrap();
        assert_eq!(config.stats_for(EnemyType::Bat).base_hp, 10.0);
        assert_eq!(config.stats_for(EnemyType::Bat).collider_radius, 8.0);
        assert_eq!(config.stats_for(EnemyType::BossDeath).base_hp, 5000.0);
        assert_eq!(config.stats_for(EnemyType::BossDeath).collider_radius, 30.0);
    }
}
