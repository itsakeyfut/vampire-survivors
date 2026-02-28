//! Spatial grid performance benchmarks.
//!
//! Measures the per-frame cost of [`SpatialGrid`] at enemy counts of 300 and
//! 500, and compares grid-assisted collision queries against a naïve O(n²)
//! brute-force scan.  This validates the acceptance criteria for issue #28:
//!
//! - 300 enemies: grid build + full collision sweep < 1 ms
//! - 500 enemies: grid build + full collision sweep < 2 ms
//! - Grid-based sweep is measurably faster than brute-force at 300+ enemies
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

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

use bevy::math::Vec2;
use bevy::prelude::World;
use vs_core::resources::SpatialGrid;

// ---------------------------------------------------------------------------
// Enemy radius used throughout (matches MAX_ENEMY_COLLIDER_RADIUS in systems)
// ---------------------------------------------------------------------------

const ENEMY_RADIUS: f32 = 32.0;
const PLAYER_RADIUS: f32 = 12.0;
/// Query radius used in collision systems: player_radius + max_enemy_radius.
const QUERY_RADIUS: f32 = PLAYER_RADIUS + ENEMY_RADIUS;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create `n` unique [`Entity`] values without spawning any components.
fn make_entities(n: usize) -> Vec<bevy::prelude::Entity> {
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

/// Build a [`SpatialGrid`] populated with `n` enemies at `positions`.
fn build_grid(positions: &[Vec2], entities: &[bevy::prelude::Entity]) -> SpatialGrid {
    let mut grid = SpatialGrid::default();
    for (pos, entity) in positions.iter().zip(entities.iter()) {
        grid.insert(*pos, *entity);
    }
    grid
}

/// Simulate collision queries from a single player position against the grid.
///
/// Mirrors what `enemy_player_collision` does each frame: query the grid for
/// candidates, then (conceptually) perform exact distance checks.  The query
/// itself is the hot path — this benchmark measures just `get_nearby`.
fn grid_query_all_positions(grid: &SpatialGrid, positions: &[Vec2]) {
    // Simulate checking from multiple player positions across the map.
    let sample_positions = [
        Vec2::ZERO,
        Vec2::new(200.0, 150.0),
        Vec2::new(-300.0, 200.0),
        Vec2::new(400.0, -300.0),
    ];
    for &player_pos in &sample_positions {
        let _ = grid.get_nearby(player_pos, QUERY_RADIUS);
    }
}

/// Brute-force O(n²) scan: check every enemy position against every query
/// position without any spatial partitioning.
///
/// This mirrors what collision detection would cost without `SpatialGrid`.
fn brute_force_query(positions: &[Vec2]) {
    let sample_positions = [
        Vec2::ZERO,
        Vec2::new(200.0, 150.0),
        Vec2::new(-300.0, 200.0),
        Vec2::new(400.0, -300.0),
    ];
    for &player_pos in &sample_positions {
        let _hits: Vec<_> = positions
            .iter()
            .filter(|&&pos| pos.distance_squared(player_pos) < (QUERY_RADIUS * QUERY_RADIUS))
            .collect();
    }
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
// Benchmark: grid query (get_nearby from a built grid)
// ---------------------------------------------------------------------------

/// Measures collision query cost using the spatial grid.
///
/// The grid is pre-built once outside the benchmark loop; only the
/// `get_nearby` calls are measured.
fn bench_grid_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("grid_query");

    for &n in &[300usize, 500] {
        let positions = generate_positions(n);
        let entities = make_entities(n);
        let grid = build_grid(&positions, &entities);

        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                grid_query_all_positions(&grid, &positions);
            });
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: brute-force O(n²) comparison
// ---------------------------------------------------------------------------

/// Measures the brute-force O(n²) collision detection cost without the grid.
///
/// Used as a baseline to confirm that `SpatialGrid` provides a meaningful
/// speedup at 300+ enemies.
fn bench_brute_force(c: &mut Criterion) {
    let mut group = c.benchmark_group("brute_force");

    for &n in &[300usize, 500] {
        let positions = generate_positions(n);

        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                brute_force_query(&positions);
            });
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: full frame simulation (build + query)
// ---------------------------------------------------------------------------

/// Simulates a complete collision frame: build the grid then run queries.
///
/// This is the closest approximation to the per-frame wall-clock budget.
/// At 60 fps the entire frame budget is ~16.7 ms; the spatial grid portion
/// should consume well under 1 ms for 300 enemies.
fn bench_full_frame(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_frame");

    for &n in &[300usize, 500] {
        let positions = generate_positions(n);
        let entities = make_entities(n);

        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let grid = build_grid(&positions, &entities);
                grid_query_all_positions(&grid, &positions);
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
