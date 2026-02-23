use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    constants::SPATIAL_GRID_CELL_SIZE,
    types::{CharacterType, MetaUpgradeType, UpgradeChoice},
};

// ---------------------------------------------------------------------------
// Core game state
// ---------------------------------------------------------------------------

/// Global game-session data. Reset at the start of each run.
#[derive(Resource, Debug)]
pub struct GameData {
    /// Seconds elapsed since the run started (paused during LevelUp/Paused).
    pub elapsed_time: f32,
    /// Current player level (1-indexed).
    pub current_level: u32,
    /// Accumulated XP within the current level.
    pub current_xp: u32,
    /// XP required to reach the next level.
    pub xp_to_next_level: u32,
    /// Total enemies defeated this run.
    pub kill_count: u32,
    /// Gold collected this run (added to MetaProgress on run end).
    pub gold_earned: u32,
    /// True once Boss Death has been spawned.
    pub boss_spawned: bool,
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            elapsed_time: 0.0,
            current_level: 1,
            current_xp: 0,
            xp_to_next_level: crate::constants::XP_LEVEL_BASE,
            kill_count: 0,
            gold_earned: 0,
            boss_spawned: false,
        }
    }
}

/// Which character the player selected on the character-select screen.
#[derive(Resource, Debug)]
pub struct SelectedCharacter(pub CharacterType);

impl Default for SelectedCharacter {
    fn default() -> Self {
        Self(CharacterType::DefaultCharacter)
    }
}

// ---------------------------------------------------------------------------
// Spawner resources
// ---------------------------------------------------------------------------

/// Controls enemy spawn timing and difficulty scaling.
#[derive(Resource, Debug)]
pub struct EnemySpawner {
    /// Accumulated time since the last spawn.
    pub spawn_timer: f32,
    /// Current interval between spawns (seconds). Decreases over time.
    pub spawn_interval: f32,
    /// Multiplier applied to enemy stats as time progresses.
    pub difficulty_multiplier: f32,
}

impl Default for EnemySpawner {
    fn default() -> Self {
        Self {
            spawn_timer: 0.0,
            spawn_interval: crate::constants::ENEMY_SPAWN_BASE_INTERVAL,
            difficulty_multiplier: 1.0,
        }
    }
}

/// Controls treasure chest spawn timing.
#[derive(Resource, Debug)]
pub struct TreasureSpawner {
    /// Accumulated time since the last chest spawn.
    pub spawn_timer: f32,
}

impl Default for TreasureSpawner {
    fn default() -> Self {
        Self { spawn_timer: 0.0 }
    }
}

// ---------------------------------------------------------------------------
// Level-up choices
// ---------------------------------------------------------------------------

/// Holds the current set of upgrade cards shown during a level-up.
#[derive(Resource, Debug, Default)]
pub struct LevelUpChoices {
    pub choices: Vec<UpgradeChoice>,
}

// ---------------------------------------------------------------------------
// Meta progression
// ---------------------------------------------------------------------------

/// Persistent cross-run data. Loaded from `save/meta.json` at startup.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct MetaProgress {
    /// Total gold accumulated across all runs.
    pub total_gold: u32,
    /// Characters that have been unlocked via the gold shop.
    pub unlocked_characters: Vec<CharacterType>,
    /// Permanent upgrades that have been purchased.
    pub purchased_upgrades: Vec<MetaUpgradeType>,
}

impl Default for MetaProgress {
    fn default() -> Self {
        Self {
            total_gold: 0,
            unlocked_characters: vec![CharacterType::DefaultCharacter],
            purchased_upgrades: vec![],
        }
    }
}

impl MetaProgress {
    /// Load meta-progression from `save/meta.json`.
    /// Falls back to default values if the file does not exist or is corrupt.
    pub fn load() -> Self {
        // TODO: implement file I/O in Phase 14
        Self::default()
    }

    /// Save meta-progression to `save/meta.json`.
    pub fn save(&self) {
        // TODO: implement file I/O in Phase 14
    }
}

// ---------------------------------------------------------------------------
// Spatial grid
// ---------------------------------------------------------------------------

/// Grid-based spatial partitioning used to accelerate collision detection.
///
/// Each frame the grid is cleared and rebuilt from current entity positions.
/// Queries then fetch only the candidate entities in nearby cells rather than
/// iterating over every entity (O(n) instead of O(n²)).
#[derive(Resource, Debug)]
pub struct SpatialGrid {
    /// Cell size in pixels. Typical value: `SPATIAL_GRID_CELL_SIZE` (64 px).
    pub cell_size: f32,
    /// Maps grid cell coordinates to the entities currently in that cell.
    pub cells: HashMap<(i32, i32), Vec<Entity>>,
}

impl SpatialGrid {
    /// Create a new, empty grid with the given cell size.
    ///
    /// # Panics
    ///
    /// Panics if `cell_size` is not positive.
    pub fn new(cell_size: f32) -> Self {
        assert!(cell_size > 0.0, "SpatialGrid cell_size must be positive");
        Self {
            cell_size,
            cells: HashMap::new(),
        }
    }

    /// Remove all entities from the grid. Call once per frame before re-inserting.
    pub fn clear(&mut self) {
        self.cells.clear();
    }

    /// Insert an entity at world position `pos`.
    pub fn insert(&mut self, pos: Vec2, entity: Entity) {
        let cell = self.pos_to_cell(pos);
        self.cells.entry(cell).or_default().push(entity);
    }

    /// Return all entities whose cells overlap a circle of `radius` around `pos`.
    ///
    /// May return false positives (entities in the same cell but outside the
    /// circle); callers should perform exact distance checks afterwards.
    pub fn get_nearby(&self, pos: Vec2, radius: f32) -> Vec<Entity> {
        let min_cell = self.pos_to_cell(pos - Vec2::splat(radius));
        let max_cell = self.pos_to_cell(pos + Vec2::splat(radius));

        let mut entities = Vec::new();
        for cx in min_cell.0..=max_cell.0 {
            for cy in min_cell.1..=max_cell.1 {
                if let Some(cell_entities) = self.cells.get(&(cx, cy)) {
                    entities.extend_from_slice(cell_entities);
                }
            }
        }
        entities
    }

    fn pos_to_cell(&self, pos: Vec2) -> (i32, i32) {
        (
            (pos.x / self.cell_size).floor() as i32,
            (pos.y / self.cell_size).floor() as i32,
        )
    }
}

impl Default for SpatialGrid {
    fn default() -> Self {
        Self::new(SPATIAL_GRID_CELL_SIZE)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_data_default() {
        let gd = GameData::default();
        assert_eq!(gd.elapsed_time, 0.0);
        assert_eq!(gd.current_level, 1);
        assert_eq!(gd.current_xp, 0);
        assert_eq!(gd.xp_to_next_level, crate::constants::XP_LEVEL_BASE);
        assert_eq!(gd.kill_count, 0);
        assert!(!gd.boss_spawned);
    }

    #[test]
    #[should_panic(expected = "cell_size must be positive")]
    fn spatial_grid_zero_cell_size_panics() {
        let _ = SpatialGrid::new(0.0);
    }

    #[test]
    #[should_panic(expected = "cell_size must be positive")]
    fn spatial_grid_negative_cell_size_panics() {
        let _ = SpatialGrid::new(-1.0);
    }

    #[test]
    fn selected_character_default_is_default_character() {
        let sc = SelectedCharacter::default();
        assert_eq!(sc.0, CharacterType::DefaultCharacter);
    }

    #[test]
    fn meta_progress_default_unlocks_default_character() {
        let mp = MetaProgress::default();
        assert!(
            mp.unlocked_characters
                .contains(&CharacterType::DefaultCharacter)
        );
        assert_eq!(mp.total_gold, 0);
    }

    fn spawn_entity() -> Entity {
        // Use a real World to get a valid Entity with proper bits for Bevy 0.17+.
        World::new().spawn_empty().id()
    }

    #[test]
    fn spatial_grid_insert_and_get_nearby() {
        let entity = spawn_entity();
        let mut grid = SpatialGrid::new(64.0);
        grid.insert(Vec2::ZERO, entity);

        let nearby = grid.get_nearby(Vec2::ZERO, 32.0);
        assert!(nearby.contains(&entity));
    }

    #[test]
    fn spatial_grid_clear() {
        let entity = spawn_entity();
        let mut grid = SpatialGrid::new(64.0);
        grid.insert(Vec2::ZERO, entity);
        grid.clear();
        assert!(grid.get_nearby(Vec2::ZERO, 100.0).is_empty());
    }

    #[test]
    fn spatial_grid_entity_not_in_distant_cell() {
        let entity = spawn_entity();
        let mut grid = SpatialGrid::new(64.0);
        // Insert far from origin
        grid.insert(Vec2::new(1000.0, 1000.0), entity);
        // Query near origin should not include it
        let nearby = grid.get_nearby(Vec2::ZERO, 10.0);
        assert!(!nearby.contains(&entity));
    }
}
