//! Knife / ThousandEdge weapon configuration.
//!
//! Loaded from `assets/config/weapons/knife.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

/// Tunable parameters for the Knife and its evolution ThousandEdge.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct KnifeConfig {
    /// Base projectile speed at level 1 (pixels/second).
    pub base_speed: f32,
    /// Speed increase applied every weapon level (floor(level / 2) steps).
    pub speed_per_two_levels: f32,
    /// Base damage at weapon level 1.
    pub base_damage: f32,
    /// Damage increase applied every two weapon levels (floor((level − 1) / 2) steps).
    pub damage_per_two_levels: f32,
    /// Projectile lifetime in seconds.
    pub lifetime: f32,
    /// Circle collider radius for hit detection (pixels).
    pub collider_radius: f32,
    /// Angular gap between adjacent knives in a multi-projectile fan (degrees).
    pub spread_angle_deg: f32,
    /// Number of projectiles fired per activation, indexed by level (index 0 = level 1).
    pub count_by_level: Vec<u32>,
}

/// Resource holding the handle to the loaded [`KnifeConfig`].
#[derive(Resource)]
pub struct KnifeConfigHandle(pub Handle<KnifeConfig>);

/// SystemParam bundle for accessing [`KnifeConfig`].
///
/// Returns `None` while the asset is still loading. Call `.get()` to obtain
/// `Option<&KnifeConfig>`.
#[derive(SystemParam)]
pub struct KnifeParams<'w> {
    handle: Option<Res<'w, KnifeConfigHandle>>,
    assets: Option<Res<'w, Assets<KnifeConfig>>>,
}

impl<'w> KnifeParams<'w> {
    /// Returns the currently loaded [`KnifeConfig`], or `None` while loading.
    pub fn get(&self) -> Option<&KnifeConfig> {
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
    fn knife_config_deserialization() {
        let ron = r#"
KnifeConfig(
    base_speed: 600.0,
    speed_per_two_levels: 100.0,
    base_damage: 15.0,
    damage_per_two_levels: 5.0,
    lifetime: 5.0,
    collider_radius: 6.0,
    spread_angle_deg: 15.0,
    count_by_level: [1, 1, 2, 2, 3, 3, 4, 5],
)
"#;
        let cfg: KnifeConfig = ron::de::from_str(ron).unwrap();
        assert_eq!(cfg.base_speed, 600.0);
        assert_eq!(cfg.base_damage, 15.0);
        assert_eq!(cfg.spread_angle_deg, 15.0);
        assert_eq!(cfg.count_by_level, vec![1, 1, 2, 2, 3, 3, 4, 5]);
    }
}
