//! Bible / UnholyVespers weapon configuration.
//!
//! Loaded from `assets/config/weapons/bible.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

/// Tunable parameters for the Bible and its evolution UnholyVespers.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct BibleConfig {
    /// Damage per hit at each weapon level (index 0 = level 1).
    pub damage_by_level: Vec<f32>,
    /// Orbit radius in pixels at each weapon level (before `area_multiplier`).
    pub orbit_radius_by_level: Vec<f32>,
    /// Angular velocity in radians/second at each weapon level.
    pub orbit_speed_by_level: Vec<f32>,
    /// Number of orbiting bodies at each weapon level.
    pub count_by_level: Vec<u32>,
}

/// Resource holding the handle to the loaded [`BibleConfig`].
#[derive(Resource)]
pub struct BibleConfigHandle(pub Handle<BibleConfig>);

/// SystemParam bundle for accessing [`BibleConfig`].
///
/// Returns `None` while the asset is still loading. Call `.get()` to obtain
/// `Option<&BibleConfig>`.
#[derive(SystemParam)]
pub struct BibleParams<'w> {
    handle: Option<Res<'w, BibleConfigHandle>>,
    assets: Option<Res<'w, Assets<BibleConfig>>>,
}

impl<'w> BibleParams<'w> {
    /// Returns the currently loaded [`BibleConfig`], or `None` while loading.
    pub fn get(&self) -> Option<&BibleConfig> {
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
    fn bible_config_deserialization() {
        let ron = r#"
BibleConfig(
    damage_by_level: [20.0, 25.0, 30.0, 35.0, 40.0, 50.0, 60.0, 80.0],
    orbit_radius_by_level: [80.0, 80.0, 80.0, 90.0, 90.0, 100.0, 100.0, 110.0],
    orbit_speed_by_level: [2.0, 2.0, 2.3, 2.3, 2.5, 2.5, 2.8, 3.0],
    count_by_level: [1, 1, 2, 2, 3, 3, 3, 3],
)
"#;
        let cfg: BibleConfig = ron::de::from_str(ron).unwrap();
        assert_eq!(cfg.damage_by_level[0], 20.0);
        assert_eq!(cfg.damage_by_level[7], 80.0);
        assert_eq!(cfg.orbit_radius_by_level[0], 80.0);
        assert_eq!(cfg.orbit_radius_by_level[7], 110.0);
        assert_eq!(cfg.count_by_level, vec![1, 1, 2, 2, 3, 3, 3, 3]);
    }

    #[test]
    fn bible_config_count_increases_with_level() {
        let ron = r#"
BibleConfig(
    damage_by_level: [20.0, 25.0, 30.0, 35.0, 40.0, 50.0, 60.0, 80.0],
    orbit_radius_by_level: [80.0, 80.0, 80.0, 90.0, 90.0, 100.0, 100.0, 110.0],
    orbit_speed_by_level: [2.0, 2.0, 2.3, 2.3, 2.5, 2.5, 2.8, 3.0],
    count_by_level: [1, 1, 2, 2, 3, 3, 3, 3],
)
"#;
        let cfg: BibleConfig = ron::de::from_str(ron).unwrap();
        // Level 1/2 → 1 orb; Level 3/4 → 2 orbs; Level 5+ → 3 orbs
        assert_eq!(cfg.count_by_level[0], 1);
        assert_eq!(cfg.count_by_level[2], 2);
        assert_eq!(cfg.count_by_level[4], 3);
    }
}
