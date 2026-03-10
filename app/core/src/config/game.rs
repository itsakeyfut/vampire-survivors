//! Game configuration loaded from `assets/config/game.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

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
    }
}
