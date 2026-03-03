//! Whip / BloodyTear weapon configuration.
//!
//! Loaded from `assets/config/weapons/whip.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

/// Tunable parameters for the Whip and its evolution BloodyTear.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
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
        let cfg: WhipConfig = ron::de::from_str(ron).unwrap();
        assert_eq!(cfg.range, 160.0);
        assert_eq!(cfg.base_damage, 20.0);
        assert_eq!(cfg.damage_per_level, 10.0);
        assert_eq!(cfg.effect_duration, 0.15);
        assert_eq!(cfg.spread_factor, 0.6);
    }
}
