//! Whip / BloodyTear weapon configuration.
//!
//! Loaded from `assets/config/weapons/whip.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Fallback constants (used while whip.ron is still loading)
// ---------------------------------------------------------------------------

const DEFAULT_RANGE: f32 = 160.0;
const DEFAULT_BASE_DAMAGE: f32 = 20.0;
const DEFAULT_DAMAGE_PER_LEVEL: f32 = 10.0;
const DEFAULT_EFFECT_DURATION: f32 = 0.15;
const DEFAULT_SPREAD_FACTOR: f32 = 0.6;

/// Deserialization mirror of [`WhipConfig`] — every field is `Option<T>` so
/// RON files with missing fields still load and emit a `warn!` instead of failing.
#[derive(Deserialize, Default)]
#[serde(default, rename = "WhipConfig")]
pub(crate) struct WhipConfigPartial {
    pub range: Option<f32>,
    pub base_damage: Option<f32>,
    pub damage_per_level: Option<f32>,
    pub effect_duration: Option<f32>,
    pub spread_factor: Option<f32>,
}

/// Tunable parameters for the Whip and its evolution BloodyTear.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct WhipConfig {
    /// Reach of the Whip in pixels (before `area_multiplier`).
    pub range: f32,
    /// Base damage at weapon level 1.
    pub base_damage: f32,
    /// Additional damage per weapon level above 1.
    pub damage_per_level: f32,
    /// How long the swing visual stays on screen (seconds).
    pub effect_duration: f32,
    /// Vertical spread factor: enemy passes when `rel.y.abs() < range * factor`.
    pub spread_factor: f32,
}

impl From<WhipConfigPartial> for WhipConfig {
    fn from(p: WhipConfigPartial) -> Self {
        WhipConfig {
            range: p.range.unwrap_or_else(|| {
                warn!("whip.ron: `range` missing → using default {DEFAULT_RANGE}");
                DEFAULT_RANGE
            }),
            base_damage: p.base_damage.unwrap_or_else(|| {
                warn!("whip.ron: `base_damage` missing → using default {DEFAULT_BASE_DAMAGE}");
                DEFAULT_BASE_DAMAGE
            }),
            damage_per_level: p.damage_per_level.unwrap_or_else(|| {
                warn!(
                    "whip.ron: `damage_per_level` missing → using default {DEFAULT_DAMAGE_PER_LEVEL}"
                );
                DEFAULT_DAMAGE_PER_LEVEL
            }),
            effect_duration: p.effect_duration.unwrap_or_else(|| {
                warn!(
                    "whip.ron: `effect_duration` missing → using default {DEFAULT_EFFECT_DURATION}"
                );
                DEFAULT_EFFECT_DURATION
            }),
            spread_factor: p.spread_factor.unwrap_or_else(|| {
                warn!(
                    "whip.ron: `spread_factor` missing → using default {DEFAULT_SPREAD_FACTOR}"
                );
                DEFAULT_SPREAD_FACTOR
            }),
        }
    }
}

/// Resource holding the handle to the loaded [`WhipConfig`].
#[derive(Resource)]
pub struct WhipConfigHandle(pub Handle<WhipConfig>);

/// SystemParam bundle for accessing [`WhipConfig`].
///
/// Returns `None` while the asset is still loading. Call `.get()` to obtain
/// `Option<&WhipConfig>`.
#[derive(SystemParam)]
pub struct WhipParams<'w> {
    handle: Option<Res<'w, WhipConfigHandle>>,
    assets: Option<Res<'w, Assets<WhipConfig>>>,
}

impl<'w> WhipParams<'w> {
    /// Returns the currently loaded [`WhipConfig`], or `None` while loading.
    pub fn get(&self) -> Option<&WhipConfig> {
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
    fn whip_config_deserialization() {
        let ron = r#"
WhipConfig(
    range: 160.0,
    base_damage: 20.0,
    damage_per_level: 10.0,
    effect_duration: 0.15,
    spread_factor: 0.6,
)
"#;
        let partial: WhipConfigPartial = ron::Options::default().with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME).from_str(ron).unwrap();
        let cfg = WhipConfig::from(partial);
        assert_eq!(cfg.range, 160.0);
        assert_eq!(cfg.base_damage, 20.0);
        assert_eq!(cfg.damage_per_level, 10.0);
        assert_eq!(cfg.effect_duration, 0.15);
        assert_eq!(cfg.spread_factor, 0.6);
    }
}
