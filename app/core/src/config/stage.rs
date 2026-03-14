//! Stage configuration loaded from `assets/config/stage.ron`.
//!
//! Each stage defines the enemy types that may spawn, multipliers for
//! HP/speed/spawn-rate, and boss strength.  `StageParams` exposes these
//! values to any system that needs them.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

use crate::types::{EnemyType, StageType};

// ---------------------------------------------------------------------------
// Fallback constants — Mad Forest (baseline, ×1.0)
// ---------------------------------------------------------------------------

const DEFAULT_MAD_FOREST_HP_MULT: f32 = 1.0;
const DEFAULT_MAD_FOREST_SPEED_MULT: f32 = 1.0;
const DEFAULT_MAD_FOREST_SPAWN_INTERVAL_MULT: f32 = 1.0;
const DEFAULT_MAD_FOREST_MAX_ENEMIES_MULT: f32 = 1.0;
const DEFAULT_MAD_FOREST_BOSS_HP_MULT: f32 = 1.0;
const DEFAULT_MAD_FOREST_BOSS_SPEED_MULT: f32 = 1.0;

// ---------------------------------------------------------------------------
// Fallback constants — Inlaid Library (medium, HP ×1.2, speed ×1.1)
// ---------------------------------------------------------------------------

const DEFAULT_INLAID_LIBRARY_HP_MULT: f32 = 1.2;
const DEFAULT_INLAID_LIBRARY_SPEED_MULT: f32 = 1.1;
const DEFAULT_INLAID_LIBRARY_SPAWN_INTERVAL_MULT: f32 = 0.9;
const DEFAULT_INLAID_LIBRARY_MAX_ENEMIES_MULT: f32 = 1.1;
const DEFAULT_INLAID_LIBRARY_BOSS_HP_MULT: f32 = 1.2;
const DEFAULT_INLAID_LIBRARY_BOSS_SPEED_MULT: f32 = 1.1;

// ---------------------------------------------------------------------------
// Fallback constants — Dairy Plant (hard, HP ×1.5, speed ×1.2)
// ---------------------------------------------------------------------------

const DEFAULT_DAIRY_PLANT_HP_MULT: f32 = 1.5;
const DEFAULT_DAIRY_PLANT_SPEED_MULT: f32 = 1.2;
const DEFAULT_DAIRY_PLANT_SPAWN_INTERVAL_MULT: f32 = 0.8;
const DEFAULT_DAIRY_PLANT_MAX_ENEMIES_MULT: f32 = 1.2;
const DEFAULT_DAIRY_PLANT_BOSS_HP_MULT: f32 = 1.5;
const DEFAULT_DAIRY_PLANT_BOSS_SPEED_MULT: f32 = 1.2;

// ---------------------------------------------------------------------------
// Partial structs for deserialization
// ---------------------------------------------------------------------------

/// Deserialization mirror of [`StageEntryConfig`] — every field is `Option<T>`
/// so RON blocks with missing fields still load and emit `warn!`.
#[derive(Deserialize, Default)]
#[serde(default)]
pub(super) struct StageEntryConfigPartial {
    pub display_name: Option<String>,
    pub enemy_types: Option<Vec<EnemyType>>,
    pub enemy_hp_multiplier: Option<f32>,
    pub enemy_speed_multiplier: Option<f32>,
    pub spawn_interval_multiplier: Option<f32>,
    pub max_enemies_multiplier: Option<f32>,
    pub boss_hp_multiplier: Option<f32>,
    pub boss_speed_multiplier: Option<f32>,
}

/// Deserialization mirror of [`StageConfig`] — every field is `Option<T>`.
#[derive(Deserialize, Default)]
#[serde(default, rename = "StageConfig")]
pub(super) struct StageConfigPartial {
    pub mad_forest: Option<StageEntryConfigPartial>,
    pub inlaid_library: Option<StageEntryConfigPartial>,
    pub dairy_plant: Option<StageEntryConfigPartial>,
}

// ---------------------------------------------------------------------------
// Full config types
// ---------------------------------------------------------------------------

/// Per-stage gameplay configuration.
#[derive(Debug, Clone)]
pub struct StageEntryConfig {
    /// Display name of the stage (e.g. `"Mad Forest"`).
    pub display_name: String,
    /// Which enemy types may appear in this stage.
    pub enemy_types: Vec<EnemyType>,
    /// Multiplier applied to enemy HP (stacks on top of difficulty scaling).
    pub enemy_hp_multiplier: f32,
    /// Multiplier applied to enemy movement speed.
    pub enemy_speed_multiplier: f32,
    /// Multiplier applied to the spawn interval (<1.0 → more frequent spawns).
    pub spawn_interval_multiplier: f32,
    /// Multiplier applied to the maximum simultaneous enemy count.
    pub max_enemies_multiplier: f32,
    /// Multiplier applied to boss HP.
    pub boss_hp_multiplier: f32,
    /// Multiplier applied to boss movement speed.
    pub boss_speed_multiplier: f32,
}

/// Full stage configuration loaded from `assets/config/stage.ron`.
///
/// Contains one [`StageEntryConfig`] per selectable stage.
/// Use [`StageConfig::entry_for`] to look up a stage by type.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct StageConfig {
    pub mad_forest: StageEntryConfig,
    pub inlaid_library: StageEntryConfig,
    pub dairy_plant: StageEntryConfig,
}

impl StageConfig {
    /// Returns the entry config for the given [`StageType`].
    pub fn entry_for(&self, stage: StageType) -> &StageEntryConfig {
        match stage {
            StageType::MadForest => &self.mad_forest,
            StageType::InlaidLibrary => &self.inlaid_library,
            StageType::DairyPlant => &self.dairy_plant,
        }
    }
}

// ---------------------------------------------------------------------------
// From<Partial> conversion
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn entry_from_partial(
    partial: StageEntryConfigPartial,
    default_name: &str,
    default_enemy_types: Vec<EnemyType>,
    default_hp: f32,
    default_speed: f32,
    default_spawn_interval: f32,
    default_max_enemies: f32,
    default_boss_hp: f32,
    default_boss_speed: f32,
    field_prefix: &str,
) -> StageEntryConfig {
    StageEntryConfig {
        display_name: partial.display_name.unwrap_or_else(|| {
            warn!("stage.ron: `{field_prefix}.display_name` missing → using built-in baseline");
            default_name.to_string()
        }),
        enemy_types: partial.enemy_types.unwrap_or_else(|| {
            warn!("stage.ron: `{field_prefix}.enemy_types` missing → using built-in baseline");
            default_enemy_types
        }),
        enemy_hp_multiplier: partial.enemy_hp_multiplier.unwrap_or_else(|| {
            warn!(
                "stage.ron: `{field_prefix}.enemy_hp_multiplier` missing → using built-in baseline"
            );
            default_hp
        }),
        enemy_speed_multiplier: partial.enemy_speed_multiplier.unwrap_or_else(|| {
            warn!("stage.ron: `{field_prefix}.enemy_speed_multiplier` missing → using built-in baseline");
            default_speed
        }),
        spawn_interval_multiplier: partial.spawn_interval_multiplier.unwrap_or_else(|| {
            warn!("stage.ron: `{field_prefix}.spawn_interval_multiplier` missing → using built-in baseline");
            default_spawn_interval
        }),
        max_enemies_multiplier: partial.max_enemies_multiplier.unwrap_or_else(|| {
            warn!("stage.ron: `{field_prefix}.max_enemies_multiplier` missing → using built-in baseline");
            default_max_enemies
        }),
        boss_hp_multiplier: partial.boss_hp_multiplier.unwrap_or_else(|| {
            warn!(
                "stage.ron: `{field_prefix}.boss_hp_multiplier` missing → using built-in baseline"
            );
            default_boss_hp
        }),
        boss_speed_multiplier: partial.boss_speed_multiplier.unwrap_or_else(|| {
            warn!("stage.ron: `{field_prefix}.boss_speed_multiplier` missing → using built-in baseline");
            default_boss_speed
        }),
    }
}

impl From<StageConfigPartial> for StageConfig {
    fn from(p: StageConfigPartial) -> Self {
        StageConfig {
            mad_forest: entry_from_partial(
                p.mad_forest.unwrap_or_default(),
                "Mad Forest",
                vec![EnemyType::Bat, EnemyType::Skeleton],
                DEFAULT_MAD_FOREST_HP_MULT,
                DEFAULT_MAD_FOREST_SPEED_MULT,
                DEFAULT_MAD_FOREST_SPAWN_INTERVAL_MULT,
                DEFAULT_MAD_FOREST_MAX_ENEMIES_MULT,
                DEFAULT_MAD_FOREST_BOSS_HP_MULT,
                DEFAULT_MAD_FOREST_BOSS_SPEED_MULT,
                "mad_forest",
            ),
            inlaid_library: entry_from_partial(
                p.inlaid_library.unwrap_or_default(),
                "Inlaid Library",
                vec![EnemyType::Zombie, EnemyType::Ghost],
                DEFAULT_INLAID_LIBRARY_HP_MULT,
                DEFAULT_INLAID_LIBRARY_SPEED_MULT,
                DEFAULT_INLAID_LIBRARY_SPAWN_INTERVAL_MULT,
                DEFAULT_INLAID_LIBRARY_MAX_ENEMIES_MULT,
                DEFAULT_INLAID_LIBRARY_BOSS_HP_MULT,
                DEFAULT_INLAID_LIBRARY_BOSS_SPEED_MULT,
                "inlaid_library",
            ),
            dairy_plant: entry_from_partial(
                p.dairy_plant.unwrap_or_default(),
                "Dairy Plant",
                vec![EnemyType::Demon, EnemyType::Medusa],
                DEFAULT_DAIRY_PLANT_HP_MULT,
                DEFAULT_DAIRY_PLANT_SPEED_MULT,
                DEFAULT_DAIRY_PLANT_SPAWN_INTERVAL_MULT,
                DEFAULT_DAIRY_PLANT_MAX_ENEMIES_MULT,
                DEFAULT_DAIRY_PLANT_BOSS_HP_MULT,
                DEFAULT_DAIRY_PLANT_BOSS_SPEED_MULT,
                "dairy_plant",
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// Resource + SystemParam
// ---------------------------------------------------------------------------

/// Resource holding the handle to the loaded stage configuration.
#[derive(Resource)]
pub struct StageConfigHandle(pub Handle<StageConfig>);

/// SystemParam bundle for accessing [`StageConfig`].
///
/// Returns `None` while the asset is still loading or when
/// [`GameConfigPlugin`] has not been registered (e.g. in unit tests).
/// Call `.get()` to obtain `Option<&StageConfig>`.
///
/// [`GameConfigPlugin`]: crate::config::GameConfigPlugin
#[derive(SystemParam)]
pub struct StageParams<'w> {
    handle: Option<Res<'w, StageConfigHandle>>,
    assets: Option<Res<'w, Assets<StageConfig>>>,
}

impl<'w> StageParams<'w> {
    /// Returns the currently loaded [`StageConfig`], or `None` while loading.
    pub fn get(&self) -> Option<&StageConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }
}

// ---------------------------------------------------------------------------
// Hot-reload system
// ---------------------------------------------------------------------------

/// Handles hot-reloading of stage configuration.
pub fn hot_reload_stage_config(mut events: MessageReader<AssetEvent<StageConfig>>) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id: _ } => {
                info!("✅ Stage config loaded");
            }
            AssetEvent::Modified { id: _ } => {
                info!("🔥 Hot-reloading stage config!");
            }
            AssetEvent::Removed { id: _ } => {
                warn!("⚠️ Stage config removed");
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_ron() -> &'static str {
        r#"
StageConfig(
    mad_forest: (
        display_name: "Mad Forest",
        enemy_types: [Bat, Skeleton],
        enemy_hp_multiplier: 1.0,
        enemy_speed_multiplier: 1.0,
        spawn_interval_multiplier: 1.0,
        max_enemies_multiplier: 1.0,
        boss_hp_multiplier: 1.0,
        boss_speed_multiplier: 1.0,
    ),
    inlaid_library: (
        display_name: "Inlaid Library",
        enemy_types: [Zombie, Ghost],
        enemy_hp_multiplier: 1.2,
        enemy_speed_multiplier: 1.1,
        spawn_interval_multiplier: 0.9,
        max_enemies_multiplier: 1.1,
        boss_hp_multiplier: 1.2,
        boss_speed_multiplier: 1.1,
    ),
    dairy_plant: (
        display_name: "Dairy Plant",
        enemy_types: [Demon, Medusa],
        enemy_hp_multiplier: 1.5,
        enemy_speed_multiplier: 1.2,
        spawn_interval_multiplier: 0.8,
        max_enemies_multiplier: 1.2,
        boss_hp_multiplier: 1.5,
        boss_speed_multiplier: 1.2,
    ),
)
"#
    }

    #[test]
    fn stage_config_deserializes() {
        let partial: StageConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(sample_ron())
            .unwrap();
        let config = StageConfig::from(partial);

        assert_eq!(config.mad_forest.display_name, "Mad Forest");
        assert_eq!(config.mad_forest.enemy_hp_multiplier, 1.0);
        assert!(config.mad_forest.enemy_types.contains(&EnemyType::Bat));
        assert!(config.mad_forest.enemy_types.contains(&EnemyType::Skeleton));

        assert_eq!(config.inlaid_library.enemy_hp_multiplier, 1.2);
        assert_eq!(config.inlaid_library.enemy_speed_multiplier, 1.1);
        assert!(
            config
                .inlaid_library
                .enemy_types
                .contains(&EnemyType::Zombie)
        );
        assert!(
            config
                .inlaid_library
                .enemy_types
                .contains(&EnemyType::Ghost)
        );

        assert_eq!(config.dairy_plant.enemy_hp_multiplier, 1.5);
        assert_eq!(config.dairy_plant.enemy_speed_multiplier, 1.2);
        assert!(config.dairy_plant.enemy_types.contains(&EnemyType::Demon));
        assert!(config.dairy_plant.enemy_types.contains(&EnemyType::Medusa));
    }

    #[test]
    fn entry_for_returns_correct_stage() {
        let partial: StageConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(sample_ron())
            .unwrap();
        let config = StageConfig::from(partial);

        assert_eq!(
            config.entry_for(StageType::MadForest).display_name,
            "Mad Forest"
        );
        assert_eq!(
            config.entry_for(StageType::InlaidLibrary).display_name,
            "Inlaid Library"
        );
        assert_eq!(
            config.entry_for(StageType::DairyPlant).display_name,
            "Dairy Plant"
        );
    }

    #[test]
    fn all_entries_have_positive_multipliers() {
        let partial: StageConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(sample_ron())
            .unwrap();
        let config = StageConfig::from(partial);

        for stage in [
            StageType::MadForest,
            StageType::InlaidLibrary,
            StageType::DairyPlant,
        ] {
            let entry = config.entry_for(stage);
            assert!(
                entry.enemy_hp_multiplier > 0.0,
                "{:?} hp_mult must be > 0",
                stage
            );
            assert!(
                entry.enemy_speed_multiplier > 0.0,
                "{:?} speed_mult must be > 0",
                stage
            );
            assert!(
                entry.boss_hp_multiplier > 0.0,
                "{:?} boss_hp_mult must be > 0",
                stage
            );
        }
    }

    #[test]
    fn missing_fields_use_defaults() {
        let partial: StageConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str("StageConfig()")
            .unwrap();
        let config = StageConfig::from(partial);

        // Mad Forest defaults
        assert_eq!(
            config.mad_forest.enemy_hp_multiplier,
            DEFAULT_MAD_FOREST_HP_MULT
        );
        assert_eq!(
            config.mad_forest.enemy_speed_multiplier,
            DEFAULT_MAD_FOREST_SPEED_MULT
        );
    }
}
