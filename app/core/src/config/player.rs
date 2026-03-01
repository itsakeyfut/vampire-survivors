//! Player configuration loaded from `assets/config/player.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Asset type
// ---------------------------------------------------------------------------

/// Player base stats and collider radii, loaded from `assets/config/player.ron`.
///
/// Hot-reloading this file during gameplay immediately updates the live
/// `PlayerStats` component on the player entity.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct PlayerConfig {
    // Base player stats
    pub base_hp: f32,
    pub base_speed: f32,
    pub base_damage_mult: f32,
    pub base_cooldown_reduction: f32,
    pub base_projectile_speed: f32,
    pub base_duration_mult: f32,
    pub base_area_mult: f32,
    pub base_luck: f32,
    pub base_hp_regen: f32,
    pub pickup_radius: f32,
    pub invincibility_time: f32,
    // Collider radii (pixels)
    pub collider_radius: f32,
    pub collider_projectile_small: f32,
    pub collider_projectile_large: f32,
    pub collider_xp_gem: f32,
    pub collider_gold_coin: f32,
    pub collider_treasure: f32,
    // XP gem attraction
    pub gem_attraction_speed: f32,
    pub gem_absorption_radius: f32,
}

/// Resource holding the handle to the loaded player configuration.
#[derive(Resource)]
pub struct PlayerConfigHandle(pub Handle<PlayerConfig>);

// ---------------------------------------------------------------------------
// SystemParam bundle
// ---------------------------------------------------------------------------

/// SystemParam bundle for accessing [`PlayerConfig`].
///
/// Returns `None` while the asset is still loading or when
/// [`GameConfigPlugin`] has not been registered (e.g. in unit tests).
/// Call `.get()` to obtain `Option<&PlayerConfig>`.
///
/// [`GameConfigPlugin`]: crate::config::GameConfigPlugin
#[derive(SystemParam)]
pub struct PlayerParams<'w> {
    handle: Option<Res<'w, PlayerConfigHandle>>,
    assets: Option<Res<'w, Assets<PlayerConfig>>>,
}

impl<'w> PlayerParams<'w> {
    /// Returns the currently loaded [`PlayerConfig`], or `None` while loading.
    pub fn get(&self) -> Option<&PlayerConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }
}

// ---------------------------------------------------------------------------
// Hot-reload system
// ---------------------------------------------------------------------------

/// Handles hot-reloading of player configuration.
///
/// On `Modified`, propagates all stat changes to the live [`PlayerStats`]
/// component. `current_hp` is intentionally left unchanged so the player
/// does not suddenly heal or die mid-run when tweaking values.
pub fn hot_reload_player_config(
    mut events: MessageReader<AssetEvent<PlayerConfig>>,
    config_assets: Res<Assets<PlayerConfig>>,
    config_handle: Res<PlayerConfigHandle>,
    mut player_q: Query<&mut crate::components::PlayerStats, With<crate::components::Player>>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id: _ } => {
                info!("✅ Player config loaded");
            }
            AssetEvent::Modified { id: _ } => {
                if let Some(cfg) = config_assets.get(&config_handle.0) {
                    info!("🔥 Hot-reloading player config!");
                    if let Ok(mut stats) = player_q.single_mut() {
                        stats.max_hp = cfg.base_hp;
                        // current_hp intentionally not reset — avoids instant kill/heal mid-run.
                        stats.move_speed = cfg.base_speed;
                        stats.damage_multiplier = cfg.base_damage_mult;
                        stats.cooldown_reduction = cfg.base_cooldown_reduction;
                        stats.projectile_speed_mult = cfg.base_projectile_speed;
                        stats.duration_multiplier = cfg.base_duration_mult;
                        stats.pickup_radius = cfg.pickup_radius;
                        stats.gem_attraction_speed = cfg.gem_attraction_speed;
                        stats.gem_absorption_radius = cfg.gem_absorption_radius;
                        stats.area_multiplier = cfg.base_area_mult;
                        stats.luck = cfg.base_luck;
                        stats.hp_regen = cfg.base_hp_regen;
                        info!("✨ PlayerStats updated from hot-reload");
                    }
                }
            }
            AssetEvent::Removed { id: _ } => {
                warn!("⚠️ Player config removed");
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
    fn player_config_deserialization() {
        let ron_data = r#"
PlayerConfig(
    base_hp: 100.0,
    base_speed: 200.0,
    base_damage_mult: 1.0,
    base_cooldown_reduction: 0.0,
    base_projectile_speed: 1.0,
    base_duration_mult: 1.0,
    base_area_mult: 1.0,
    base_luck: 1.0,
    base_hp_regen: 0.0,
    pickup_radius: 80.0,
    invincibility_time: 0.5,
    collider_radius: 12.0,
    collider_projectile_small: 5.0,
    collider_projectile_large: 10.0,
    collider_xp_gem: 6.0,
    collider_gold_coin: 6.0,
    collider_treasure: 20.0,
    gem_attraction_speed: 200.0,
    gem_absorption_radius: 8.0,
)
"#;
        let config: PlayerConfig = ron::de::from_str(ron_data).unwrap();
        assert_eq!(config.base_hp, 100.0);
        assert_eq!(config.base_speed, 200.0);
        assert_eq!(config.base_damage_mult, 1.0);
        assert_eq!(config.pickup_radius, 80.0);
        assert_eq!(config.gem_attraction_speed, 200.0);
        assert_eq!(config.gem_absorption_radius, 8.0);
        assert_eq!(config.collider_radius, 12.0);
        assert_eq!(config.collider_projectile_small, 5.0);
    }
}
