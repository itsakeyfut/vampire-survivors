//! Garlic / SoulEater weapon configuration.
//!
//! Loaded from `assets/config/weapons/garlic.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Fallback constants (used while garlic.ron is still loading)
// ---------------------------------------------------------------------------

const DEFAULT_DAMAGE_BY_LEVEL: &[f32] =
    &[5.0, 5.0, 8.0, 8.0, 10.0, 12.0, 15.0, 20.0];
const DEFAULT_RADIUS_BY_LEVEL: &[f32] =
    &[80.0, 90.0, 90.0, 100.0, 110.0, 120.0, 130.0, 150.0];

/// Deserialization mirror of [`GarlicConfig`] — every field is `Option<T>` so
/// RON files with missing fields still load and emit a `warn!` instead of failing.
#[derive(Deserialize, Default)]
#[serde(default, rename = "GarlicConfig")]
pub(crate) struct GarlicConfigPartial {
    pub damage_by_level: Option<Vec<f32>>,
    pub radius_by_level: Option<Vec<f32>>,
}

/// Tunable parameters for Garlic and its evolution SoulEater.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct GarlicConfig {
    /// Damage dealt per aura tick, indexed by level (index 0 = level 1).
    pub damage_by_level: Vec<f32>,
    /// Aura radius in pixels, indexed by level (index 0 = level 1).
    pub radius_by_level: Vec<f32>,
}

impl From<GarlicConfigPartial> for GarlicConfig {
    fn from(p: GarlicConfigPartial) -> Self {
        GarlicConfig {
            damage_by_level: p.damage_by_level.unwrap_or_else(|| {
                warn!("garlic.ron: `damage_by_level` missing → using default");
                DEFAULT_DAMAGE_BY_LEVEL.to_vec()
            }),
            radius_by_level: p.radius_by_level.unwrap_or_else(|| {
                warn!("garlic.ron: `radius_by_level` missing → using default");
                DEFAULT_RADIUS_BY_LEVEL.to_vec()
            }),
        }
    }
}

/// Resource holding the handle to the loaded [`GarlicConfig`].
#[derive(Resource)]
pub struct GarlicConfigHandle(pub Handle<GarlicConfig>);

/// SystemParam bundle for accessing [`GarlicConfig`].
///
/// Returns `None` while the asset is still loading. Call `.get()` to obtain
/// `Option<&GarlicConfig>`.
#[derive(SystemParam)]
pub struct GarlicParams<'w> {
    handle: Option<Res<'w, GarlicConfigHandle>>,
    assets: Option<Res<'w, Assets<GarlicConfig>>>,
}

impl<'w> GarlicParams<'w> {
    /// Returns the currently loaded [`GarlicConfig`], or `None` while loading.
    pub fn get(&self) -> Option<&GarlicConfig> {
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
    fn garlic_config_deserialization() {
        let ron = r#"
GarlicConfig(
    damage_by_level: [5.0, 5.0, 8.0, 8.0, 10.0, 12.0, 15.0, 20.0],
    radius_by_level: [80.0, 90.0, 90.0, 100.0, 110.0, 120.0, 130.0, 150.0],
)
"#;
        let partial: GarlicConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(ron)
            .unwrap();
        let cfg = GarlicConfig::from(partial);
        assert_eq!(
            cfg.damage_by_level,
            vec![5.0, 5.0, 8.0, 8.0, 10.0, 12.0, 15.0, 20.0]
        );
        assert_eq!(cfg.radius_by_level[0], 80.0);
        assert_eq!(cfg.radius_by_level[7], 150.0);
    }
}
