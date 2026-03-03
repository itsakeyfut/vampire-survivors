//! Cross weapon configuration.
//!
//! Loaded from `assets/config/weapons/cross.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

/// Tunable parameters for the Cross boomerang weapon.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct CrossConfig {
    /// Damage per hit at each weapon level (index 0 = level 1).
    pub damage_by_level: Vec<f32>,
    /// Projectile speed in pixels/second at each weapon level.
    pub speed_by_level: Vec<f32>,
    /// Maximum travel distance before reversing, in pixels, at each weapon level.
    pub max_range_by_level: Vec<f32>,
    /// Number of projectiles fired per activation at each weapon level.
    pub count_by_level: Vec<u32>,
    /// Angular gap between adjacent projectiles in a fan (degrees).
    pub spread_angle_deg: f32,
    /// Circle collider radius for hit detection (pixels).
    pub collider_radius: f32,
}

/// Resource holding the handle to the loaded [`CrossConfig`].
#[derive(Resource)]
pub struct CrossConfigHandle(pub Handle<CrossConfig>);

/// SystemParam bundle for accessing [`CrossConfig`].
///
/// Returns `None` while the asset is still loading. Call `.get()` to obtain
/// `Option<&CrossConfig>`.
#[derive(SystemParam)]
pub struct CrossParams<'w> {
    handle: Option<Res<'w, CrossConfigHandle>>,
    assets: Option<Res<'w, Assets<CrossConfig>>>,
}

impl<'w> CrossParams<'w> {
    /// Returns the currently loaded [`CrossConfig`], or `None` while loading.
    pub fn get(&self) -> Option<&CrossConfig> {
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

    fn full_ron() -> &'static str {
        r#"
CrossConfig(
    damage_by_level:    [50.0, 60.0, 70.0, 80.0, 90.0, 110.0, 130.0, 160.0],
    speed_by_level:     [300.0, 320.0, 340.0, 360.0, 380.0, 400.0, 430.0, 460.0],
    max_range_by_level: [150.0, 160.0, 175.0, 190.0, 205.0, 220.0, 235.0, 250.0],
    count_by_level:     [1, 1, 1, 1, 2, 2, 2, 2],
    spread_angle_deg:   30.0,
    collider_radius:    8.0,
)
"#
    }

    #[test]
    fn cross_config_deserialization() {
        let cfg: CrossConfig = ron::de::from_str(full_ron()).unwrap();
        assert_eq!(cfg.damage_by_level[0], 50.0);
        assert_eq!(cfg.damage_by_level[7], 160.0);
        assert_eq!(cfg.speed_by_level[0], 300.0);
        assert_eq!(cfg.max_range_by_level[0], 150.0);
        assert_eq!(cfg.count_by_level, vec![1, 1, 1, 1, 2, 2, 2, 2]);
        assert_eq!(cfg.spread_angle_deg, 30.0);
        assert_eq!(cfg.collider_radius, 8.0);
    }

    #[test]
    fn cross_config_damage_increases_with_level() {
        let cfg: CrossConfig = ron::de::from_str(full_ron()).unwrap();
        // Each level should be >= the previous.
        for i in 1..cfg.damage_by_level.len() {
            assert!(
                cfg.damage_by_level[i] >= cfg.damage_by_level[i - 1],
                "damage at level {} should be >= level {}",
                i + 1,
                i
            );
        }
    }

    #[test]
    fn cross_config_count_increases_at_level_5() {
        let cfg: CrossConfig = ron::de::from_str(full_ron()).unwrap();
        // Lv1-4: 1 projectile; Lv5-8: 2 projectiles
        assert_eq!(cfg.count_by_level[0], 1);
        assert_eq!(cfg.count_by_level[3], 1);
        assert_eq!(cfg.count_by_level[4], 2);
        assert_eq!(cfg.count_by_level[7], 2);
    }
}
