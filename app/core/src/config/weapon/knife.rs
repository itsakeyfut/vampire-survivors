//! Knife / ThousandEdge weapon configuration.
//!
//! Loaded from `assets/config/weapons/knife.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Fallback constants (used while knife.ron is still loading)
// ---------------------------------------------------------------------------

const DEFAULT_BASE_SPEED: f32 = 600.0;
const DEFAULT_SPEED_PER_TWO_LEVELS: f32 = 100.0;
const DEFAULT_BASE_DAMAGE: f32 = 15.0;
const DEFAULT_DAMAGE_PER_TWO_LEVELS: f32 = 5.0;
const DEFAULT_LIFETIME: f32 = 5.0;
const DEFAULT_COLLIDER_RADIUS: f32 = 6.0;
const DEFAULT_SPREAD_ANGLE_DEG: f32 = 15.0;
const DEFAULT_COUNT_BY_LEVEL: &[u32] = &[1, 1, 2, 2, 3, 3, 4, 5];

/// Deserialization mirror of [`KnifeConfig`] — every field is `Option<T>` so
/// RON files with missing fields still load and emit a `warn!` instead of failing.
#[derive(Deserialize, Default)]
#[serde(default, rename = "KnifeConfig")]
pub(crate) struct KnifeConfigPartial {
    pub base_speed: Option<f32>,
    pub speed_per_two_levels: Option<f32>,
    pub base_damage: Option<f32>,
    pub damage_per_two_levels: Option<f32>,
    pub lifetime: Option<f32>,
    pub collider_radius: Option<f32>,
    pub spread_angle_deg: Option<f32>,
    pub count_by_level: Option<Vec<u32>>,
}

/// Tunable parameters for the Knife and its evolution ThousandEdge.
#[derive(Asset, TypePath, Debug, Clone)]
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

impl From<KnifeConfigPartial> for KnifeConfig {
    fn from(p: KnifeConfigPartial) -> Self {
        KnifeConfig {
            base_speed: p.base_speed.unwrap_or_else(|| {
                warn!("knife.ron: `base_speed` missing → using default {DEFAULT_BASE_SPEED}");
                DEFAULT_BASE_SPEED
            }),
            speed_per_two_levels: p.speed_per_two_levels.unwrap_or_else(|| {
                warn!(
                    "knife.ron: `speed_per_two_levels` missing → using default {DEFAULT_SPEED_PER_TWO_LEVELS}"
                );
                DEFAULT_SPEED_PER_TWO_LEVELS
            }),
            base_damage: p.base_damage.unwrap_or_else(|| {
                warn!("knife.ron: `base_damage` missing → using default {DEFAULT_BASE_DAMAGE}");
                DEFAULT_BASE_DAMAGE
            }),
            damage_per_two_levels: p.damage_per_two_levels.unwrap_or_else(|| {
                warn!(
                    "knife.ron: `damage_per_two_levels` missing → using default {DEFAULT_DAMAGE_PER_TWO_LEVELS}"
                );
                DEFAULT_DAMAGE_PER_TWO_LEVELS
            }),
            lifetime: p.lifetime.unwrap_or_else(|| {
                warn!("knife.ron: `lifetime` missing → using default {DEFAULT_LIFETIME}");
                DEFAULT_LIFETIME
            }),
            collider_radius: p.collider_radius.unwrap_or_else(|| {
                warn!(
                    "knife.ron: `collider_radius` missing → using default {DEFAULT_COLLIDER_RADIUS}"
                );
                DEFAULT_COLLIDER_RADIUS
            }),
            spread_angle_deg: p.spread_angle_deg.unwrap_or_else(|| {
                warn!(
                    "knife.ron: `spread_angle_deg` missing → using default {DEFAULT_SPREAD_ANGLE_DEG}"
                );
                DEFAULT_SPREAD_ANGLE_DEG
            }),
            count_by_level: p.count_by_level.unwrap_or_else(|| {
                warn!("knife.ron: `count_by_level` missing → using default");
                DEFAULT_COUNT_BY_LEVEL.to_vec()
            }),
        }
    }
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
        let partial: KnifeConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(ron)
            .unwrap();
        let cfg = KnifeConfig::from(partial);
        assert_eq!(cfg.base_speed, 600.0);
        assert_eq!(cfg.base_damage, 15.0);
        assert_eq!(cfg.spread_angle_deg, 15.0);
        assert_eq!(cfg.count_by_level, vec![1, 1, 2, 2, 3, 3, 4, 5]);
    }
}
