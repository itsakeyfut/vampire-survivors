//! Spatial grid performance benchmarks.
//!
//! Measures the per-frame cost of [`SpatialGrid`] at enemy counts of 300 and
//! 500, and compares grid-assisted collision queries against a baseline linear
//! scan.  This validates the acceptance criteria for issue #28:
//!
//! - 300 enemies: grid build + full collision sweep < 1 ms
//! - 500 enemies: grid build + full collision sweep < 2 ms
//! - Grid-based sweep is measurably faster than the baseline at 300+ enemies
//!
//! # Running
//!
//! ```bash
//! cargo bench -p vs-core
//! # or from the workspace root:
//! just bench
//! ```
//!
//! HTML reports are written to `target/criterion/`.

use std::collections::HashMap;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

use bevy::math::Vec2;
use bevy::prelude::{Entity, World};
use vs_core::resources::SpatialGrid;

// ---------------------------------------------------------------------------
// Enemy radius used throughout (matches MAX_ENEMY_COLLIDER_RADIUS in systems)
// ---------------------------------------------------------------------------

const ENEMY_RADIUS: f32 = 32.0;
const PLAYER_RADIUS: f32 = 12.0;
/// Query radius used in collision systems: player_radius + max_enemy_radius.
const QUERY_RADIUS: f32 = PLAYER_RADIUS + ENEMY_RADIUS;
/// Precomputed squared query radius for exact distance checks.
const QUERY_RADIUS_SQ: f32 = QUERY_RADIUS * QUERY_RADIUS;

// ---------------------------------------------------------------------------
// Sample query positions (four screen-corner-like locations)
// ---------------------------------------------------------------------------

const SAMPLE_POSITIONS: [Vec2; 4] = [
    Vec2::ZERO,
    Vec2::new(200.0, 150.0),
    Vec2::new(-300.0, 200.0),
    Vec2::new(400.0, -300.0),
];

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create `n` unique [`Entity`] values without spawning any components.
fn make_entities(n: usize) -> Vec<Entity> {
    let mut world = World::new();
    (0..n).map(|_| world.spawn_empty().id()).collect()
}

/// Generate `n` enemy positions scattered over a 1920×1080-pixel play area.
///
/// Positions are deterministic (no RNG) so benchmark results are reproducible.
/// Enemies are arranged in a roughly uniform grid to simulate a realistic
/// spread across the viewport.
fn generate_positions(n: usize) -> Vec<Vec2> {
    let cols = (n as f32).sqrt().ceil() as usize;
    let rows = n.div_ceil(cols);
    let x_spacing = 1920.0 / cols as f32;
    let y_spacing = 1080.0 / rows as f32;
    (0..n)
        .map(|i| {
            let col = i % cols;
            let row = i / cols;
            Vec2::new(
                col as f32 * x_spacing - 960.0,
                row as f32 * y_spacing - 540.0,
            )
        })
        .collect()
}

/// Build an `Entity → Vec2` lookup map.
///
/// In the real game this corresponds to the ECS query cache that maps enemy
/// entities to their current [`Transform`].  Building it once before the
/// benchmark loop mirrors the ECS's persistent query state.
fn build_pos_map(entities: &[Entity], positions: &[Vec2]) -> HashMap<Entity, Vec2> {
    entities
        .iter()
        .copied()
        .zip(positions.iter().copied())
        .collect()
}

/// Build a [`SpatialGrid`] populated with `n` enemies at `positions`.
fn build_grid(positions: &[Vec2], entities: &[Entity]) -> SpatialGrid {
    let mut grid = SpatialGrid::default();
    for (pos, entity) in positions.iter().zip(entities.iter()) {
        grid.insert(*pos, *entity);
    }
    grid
}

/// Simulate the full per-query collision flow used by `enemy_player_collision`.
///
/// For each sample player position:
/// 1. Call [`SpatialGrid::get_nearby`] to retrieve candidates (may include
///    false positives from the same grid cell).
/// 2. Perform an exact squared-distance check on each candidate to filter
///    false positives — mirroring [`check_circle_collision`].
///
/// The `pos_map` argument represents the ECS entity→transform lookup (O(1)
/// hash access per candidate, matching Bevy's query performance).
fn grid_query_with_exact_check(grid: &SpatialGrid, pos_map: &HashMap<Entity, Vec2>) -> usize {
    let mut hits = 0usize;
    for &player_pos in &SAMPLE_POSITIONS {
        let candidates = grid.get_nearby(player_pos, QUERY_RADIUS);
        for entity in candidates {
            if let Some(&enemy_pos) = pos_map.get(&entity) {
                if enemy_pos.distance_squared(player_pos) < QUERY_RADIUS_SQ {
                    hits += 1;
                }
            }
        }
    }
    hits
}

/// Baseline linear scan: check all `n` enemy positions against each query
/// point without any spatial partitioning.
///
/// This benchmarks 4 fixed query points scanned over all `n` enemy positions,
/// so the complexity is O(n) in the number of enemies (not O(n²)).  Its purpose
/// is to show that the grid reduces the per-query work from scanning all `n`
/// enemies to scanning only the small candidate set returned by `get_nearby`.
fn brute_force_query(positions: &[Vec2]) -> usize {
    let mut hits = 0usize;
    for &player_pos in &SAMPLE_POSITIONS {
        for &pos in positions {
            if pos.distance_squared(player_pos) < QUERY_RADIUS_SQ {
                hits += 1;
            }
        }
    }
    hits
}

// ---------------------------------------------------------------------------
// Benchmark: grid build (clear + insert all enemies)
// ---------------------------------------------------------------------------

/// Measures the per-frame cost of clearing and rebuilding the spatial grid.
///
/// This corresponds to `update_spatial_grid` running once at the start of
/// each frame.
fn bench_grid_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("grid_build");

    for &n in &[300usize, 500] {
        let positions = generate_positions(n);
        let entities = make_entities(n);

        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let _ = build_grid(&positions, &entities);
            });
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: grid query (get_nearby + exact distance check)
// ---------------------------------------------------------------------------

/// Measures collision query cost using the spatial grid.
///
/// The grid and position map are pre-built once outside the benchmark loop.
/// Each iteration runs `get_nearby` followed by exact squared-distance checks
/// on all candidates — mirroring the full `enemy_player_collision` hot path.
fn bench_grid_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("grid_query");

    for &n in &[300usize, 500] {
        let positions = generate_positions(n);
        let entities = make_entities(n);
        let grid = build_grid(&positions, &entities);
        let pos_map = build_pos_map(&entities, &positions);

        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| grid_query_with_exact_check(&grid, &pos_map));
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: baseline linear scan comparison
// ---------------------------------------------------------------------------

/// Measures the baseline O(n) linear scan cost without the grid.
///
/// Scans all `n` enemy positions for each of 4 fixed query points (total
/// 4 × n distance checks per iteration).  Used as a baseline to confirm that
/// `SpatialGrid` reduces the per-query candidate set and thus lowers the
/// number of exact distance checks at 300+ enemies.
fn bench_brute_force(c: &mut Criterion) {
    let mut group = c.benchmark_group("brute_force");

    for &n in &[300usize, 500] {
        let positions = generate_positions(n);

        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| brute_force_query(&positions));
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: full frame simulation (build + query with exact check)
// ---------------------------------------------------------------------------

/// Simulates a complete collision frame: build the grid, then run queries with
/// exact distance filtering.
///
/// This is the closest approximation to the actual per-frame wall-clock cost of
/// the spatial grid subsystem.  At 60 fps the entire frame budget is ~16.7 ms;
/// the spatial grid portion should consume well under 1 ms for 300 enemies.
fn bench_full_frame(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_frame");

    for &n in &[300usize, 500] {
        let positions = generate_positions(n);
        let entities = make_entities(n);
        let pos_map = build_pos_map(&entities, &positions);

        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let grid = build_grid(&positions, &entities);
                grid_query_with_exact_check(&grid, &pos_map)
            });
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Registration
// ---------------------------------------------------------------------------

criterion_group!(
    benches,
    bench_grid_build,
    bench_grid_query,
    bench_brute_force,
    bench_full_frame,
);
criterion_main!(benches);
