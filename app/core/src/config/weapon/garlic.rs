//! Garlic / SoulEater weapon configuration.
//!
//! Loaded from `assets/config/weapons/garlic.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

/// Tunable parameters for Garlic and its evolution SoulEater.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct GarlicConfig {
    /// Damage dealt per aura tick, indexed by level (index 0 = level 1).
    pub damage_by_level: Vec<f32>,
    /// Aura radius in pixels, indexed by level (index 0 = level 1).
    pub radius_by_level: Vec<f32>,
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
        let cfg: GarlicConfig = ron::de::from_str(ron).unwrap();
        assert_eq!(
            cfg.damage_by_level,
            vec![5.0, 5.0, 8.0, 8.0, 10.0, 12.0, 15.0, 20.0]
        );
        assert_eq!(cfg.radius_by_level[0], 80.0);
        assert_eq!(cfg.radius_by_level[7], 150.0);
    }
}
