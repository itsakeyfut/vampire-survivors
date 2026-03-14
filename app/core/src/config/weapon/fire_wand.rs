//! Fire Wand weapon configuration.
//!
//! Loaded from `assets/config/weapons/fire_wand.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Fallback constants (used while fire_wand.ron is still loading)
// ---------------------------------------------------------------------------

const DEFAULT_SPEED: f32 = 250.0;
const DEFAULT_LIFETIME: f32 = 3.0;
const DEFAULT_COLLIDER_RADIUS: f32 = 12.0;
const DEFAULT_EXPLOSION_DURATION: f32 = 0.3;
const DEFAULT_EXPLOSION_Z: f32 = 7.0;
const DEFAULT_DAMAGE_BY_LEVEL: [f32; 8] = [80.0, 100.0, 120.0, 150.0, 180.0, 220.0, 270.0, 330.0];
const DEFAULT_AOE_DAMAGE_BY_LEVEL: [f32; 8] = [40.0, 50.0, 60.0, 75.0, 90.0, 110.0, 135.0, 165.0];
const DEFAULT_AOE_RADIUS_BY_LEVEL: [f32; 8] =
    [80.0, 90.0, 100.0, 110.0, 120.0, 130.0, 140.0, 150.0];

/// Deserialization mirror of [`FireWandConfig`] — every field is `Option<T>` so
/// RON files with missing fields still load and emit a `warn!` instead of failing.
#[derive(Deserialize, Default)]
#[serde(default, rename = "FireWandConfig")]
pub(crate) struct FireWandConfigPartial {
    pub damage_by_level: Option<[f32; 8]>,
    pub aoe_damage_by_level: Option<[f32; 8]>,
    pub aoe_radius_by_level: Option<[f32; 8]>,
    pub speed: Option<f32>,
    pub lifetime: Option<f32>,
    pub collider_radius: Option<f32>,
    pub explosion_duration: Option<f32>,
    pub explosion_color: Option<(f32, f32, f32, f32)>,
    pub explosion_z: Option<f32>,
}

/// Tunable parameters for the Fire Wand weapon.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct FireWandConfig {
    /// Direct-hit damage at each weapon level (index 0 = level 1).
    pub damage_by_level: [f32; 8],
    /// Area-of-effect explosion damage at each weapon level.
    pub aoe_damage_by_level: [f32; 8],
    /// Explosion radius in pixels at each weapon level.
    pub aoe_radius_by_level: [f32; 8],
    /// Fireball travel speed in pixels/second (constant across levels).
    pub speed: f32,
    /// Fireball lifetime in seconds (despawned if it never hits anything).
    pub lifetime: f32,
    /// Circle collider radius for hit detection (pixels).
    pub collider_radius: f32,
    /// How long the explosion visual lingers after impact (seconds).
    pub explosion_duration: f32,
    /// RGBA colour of the explosion visual.
    pub explosion_color: (f32, f32, f32, f32),
    /// Z-depth of spawned explosion visual entities.
    pub explosion_z: f32,
}

impl From<FireWandConfigPartial> for FireWandConfig {
    fn from(p: FireWandConfigPartial) -> Self {
        FireWandConfig {
            damage_by_level: p.damage_by_level.unwrap_or_else(|| {
                warn!("fire_wand.ron: `damage_by_level` missing → using default");
                DEFAULT_DAMAGE_BY_LEVEL
            }),
            aoe_damage_by_level: p.aoe_damage_by_level.unwrap_or_else(|| {
                warn!("fire_wand.ron: `aoe_damage_by_level` missing → using default");
                DEFAULT_AOE_DAMAGE_BY_LEVEL
            }),
            aoe_radius_by_level: p.aoe_radius_by_level.unwrap_or_else(|| {
                warn!("fire_wand.ron: `aoe_radius_by_level` missing → using default");
                DEFAULT_AOE_RADIUS_BY_LEVEL
            }),
            speed: p.speed.unwrap_or_else(|| {
                warn!("fire_wand.ron: `speed` missing → using default {DEFAULT_SPEED}");
                DEFAULT_SPEED
            }),
            lifetime: p.lifetime.unwrap_or_else(|| {
                warn!("fire_wand.ron: `lifetime` missing → using default {DEFAULT_LIFETIME}");
                DEFAULT_LIFETIME
            }),
            collider_radius: p.collider_radius.unwrap_or_else(|| {
                warn!(
                    "fire_wand.ron: `collider_radius` missing → using default {DEFAULT_COLLIDER_RADIUS}"
                );
                DEFAULT_COLLIDER_RADIUS
            }),
            explosion_duration: p.explosion_duration.unwrap_or_else(|| {
                warn!(
                    "fire_wand.ron: `explosion_duration` missing → using default {DEFAULT_EXPLOSION_DURATION}"
                );
                DEFAULT_EXPLOSION_DURATION
            }),
            explosion_color: p.explosion_color.unwrap_or_else(|| {
                warn!("fire_wand.ron: `explosion_color` missing → using default");
                (1.0, 1.0, 1.0, 1.0)
            }),
            explosion_z: p.explosion_z.unwrap_or_else(|| {
                warn!(
                    "fire_wand.ron: `explosion_z` missing → using default {DEFAULT_EXPLOSION_Z}"
                );
                DEFAULT_EXPLOSION_Z
            }),
        }
    }
}

/// Resource holding the handle to the loaded [`FireWandConfig`].
#[derive(Resource)]
pub struct FireWandConfigHandle(pub Handle<FireWandConfig>);

/// SystemParam bundle for accessing [`FireWandConfig`].
///
/// Returns `None` while the asset is still loading. Call `.get()` to obtain
/// `Option<&FireWandConfig>`.
#[derive(SystemParam)]
pub struct FireWandParams<'w> {
    handle: Option<Res<'w, FireWandConfigHandle>>,
    assets: Option<Res<'w, Assets<FireWandConfig>>>,
}

impl<'w> FireWandParams<'w> {
    /// Returns the currently loaded [`FireWandConfig`], or `None` while loading.
    pub fn get(&self) -> Option<&FireWandConfig> {
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
FireWandConfig(
    damage_by_level:    (80.0, 100.0, 120.0, 150.0, 180.0, 220.0, 270.0, 330.0),
    aoe_damage_by_level:(40.0, 50.0,  60.0,  75.0,  90.0,  110.0, 135.0, 165.0),
    aoe_radius_by_level:(80.0, 90.0,  100.0, 110.0, 120.0, 130.0, 140.0, 150.0),
    speed:              250.0,
    lifetime:           3.0,
    collider_radius:    12.0,
    explosion_duration: 0.3,
    explosion_color:    (1.0, 0.4, 0.1, 0.8),
    explosion_z:        7.0,
)
"#
    }

    #[test]
    fn fire_wand_config_deserialization() {
        let partial: FireWandConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(full_ron())
            .unwrap();
        let cfg = FireWandConfig::from(partial);
        assert_eq!(cfg.damage_by_level[0], 80.0);
        assert_eq!(cfg.damage_by_level[7], 330.0);
        assert_eq!(cfg.aoe_damage_by_level[0], 40.0);
        assert_eq!(cfg.aoe_radius_by_level[0], 80.0);
        assert_eq!(cfg.speed, 250.0);
        assert_eq!(cfg.lifetime, 3.0);
        assert_eq!(cfg.collider_radius, 12.0);
        assert_eq!(cfg.explosion_duration, 0.3);
        assert_eq!(cfg.explosion_color, (1.0, 0.4, 0.1, 0.8));
        assert_eq!(cfg.explosion_z, 7.0);
    }

    #[test]
    fn fire_wand_config_damage_increases_with_level() {
        let partial: FireWandConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(full_ron())
            .unwrap();
        let cfg = FireWandConfig::from(partial);
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
    fn fire_wand_config_aoe_radius_increases_with_level() {
        let partial: FireWandConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(full_ron())
            .unwrap();
        let cfg = FireWandConfig::from(partial);
        for i in 1..cfg.aoe_radius_by_level.len() {
            assert!(
                cfg.aoe_radius_by_level[i] >= cfg.aoe_radius_by_level[i - 1],
                "aoe_radius at level {} should be >= level {}",
                i + 1,
                i
            );
        }
    }
}
