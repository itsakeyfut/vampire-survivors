//! Passive item configuration loaded from `assets/config/passive.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Asset type
// ---------------------------------------------------------------------------

/// Per-level stat bonuses for each passive item type.
///
/// Loaded from `assets/config/passive.ron` and hot-reloaded while the game
/// is running. Systems that read via [`PassiveParams`] pick up new values
/// immediately — however, already-applied bonuses on live `PlayerStats` are
/// not retroactively adjusted; changes take effect on the next upgrade.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct PassiveConfig {
    /// Damage multiplier bonus added per Spinach level.
    pub spinach_damage_per_level: f32,
    /// Move-speed bonus (px/s) added per Wings level.
    pub wings_speed_per_level: f32,
    /// Max-HP (and current-HP) bonus added per HollowHeart level.
    pub hollow_heart_hp_per_level: f32,
    /// Luck multiplier bonus added per Clover level.
    pub clover_luck_per_level: f32,
    /// Cooldown-reduction fraction added per EmptyTome level (capped at 0.9 total).
    pub empty_tome_cdr_per_level: f32,
    /// Projectile-speed multiplier bonus added per Bracer level.
    pub bracer_proj_speed_per_level: f32,
    /// Duration multiplier bonus added per Spellbinder level.
    pub spellbinder_duration_per_level: f32,
    /// Extra projectiles added per Duplicator level.
    pub duplicator_projectiles_per_level: u32,
    /// HP-regeneration bonus (HP/s) added per Pummarola level.
    pub pummarola_regen_per_level: f32,
}

/// Resource holding the handle to the loaded passive configuration.
#[derive(Resource)]
pub struct PassiveConfigHandle(pub Handle<PassiveConfig>);

// ---------------------------------------------------------------------------
// SystemParam bundle
// ---------------------------------------------------------------------------

/// SystemParam bundle for accessing [`PassiveConfig`].
///
/// Returns `None` while the asset is still loading or when
/// [`GameConfigPlugin`](crate::config::GameConfigPlugin) has not been
/// registered (e.g. in unit tests). Call `.get()` to obtain
/// `Option<&PassiveConfig>`.
#[derive(SystemParam)]
pub struct PassiveParams<'w> {
    handle: Option<Res<'w, PassiveConfigHandle>>,
    assets: Option<Res<'w, Assets<PassiveConfig>>>,
}

impl<'w> PassiveParams<'w> {
    /// Returns the currently loaded [`PassiveConfig`], or `None`.
    pub fn get(&self) -> Option<&PassiveConfig> {
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
    fn passive_config_deserialization() {
        let ron_data = r#"
PassiveConfig(
    spinach_damage_per_level:       0.10,
    wings_speed_per_level:         20.0,
    hollow_heart_hp_per_level:     20.0,
    clover_luck_per_level:          0.10,
    empty_tome_cdr_per_level:       0.08,
    bracer_proj_speed_per_level:    0.10,
    spellbinder_duration_per_level: 0.10,
    duplicator_projectiles_per_level: 1,
    pummarola_regen_per_level:      0.5,
)
"#;
        let cfg: PassiveConfig = ron::de::from_str(ron_data).expect("RON parse must succeed");
        assert!((cfg.spinach_damage_per_level - 0.10).abs() < 1e-6);
        assert!((cfg.wings_speed_per_level - 20.0).abs() < 1e-6);
        assert!((cfg.hollow_heart_hp_per_level - 20.0).abs() < 1e-6);
        assert_eq!(cfg.duplicator_projectiles_per_level, 1);
    }

    #[test]
    fn passive_bonus_values_are_positive() {
        let ron_data = r#"
PassiveConfig(
    spinach_damage_per_level:       0.10,
    wings_speed_per_level:         20.0,
    hollow_heart_hp_per_level:     20.0,
    clover_luck_per_level:          0.10,
    empty_tome_cdr_per_level:       0.08,
    bracer_proj_speed_per_level:    0.10,
    spellbinder_duration_per_level: 0.10,
    duplicator_projectiles_per_level: 1,
    pummarola_regen_per_level:      0.5,
)
"#;
        let cfg: PassiveConfig = ron::de::from_str(ron_data).expect("RON parse must succeed");
        assert!(cfg.spinach_damage_per_level > 0.0);
        assert!(cfg.wings_speed_per_level > 0.0);
        assert!(cfg.hollow_heart_hp_per_level > 0.0);
        assert!(cfg.clover_luck_per_level > 0.0);
        assert!(cfg.empty_tome_cdr_per_level > 0.0);
        assert!(cfg.bracer_proj_speed_per_level > 0.0);
        assert!(cfg.spellbinder_duration_per_level > 0.0);
        assert!(cfg.duplicator_projectiles_per_level > 0);
        assert!(cfg.pummarola_regen_per_level > 0.0);
    }
}
