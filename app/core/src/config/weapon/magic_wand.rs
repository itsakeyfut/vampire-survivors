//! Magic Wand / HolyWand weapon configuration.
//!
//! Loaded from `assets/config/weapons/magic_wand.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

/// Tunable parameters for the Magic Wand and its evolution HolyWand.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct MagicWandConfig {
    /// Projectile travel speed in pixels/second.
    pub speed: f32,
    /// Base damage at weapon level 1.
    pub base_damage: f32,
    /// Additional damage per weapon level above 1.
    pub damage_per_level: f32,
    /// Projectile lifetime in seconds.
    pub lifetime: f32,
    /// Circle collider radius for hit detection (pixels).
    pub collider_radius: f32,
    /// Number of projectile directions fired by HolyWand (evenly spread over a full circle).
    pub holy_wand_direction_count: u32,
    /// Piercing value for HolyWand projectiles; `u32::MAX` (4294967295) means infinite pierce.
    pub holy_wand_piercing: u32,
}

/// Resource holding the handle to the loaded [`MagicWandConfig`].
#[derive(Resource)]
pub struct MagicWandConfigHandle(pub Handle<MagicWandConfig>);

/// SystemParam bundle for accessing [`MagicWandConfig`].
///
/// Returns `None` while the asset is still loading. Call `.get()` to obtain
/// `Option<&MagicWandConfig>`.
#[derive(SystemParam)]
pub struct MagicWandParams<'w> {
    handle: Option<Res<'w, MagicWandConfigHandle>>,
    assets: Option<Res<'w, Assets<MagicWandConfig>>>,
}

impl<'w> MagicWandParams<'w> {
    /// Returns the currently loaded [`MagicWandConfig`], or `None` while loading.
    pub fn get(&self) -> Option<&MagicWandConfig> {
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
    fn magic_wand_config_deserialization() {
        let ron = r#"
MagicWandConfig(
    speed: 600.0,
    base_damage: 20.0,
    damage_per_level: 10.0,
    lifetime: 5.0,
    collider_radius: 8.0,
    holy_wand_direction_count: 8,
    holy_wand_piercing: 4294967295,
)
"#;
        let cfg: MagicWandConfig = ron::de::from_str(ron).unwrap();
        assert_eq!(cfg.speed, 600.0);
        assert_eq!(cfg.base_damage, 20.0);
        assert_eq!(cfg.damage_per_level, 10.0);
        assert_eq!(cfg.lifetime, 5.0);
        assert_eq!(cfg.collider_radius, 8.0);
        assert_eq!(cfg.holy_wand_direction_count, 8);
        assert_eq!(cfg.holy_wand_piercing, u32::MAX);
    }
}
