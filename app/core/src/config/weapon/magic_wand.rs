//! Magic Wand / HolyWand weapon configuration.
//!
//! Loaded from `assets/config/weapons/magic_wand.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Fallback constants (used while magic_wand.ron is still loading)
// ---------------------------------------------------------------------------

const DEFAULT_SPEED: f32 = 600.0;
const DEFAULT_BASE_DAMAGE: f32 = 20.0;
const DEFAULT_DAMAGE_PER_LEVEL: f32 = 10.0;
const DEFAULT_LIFETIME: f32 = 5.0;
const DEFAULT_COLLIDER_RADIUS: f32 = 8.0;
const DEFAULT_HOLY_WAND_DIRECTION_COUNT: u32 = 8;
const DEFAULT_HOLY_WAND_PIERCING: u32 = u32::MAX;

/// Deserialization mirror of [`MagicWandConfig`] — every field is `Option<T>` so
/// RON files with missing fields still load and emit a `warn!` instead of failing.
#[derive(Deserialize, Default)]
#[serde(default, rename = "MagicWandConfig")]
pub(crate) struct MagicWandConfigPartial {
    pub speed: Option<f32>,
    pub base_damage: Option<f32>,
    pub damage_per_level: Option<f32>,
    pub lifetime: Option<f32>,
    pub collider_radius: Option<f32>,
    pub holy_wand_direction_count: Option<u32>,
    pub holy_wand_piercing: Option<u32>,
}

/// Tunable parameters for the Magic Wand and its evolution HolyWand.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct MagicWandConfig {
    /// Projectile travel speed in pixels/second.
    pub speed: f32,
    /// Base damage at weapon level 1.
    pub base_damage: f32,
    /// Additional damage per weapon level above 1.
    pub damage_per_level: f32,
    /// Projectile lifetime in seconds.
    pub lifetime: f32,
    /// Circle collider radius for hit detection (pixels).
    pub collider_radius: f32,
    /// Number of projectile directions fired by HolyWand (evenly spread over a full circle).
    pub holy_wand_direction_count: u32,
    /// Piercing value for HolyWand projectiles; `u32::MAX` (4294967295) means infinite pierce.
    pub holy_wand_piercing: u32,
}

impl From<MagicWandConfigPartial> for MagicWandConfig {
    fn from(p: MagicWandConfigPartial) -> Self {
        MagicWandConfig {
            speed: p.speed.unwrap_or_else(|| {
                warn!("magic_wand.ron: `speed` missing → using default {DEFAULT_SPEED}");
                DEFAULT_SPEED
            }),
            base_damage: p.base_damage.unwrap_or_else(|| {
                warn!(
                    "magic_wand.ron: `base_damage` missing → using default {DEFAULT_BASE_DAMAGE}"
                );
                DEFAULT_BASE_DAMAGE
            }),
            damage_per_level: p.damage_per_level.unwrap_or_else(|| {
                warn!(
                    "magic_wand.ron: `damage_per_level` missing → using default {DEFAULT_DAMAGE_PER_LEVEL}"
                );
                DEFAULT_DAMAGE_PER_LEVEL
            }),
            lifetime: p.lifetime.unwrap_or_else(|| {
                warn!("magic_wand.ron: `lifetime` missing → using default {DEFAULT_LIFETIME}");
                DEFAULT_LIFETIME
            }),
            collider_radius: p.collider_radius.unwrap_or_else(|| {
                warn!(
                    "magic_wand.ron: `collider_radius` missing → using default {DEFAULT_COLLIDER_RADIUS}"
                );
                DEFAULT_COLLIDER_RADIUS
            }),
            holy_wand_direction_count: p.holy_wand_direction_count.unwrap_or_else(|| {
                warn!(
                    "magic_wand.ron: `holy_wand_direction_count` missing → using default {DEFAULT_HOLY_WAND_DIRECTION_COUNT}"
                );
                DEFAULT_HOLY_WAND_DIRECTION_COUNT
            }),
            holy_wand_piercing: p.holy_wand_piercing.unwrap_or_else(|| {
                warn!(
                    "magic_wand.ron: `holy_wand_piercing` missing → using default"
                );
                DEFAULT_HOLY_WAND_PIERCING
            }),
        }
    }
}

/// Resource holding the handle to the loaded [`MagicWandConfig`].
#[derive(Resource)]
pub struct MagicWandConfigHandle(pub Handle<MagicWandConfig>);

/// SystemParam bundle for accessing [`MagicWandConfig`].
///
/// Returns `None` while the asset is still loading. Call `.get()` to obtain
/// `Option<&MagicWandConfig>`.
#[derive(SystemParam)]
pub struct MagicWandParams<'w> {
    handle: Option<Res<'w, MagicWandConfigHandle>>,
    assets: Option<Res<'w, Assets<MagicWandConfig>>>,
}

impl<'w> MagicWandParams<'w> {
    /// Returns the currently loaded [`MagicWandConfig`], or `None` while loading.
    pub fn get(&self) -> Option<&MagicWandConfig> {
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
    fn magic_wand_config_deserialization() {
        let ron = r#"
MagicWandConfig(
    speed: 600.0,
    base_damage: 20.0,
    damage_per_level: 10.0,
    lifetime: 5.0,
    collider_radius: 8.0,
    holy_wand_direction_count: 8,
    holy_wand_piercing: 4294967295,
)
"#;
        let partial: MagicWandConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(ron)
            .unwrap();
        let cfg = MagicWandConfig::from(partial);
        assert_eq!(cfg.speed, 600.0);
        assert_eq!(cfg.base_damage, 20.0);
        assert_eq!(cfg.damage_per_level, 10.0);
        assert_eq!(cfg.lifetime, 5.0);
        assert_eq!(cfg.collider_radius, 8.0);
        assert_eq!(cfg.holy_wand_direction_count, 8);
        assert_eq!(cfg.holy_wand_piercing, u32::MAX);
    }
}
