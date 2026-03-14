//! Bible / UnholyVespers weapon configuration.
//!
//! Loaded from `assets/config/weapons/bible.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Fallback constants (used while bible.ron is still loading)
// ---------------------------------------------------------------------------

const DEFAULT_ORB_COLLISION_RADIUS: f32 = 12.0;
const DEFAULT_HIT_COOLDOWN_SECS: f32 = 1.5;
const DEFAULT_DAMAGE_BY_LEVEL: &[f32] = &[20.0, 25.0, 30.0, 35.0, 40.0, 50.0, 60.0, 80.0];
const DEFAULT_ORBIT_RADIUS_BY_LEVEL: &[f32] = &[80.0, 80.0, 80.0, 90.0, 90.0, 100.0, 100.0, 110.0];
const DEFAULT_ORBIT_SPEED_BY_LEVEL: &[f32] = &[2.0, 2.0, 2.3, 2.3, 2.5, 2.5, 2.8, 3.0];
const DEFAULT_COUNT_BY_LEVEL: &[u32] = &[1, 1, 2, 2, 3, 3, 3, 3];

/// Deserialization mirror of [`BibleConfig`] — every field is `Option<T>` so
/// RON files with missing fields still load and emit a `warn!` instead of failing.
#[derive(Deserialize, Default)]
#[serde(default, rename = "BibleConfig")]
pub(crate) struct BibleConfigPartial {
    pub damage_by_level: Option<Vec<f32>>,
    pub orbit_radius_by_level: Option<Vec<f32>>,
    pub orbit_speed_by_level: Option<Vec<f32>>,
    pub count_by_level: Option<Vec<u32>>,
    pub orb_collision_radius: Option<f32>,
    pub hit_cooldown_secs: Option<f32>,
}

/// Tunable parameters for the Bible and its evolution UnholyVespers.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct BibleConfig {
    /// Damage per hit at each weapon level (index 0 = level 1).
    pub damage_by_level: Vec<f32>,
    /// Orbit radius in pixels at each weapon level (before `area_multiplier`).
    pub orbit_radius_by_level: Vec<f32>,
    /// Angular velocity in radians/second at each weapon level.
    pub orbit_speed_by_level: Vec<f32>,
    /// Number of orbiting bodies at each weapon level.
    pub count_by_level: Vec<u32>,
    /// Collision radius of each orb in pixels (used for hit detection).
    pub orb_collision_radius: f32,
    /// Seconds before the same enemy can be hit again by the same orb.
    pub hit_cooldown_secs: f32,
}

impl From<BibleConfigPartial> for BibleConfig {
    fn from(p: BibleConfigPartial) -> Self {
        BibleConfig {
            damage_by_level: p.damage_by_level.unwrap_or_else(|| {
                warn!("bible.ron: `damage_by_level` missing → using default");
                DEFAULT_DAMAGE_BY_LEVEL.to_vec()
            }),
            orbit_radius_by_level: p.orbit_radius_by_level.unwrap_or_else(|| {
                warn!("bible.ron: `orbit_radius_by_level` missing → using default");
                DEFAULT_ORBIT_RADIUS_BY_LEVEL.to_vec()
            }),
            orbit_speed_by_level: p.orbit_speed_by_level.unwrap_or_else(|| {
                warn!("bible.ron: `orbit_speed_by_level` missing → using default");
                DEFAULT_ORBIT_SPEED_BY_LEVEL.to_vec()
            }),
            count_by_level: p.count_by_level.unwrap_or_else(|| {
                warn!("bible.ron: `count_by_level` missing → using default");
                DEFAULT_COUNT_BY_LEVEL.to_vec()
            }),
            orb_collision_radius: p.orb_collision_radius.unwrap_or_else(|| {
                warn!(
                    "bible.ron: `orb_collision_radius` missing → using default {DEFAULT_ORB_COLLISION_RADIUS}"
                );
                DEFAULT_ORB_COLLISION_RADIUS
            }),
            hit_cooldown_secs: p.hit_cooldown_secs.unwrap_or_else(|| {
                warn!(
                    "bible.ron: `hit_cooldown_secs` missing → using default {DEFAULT_HIT_COOLDOWN_SECS}"
                );
                DEFAULT_HIT_COOLDOWN_SECS
            }),
        }
    }
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

    fn full_ron() -> &'static str {
        r#"
BibleConfig(
    damage_by_level: [20.0, 25.0, 30.0, 35.0, 40.0, 50.0, 60.0, 80.0],
    orbit_radius_by_level: [80.0, 80.0, 80.0, 90.0, 90.0, 100.0, 100.0, 110.0],
    orbit_speed_by_level: [2.0, 2.0, 2.3, 2.3, 2.5, 2.5, 2.8, 3.0],
    count_by_level: [1, 1, 2, 2, 3, 3, 3, 3],
    orb_collision_radius: 12.0,
    hit_cooldown_secs: 1.5,
)
"#
    }

    #[test]
    fn bible_config_deserialization() {
        let partial: BibleConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(full_ron())
            .unwrap();
        let cfg = BibleConfig::from(partial);
        assert_eq!(cfg.damage_by_level[0], 20.0);
        assert_eq!(cfg.damage_by_level[7], 80.0);
        assert_eq!(cfg.orbit_radius_by_level[0], 80.0);
        assert_eq!(cfg.orbit_radius_by_level[7], 110.0);
        assert_eq!(cfg.count_by_level, vec![1, 1, 2, 2, 3, 3, 3, 3]);
        assert_eq!(cfg.orb_collision_radius, 12.0);
        assert_eq!(cfg.hit_cooldown_secs, 1.5);
    }

    #[test]
    fn bible_config_count_increases_with_level() {
        let partial: BibleConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(full_ron())
            .unwrap();
        let cfg = BibleConfig::from(partial);
        // Level 1/2 → 1 orb; Level 3/4 → 2 orbs; Level 5+ → 3 orbs
        assert_eq!(cfg.count_by_level[0], 1);
        assert_eq!(cfg.count_by_level[2], 2);
        assert_eq!(cfg.count_by_level[4], 3);
    }
}
