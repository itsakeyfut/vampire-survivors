use std::collections::HashMap;

use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Fallback constants (used when RON config is not yet loaded)
// ---------------------------------------------------------------------------

/// Grid cell size for spatial partitioning (pixels).
const DEFAULT_SPATIAL_GRID_CELL_SIZE: f32 = 64.0;

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
        Self::new(DEFAULT_SPATIAL_GRID_CELL_SIZE)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entities(n: usize) -> Vec<Entity> {
        // Spawn all entities from a single World so each gets a unique ID.
        let mut world = World::new();
        (0..n).map(|_| world.spawn_empty().id()).collect()
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
    fn spatial_grid_insert_and_get_nearby() {
        let entity = make_entities(1)[0];
        let mut grid = SpatialGrid::new(64.0);
        grid.insert(Vec2::ZERO, entity);

        let nearby = grid.get_nearby(Vec2::ZERO, 32.0);
        assert!(nearby.contains(&entity));
    }

    #[test]
    fn spatial_grid_clear() {
        let entity = make_entities(1)[0];
        let mut grid = SpatialGrid::new(64.0);
        grid.insert(Vec2::ZERO, entity);
        grid.clear();
        assert!(grid.get_nearby(Vec2::ZERO, 100.0).is_empty());
    }

    #[test]
    fn spatial_grid_entity_not_in_distant_cell() {
        let entity = make_entities(1)[0];
        let mut grid = SpatialGrid::new(64.0);
        // Insert far from origin
        grid.insert(Vec2::new(1000.0, 1000.0), entity);
        // Query near origin should not include it
        let nearby = grid.get_nearby(Vec2::ZERO, 10.0);
        assert!(!nearby.contains(&entity));
    }
}
