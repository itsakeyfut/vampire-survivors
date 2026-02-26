//! SpatialGrid population system.
//!
//! [`update_spatial_grid`] must run once per frame — before any system that
//! queries the grid (e.g. [`fire_whip`](super::weapon_whip::fire_whip)).
//! It clears the previous frame's data and re-inserts all current enemy
//! positions so that spatial queries reflect the latest world state.

use bevy::prelude::*;

use crate::{components::Enemy, resources::SpatialGrid};

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Clears the [`SpatialGrid`] and re-inserts every enemy's current position.
///
/// Must run before any system that calls [`SpatialGrid::get_nearby`].
/// Systems that rely on the grid should be ordered `.after(update_spatial_grid)`.
pub fn update_spatial_grid(
    mut grid: ResMut<SpatialGrid>,
    enemy_q: Query<(Entity, &Transform), With<Enemy>>,
) {
    grid.clear();
    for (entity, transform) in enemy_q.iter() {
        grid.insert(transform.translation.truncate(), entity);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce as _;

    use super::*;
    use crate::{components::Enemy, resources::SpatialGrid, types::EnemyType};

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(SpatialGrid::default());
        app
    }

    fn spawn_enemy(app: &mut App, pos: Vec2) -> Entity {
        app.world_mut()
            .spawn((
                Enemy::from_type(EnemyType::Bat, 1.0),
                Transform::from_xyz(pos.x, pos.y, 1.0),
            ))
            .id()
    }

    fn run_update(app: &mut App) {
        app.world_mut()
            .run_system_once(update_spatial_grid)
            .expect("update_spatial_grid should run");
    }

    /// After update, an enemy is findable via get_nearby.
    #[test]
    fn enemy_inserted_into_grid() {
        let mut app = build_app();
        let entity = spawn_enemy(&mut app, Vec2::new(50.0, 0.0));

        run_update(&mut app);

        let grid = app.world().resource::<SpatialGrid>();
        let nearby = grid.get_nearby(Vec2::ZERO, 100.0);
        assert!(
            nearby.contains(&entity),
            "enemy should be in grid after update"
        );
    }

    /// Grid is cleared each frame so despawned enemies don't linger.
    #[test]
    fn despawned_enemy_removed_after_update() {
        let mut app = build_app();
        let entity = spawn_enemy(&mut app, Vec2::new(50.0, 0.0));

        run_update(&mut app);

        // Despawn the enemy, then update the grid again.
        app.world_mut().entity_mut(entity).despawn();
        run_update(&mut app);

        let grid = app.world().resource::<SpatialGrid>();
        let nearby = grid.get_nearby(Vec2::ZERO, 100.0);
        assert!(
            !nearby.contains(&entity),
            "despawned enemy must not appear in grid after second update"
        );
    }

    /// No enemies → grid is empty after update.
    #[test]
    fn empty_world_produces_empty_grid() {
        let mut app = build_app();
        run_update(&mut app);
        let grid = app.world().resource::<SpatialGrid>();
        assert!(grid.get_nearby(Vec2::ZERO, 1000.0).is_empty());
    }
}
