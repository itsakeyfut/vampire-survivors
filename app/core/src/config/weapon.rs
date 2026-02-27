//! Weapon configuration loaded from `assets/config/weapons.ron`.
//!
//! Covers tunable parameters for all base weapons and their evolutions.
//! Systems that read via [`WeaponParams`] pick up hot-reloaded values
//! automatically on the next frame — no extra hot-reload system needed.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Asset type
// ---------------------------------------------------------------------------

/// Tunable parameters for all weapons, loaded from `assets/config/weapons.ron`.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct WeaponConfig {
    // ----- Whip / BloodyTear -----
    /// Reach of the Whip in pixels (before `area_multiplier`).
    pub whip_range: f32,
    /// Base damage at weapon level 1.
    pub whip_base_damage: f32,
    /// Additional damage per weapon level above 1.
    pub whip_damage_per_level: f32,
    /// How long the swing visual stays on screen (seconds).
    pub whip_effect_duration: f32,
    /// Vertical spread factor: enemy passes when `rel.y.abs() < range * factor`.
    pub whip_spread_factor: f32,

    // ----- Magic Wand / HolyWand -----
    /// Projectile travel speed in pixels/second.
    pub magic_wand_speed: f32,
    /// Base damage at weapon level 1.
    pub magic_wand_base_damage: f32,
    /// Additional damage per weapon level above 1.
    pub magic_wand_damage_per_level: f32,
    /// Projectile lifetime in seconds.
    pub magic_wand_lifetime: f32,
    /// Circle collider radius for hit detection (pixels).
    pub magic_wand_collider_radius: f32,
}

/// Resource holding the handle to the loaded weapon configuration.
#[derive(Resource)]
pub struct WeaponConfigHandle(pub Handle<WeaponConfig>);

// ---------------------------------------------------------------------------
// SystemParam bundle
// ---------------------------------------------------------------------------

/// SystemParam bundle for accessing [`WeaponConfig`].
///
/// Returns `None` while the asset is still loading or when
/// [`GameConfigPlugin`](crate::config::GameConfigPlugin) has not been
/// registered (e.g. in unit tests). Call `.get()` to obtain
/// `Option<&WeaponConfig>`.
#[derive(SystemParam)]
pub struct WeaponParams<'w> {
    handle: Option<Res<'w, WeaponConfigHandle>>,
    assets: Option<Res<'w, Assets<WeaponConfig>>>,
}

impl<'w> WeaponParams<'w> {
    /// Returns the currently loaded [`WeaponConfig`], or `None` while loading.
    pub fn get(&self) -> Option<&WeaponConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weapon_config_deserialization() {
        let ron_data = r#"
WeaponConfig(
    whip_range: 160.0,
    whip_base_damage: 20.0,
    whip_damage_per_level: 10.0,
    whip_effect_duration: 0.15,
    whip_spread_factor: 0.6,
    magic_wand_speed: 600.0,
    magic_wand_base_damage: 20.0,
    magic_wand_damage_per_level: 10.0,
    magic_wand_lifetime: 5.0,
    magic_wand_collider_radius: 8.0,
)
"#;
        let config: WeaponConfig = ron::de::from_str(ron_data).unwrap();
        assert_eq!(config.whip_range, 160.0);
        assert_eq!(config.whip_base_damage, 20.0);
        assert_eq!(config.whip_damage_per_level, 10.0);
        assert_eq!(config.whip_effect_duration, 0.15);
        assert_eq!(config.whip_spread_factor, 0.6);
        assert_eq!(config.magic_wand_speed, 600.0);
        assert_eq!(config.magic_wand_base_damage, 20.0);
        assert_eq!(config.magic_wand_damage_per_level, 10.0);
        assert_eq!(config.magic_wand_lifetime, 5.0);
        assert_eq!(config.magic_wand_collider_radius, 8.0);
    }
}
