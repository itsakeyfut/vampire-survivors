//! Passive item configuration loaded from `assets/config/passive.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Fallback constants (used while passive.ron is still loading)
// ---------------------------------------------------------------------------

const DEFAULT_SPINACH_DAMAGE_PER_LEVEL: f32 = 0.10;
const DEFAULT_WINGS_SPEED_PER_LEVEL: f32 = 20.0;
const DEFAULT_HOLLOW_HEART_HP_PER_LEVEL: f32 = 20.0;
const DEFAULT_CLOVER_LUCK_PER_LEVEL: f32 = 0.10;
const DEFAULT_EMPTY_TOME_CDR_PER_LEVEL: f32 = 0.08;
const DEFAULT_BRACER_PROJ_SPEED_PER_LEVEL: f32 = 0.10;
const DEFAULT_SPELLBINDER_DURATION_PER_LEVEL: f32 = 0.10;
const DEFAULT_DUPLICATOR_PROJECTILES_PER_LEVEL: u32 = 1;
const DEFAULT_PUMMAROLA_REGEN_PER_LEVEL: f32 = 0.5;

// ---------------------------------------------------------------------------
// Asset type
// ---------------------------------------------------------------------------

/// Deserialization mirror of [`PassiveConfig`] — every field is `Option<T>` so
/// RON files with missing fields still load and emit a `warn!` instead of failing.
#[derive(Deserialize, Default)]
#[serde(default, rename = "PassiveConfig")]
pub(super) struct PassiveConfigPartial {
    pub spinach_damage_per_level: Option<f32>,
    pub wings_speed_per_level: Option<f32>,
    pub hollow_heart_hp_per_level: Option<f32>,
    pub clover_luck_per_level: Option<f32>,
    pub empty_tome_cdr_per_level: Option<f32>,
    pub bracer_proj_speed_per_level: Option<f32>,
    pub spellbinder_duration_per_level: Option<f32>,
    pub duplicator_projectiles_per_level: Option<u32>,
    pub pummarola_regen_per_level: Option<f32>,
}

/// Per-level stat bonuses for each passive item type.
///
/// Loaded from `assets/config/passive.ron` and hot-reloaded while the game
/// is running. Systems that read via [`PassiveParams`] pick up new values
/// immediately — however, already-applied bonuses on live `PlayerStats` are
/// not retroactively adjusted; changes take effect on the next upgrade.
#[derive(Asset, TypePath, Debug, Clone)]
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

impl From<PassiveConfigPartial> for PassiveConfig {
    fn from(p: PassiveConfigPartial) -> Self {
        PassiveConfig {
            spinach_damage_per_level: p.spinach_damage_per_level.unwrap_or_else(|| {
                warn!(
                    "passive.ron: `spinach_damage_per_level` missing → using default {DEFAULT_SPINACH_DAMAGE_PER_LEVEL}"
                );
                DEFAULT_SPINACH_DAMAGE_PER_LEVEL
            }),
            wings_speed_per_level: p.wings_speed_per_level.unwrap_or_else(|| {
                warn!(
                    "passive.ron: `wings_speed_per_level` missing → using default {DEFAULT_WINGS_SPEED_PER_LEVEL}"
                );
                DEFAULT_WINGS_SPEED_PER_LEVEL
            }),
            hollow_heart_hp_per_level: p.hollow_heart_hp_per_level.unwrap_or_else(|| {
                warn!(
                    "passive.ron: `hollow_heart_hp_per_level` missing → using default {DEFAULT_HOLLOW_HEART_HP_PER_LEVEL}"
                );
                DEFAULT_HOLLOW_HEART_HP_PER_LEVEL
            }),
            clover_luck_per_level: p.clover_luck_per_level.unwrap_or_else(|| {
                warn!(
                    "passive.ron: `clover_luck_per_level` missing → using default {DEFAULT_CLOVER_LUCK_PER_LEVEL}"
                );
                DEFAULT_CLOVER_LUCK_PER_LEVEL
            }),
            empty_tome_cdr_per_level: p.empty_tome_cdr_per_level.unwrap_or_else(|| {
                warn!(
                    "passive.ron: `empty_tome_cdr_per_level` missing → using default {DEFAULT_EMPTY_TOME_CDR_PER_LEVEL}"
                );
                DEFAULT_EMPTY_TOME_CDR_PER_LEVEL
            }),
            bracer_proj_speed_per_level: p.bracer_proj_speed_per_level.unwrap_or_else(|| {
                warn!(
                    "passive.ron: `bracer_proj_speed_per_level` missing → using default {DEFAULT_BRACER_PROJ_SPEED_PER_LEVEL}"
                );
                DEFAULT_BRACER_PROJ_SPEED_PER_LEVEL
            }),
            spellbinder_duration_per_level: p
                .spellbinder_duration_per_level
                .unwrap_or_else(|| {
                    warn!(
                        "passive.ron: `spellbinder_duration_per_level` missing → using default {DEFAULT_SPELLBINDER_DURATION_PER_LEVEL}"
                    );
                    DEFAULT_SPELLBINDER_DURATION_PER_LEVEL
                }),
            duplicator_projectiles_per_level: p
                .duplicator_projectiles_per_level
                .unwrap_or_else(|| {
                    warn!(
                        "passive.ron: `duplicator_projectiles_per_level` missing → using default {DEFAULT_DUPLICATOR_PROJECTILES_PER_LEVEL}"
                    );
                    DEFAULT_DUPLICATOR_PROJECTILES_PER_LEVEL
                }),
            pummarola_regen_per_level: p.pummarola_regen_per_level.unwrap_or_else(|| {
                warn!(
                    "passive.ron: `pummarola_regen_per_level` missing → using default {DEFAULT_PUMMAROLA_REGEN_PER_LEVEL}"
                );
                DEFAULT_PUMMAROLA_REGEN_PER_LEVEL
            }),
        }
    }
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

    pub fn spinach_damage_per_level(&self) -> f32 {
        self.get()
            .map(|c| c.spinach_damage_per_level)
            .unwrap_or(DEFAULT_SPINACH_DAMAGE_PER_LEVEL)
    }

    pub fn wings_speed_per_level(&self) -> f32 {
        self.get()
            .map(|c| c.wings_speed_per_level)
            .unwrap_or(DEFAULT_WINGS_SPEED_PER_LEVEL)
    }

    pub fn hollow_heart_hp_per_level(&self) -> f32 {
        self.get()
            .map(|c| c.hollow_heart_hp_per_level)
            .unwrap_or(DEFAULT_HOLLOW_HEART_HP_PER_LEVEL)
    }

    pub fn clover_luck_per_level(&self) -> f32 {
        self.get()
            .map(|c| c.clover_luck_per_level)
            .unwrap_or(DEFAULT_CLOVER_LUCK_PER_LEVEL)
    }

    pub fn empty_tome_cdr_per_level(&self) -> f32 {
        self.get()
            .map(|c| c.empty_tome_cdr_per_level)
            .unwrap_or(DEFAULT_EMPTY_TOME_CDR_PER_LEVEL)
    }

    pub fn bracer_proj_speed_per_level(&self) -> f32 {
        self.get()
            .map(|c| c.bracer_proj_speed_per_level)
            .unwrap_or(DEFAULT_BRACER_PROJ_SPEED_PER_LEVEL)
    }

    pub fn spellbinder_duration_per_level(&self) -> f32 {
        self.get()
            .map(|c| c.spellbinder_duration_per_level)
            .unwrap_or(DEFAULT_SPELLBINDER_DURATION_PER_LEVEL)
    }

    pub fn duplicator_projectiles_per_level(&self) -> u32 {
        self.get()
            .map(|c| c.duplicator_projectiles_per_level)
            .unwrap_or(DEFAULT_DUPLICATOR_PROJECTILES_PER_LEVEL)
    }

    pub fn pummarola_regen_per_level(&self) -> f32 {
        self.get()
            .map(|c| c.pummarola_regen_per_level)
            .unwrap_or(DEFAULT_PUMMAROLA_REGEN_PER_LEVEL)
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
        let partial: PassiveConfigPartial =
            ron::Options::default().with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME).from_str(ron_data).expect("RON parse must succeed");
        let cfg = PassiveConfig::from(partial);
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
        let partial: PassiveConfigPartial =
            ron::Options::default().with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME).from_str(ron_data).expect("RON parse must succeed");
        let cfg = PassiveConfig::from(partial);
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
