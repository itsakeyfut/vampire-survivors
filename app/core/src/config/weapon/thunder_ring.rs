//! Thunder Ring / LightningRing weapon configuration.
//!
//! Loaded from `assets/config/weapons/thunder_ring.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Fallback constants (used while thunder_ring.ron is still loading)
// ---------------------------------------------------------------------------

const DEFAULT_EFFECT_DURATION: f32 = 0.2;
const DEFAULT_VISUAL_SIZE: f32 = 24.0;
const DEFAULT_STRIKE_Z: f32 = 6.0;
const DEFAULT_TARGET_RANGE: f32 = 800.0;
const DEFAULT_DAMAGE_BY_LEVEL: &[f32] =
    &[40.0, 50.0, 60.0, 60.0, 70.0, 80.0, 90.0, 100.0];
const DEFAULT_COUNT_BY_LEVEL: &[u32] = &[1, 1, 2, 2, 3, 3, 3, 4];

/// Deserialization mirror of [`ThunderRingConfig`] — every field is `Option<T>` so
/// RON files with missing fields still load and emit a `warn!` instead of failing.
#[derive(Deserialize, Default)]
#[serde(default, rename = "ThunderRingConfig")]
pub(crate) struct ThunderRingConfigPartial {
    pub damage_by_level: Option<Vec<f32>>,
    pub count_by_level: Option<Vec<u32>>,
    pub effect_duration: Option<f32>,
    pub visual_size: Option<f32>,
    pub visual_color: Option<(f32, f32, f32, f32)>,
    pub strike_z: Option<f32>,
    pub target_range: Option<f32>,
}

/// Tunable parameters for the Thunder Ring and its evolution LightningRing.
#[derive(Asset, TypePath, Debug, Clone)]
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

impl From<ThunderRingConfigPartial> for ThunderRingConfig {
    fn from(p: ThunderRingConfigPartial) -> Self {
        ThunderRingConfig {
            damage_by_level: p.damage_by_level.unwrap_or_else(|| {
                warn!("thunder_ring.ron: `damage_by_level` missing → using default");
                DEFAULT_DAMAGE_BY_LEVEL.to_vec()
            }),
            count_by_level: p.count_by_level.unwrap_or_else(|| {
                warn!("thunder_ring.ron: `count_by_level` missing → using default");
                DEFAULT_COUNT_BY_LEVEL.to_vec()
            }),
            effect_duration: p.effect_duration.unwrap_or_else(|| {
                warn!(
                    "thunder_ring.ron: `effect_duration` missing → using default {DEFAULT_EFFECT_DURATION}"
                );
                DEFAULT_EFFECT_DURATION
            }),
            visual_size: p.visual_size.unwrap_or_else(|| {
                warn!(
                    "thunder_ring.ron: `visual_size` missing → using default {DEFAULT_VISUAL_SIZE}"
                );
                DEFAULT_VISUAL_SIZE
            }),
            visual_color: p.visual_color.unwrap_or_else(|| {
                warn!("thunder_ring.ron: `visual_color` missing → using default");
                (1.0, 1.0, 1.0, 1.0)
            }),
            strike_z: p.strike_z.unwrap_or_else(|| {
                warn!(
                    "thunder_ring.ron: `strike_z` missing → using default {DEFAULT_STRIKE_Z}"
                );
                DEFAULT_STRIKE_Z
            }),
            target_range: p.target_range.unwrap_or_else(|| {
                warn!(
                    "thunder_ring.ron: `target_range` missing → using default {DEFAULT_TARGET_RANGE}"
                );
                DEFAULT_TARGET_RANGE
            }),
        }
    }
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
        let partial: ThunderRingConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(full_ron())
            .unwrap();
        let cfg = ThunderRingConfig::from(partial);
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
        let partial: ThunderRingConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(full_ron())
            .unwrap();
        let cfg = ThunderRingConfig::from(partial);
        // Level 1/2 → 1 strike; Level 3/4 → 2 strikes; Level 5+ → 3-4 strikes
        assert_eq!(cfg.count_by_level[0], 1);
        assert_eq!(cfg.count_by_level[2], 2);
        assert_eq!(cfg.count_by_level[4], 3);
        assert_eq!(cfg.count_by_level[7], 4);
    }
}
