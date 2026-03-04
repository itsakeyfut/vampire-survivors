//! Fire Wand weapon configuration.
//!
//! Loaded from `assets/config/weapons/fire_wand.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

/// Tunable parameters for the Fire Wand weapon.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct FireWandConfig {
    /// Direct-hit damage at each weapon level (index 0 = level 1).
    pub damage_by_level: Vec<f32>,
    /// Area-of-effect explosion damage at each weapon level.
    pub aoe_damage_by_level: Vec<f32>,
    /// Explosion radius in pixels at each weapon level.
    pub aoe_radius_by_level: Vec<f32>,
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
    damage_by_level:    [80.0, 100.0, 120.0, 150.0, 180.0, 220.0, 270.0, 330.0],
    aoe_damage_by_level:[40.0, 50.0,  60.0,  75.0,  90.0,  110.0, 135.0, 165.0],
    aoe_radius_by_level:[80.0, 90.0,  100.0, 110.0, 120.0, 130.0, 140.0, 150.0],
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
        let cfg: FireWandConfig = ron::de::from_str(full_ron()).unwrap();
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
        let cfg: FireWandConfig = ron::de::from_str(full_ron()).unwrap();
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
        let cfg: FireWandConfig = ron::de::from_str(full_ron()).unwrap();
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
