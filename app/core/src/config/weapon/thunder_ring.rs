//! Thunder Ring / LightningRing weapon configuration.
//!
//! Loaded from `assets/config/weapons/thunder_ring.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

/// Tunable parameters for the Thunder Ring and its evolution LightningRing.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct ThunderRingConfig {
    /// Damage per lightning strike at each weapon level (index 0 = level 1).
    pub damage_by_level: Vec<f32>,
    /// Number of simultaneous strikes per activation at each weapon level.
    pub count_by_level: Vec<u32>,
    /// Duration of the lightning flash visual in seconds.
    pub effect_duration: f32,
    /// Side length of the square lightning flash sprite in pixels.
    pub visual_size: f32,
    /// RGBA color of the lightning flash sprite (values in `[0.0, 1.0]`).
    pub visual_color: (f32, f32, f32, f32),
    /// Z-depth of the strike sprite (higher = drawn on top).
    pub strike_z: f32,
    /// Maximum distance from the player (pixels) within which enemies can be targeted.
    /// Approximates the visible screen radius; enemies culled beyond this range are not struck.
    pub target_range: f32,
}

/// Resource holding the handle to the loaded [`ThunderRingConfig`].
#[derive(Resource)]
pub struct ThunderRingConfigHandle(pub Handle<ThunderRingConfig>);

/// SystemParam bundle for accessing [`ThunderRingConfig`].
///
/// Returns `None` while the asset is still loading. Call `.get()` to obtain
/// `Option<&ThunderRingConfig>`.
#[derive(SystemParam)]
pub struct ThunderRingParams<'w> {
    handle: Option<Res<'w, ThunderRingConfigHandle>>,
    assets: Option<Res<'w, Assets<ThunderRingConfig>>>,
}

impl<'w> ThunderRingParams<'w> {
    /// Returns the currently loaded [`ThunderRingConfig`], or `None` while loading.
    pub fn get(&self) -> Option<&ThunderRingConfig> {
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
ThunderRingConfig(
    damage_by_level: [40.0, 50.0, 60.0, 60.0, 70.0, 80.0, 90.0, 100.0],
    count_by_level: [1, 1, 2, 2, 3, 3, 3, 4],
    effect_duration: 0.2,
    visual_size: 24.0,
    visual_color: (0.9, 1.0, 0.2, 0.85),
    strike_z: 6.0,
    target_range: 800.0,
)
"#
    }

    #[test]
    fn thunder_ring_config_deserialization() {
        let cfg: ThunderRingConfig = ron::de::from_str(full_ron()).unwrap();
        assert_eq!(cfg.damage_by_level[0], 40.0);
        assert_eq!(cfg.damage_by_level[7], 100.0);
        assert_eq!(cfg.count_by_level, vec![1, 1, 2, 2, 3, 3, 3, 4]);
        assert_eq!(cfg.effect_duration, 0.2);
        assert_eq!(cfg.visual_size, 24.0);
        assert_eq!(cfg.visual_color, (0.9, 1.0, 0.2, 0.85));
        assert_eq!(cfg.strike_z, 6.0);
        assert_eq!(cfg.target_range, 800.0);
    }

    #[test]
    fn thunder_ring_config_count_increases_with_level() {
        let cfg: ThunderRingConfig = ron::de::from_str(full_ron()).unwrap();
        // Level 1/2 → 1 strike; Level 3/4 → 2 strikes; Level 5+ → 3-4 strikes
        assert_eq!(cfg.count_by_level[0], 1);
        assert_eq!(cfg.count_by_level[2], 2);
        assert_eq!(cfg.count_by_level[4], 3);
        assert_eq!(cfg.count_by_level[7], 4);
    }
}
