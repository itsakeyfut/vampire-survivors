//! Enemy culling system — silently despawns enemies that wander too far from
//! the player.
//!
//! [`cull_distant_enemies`] runs every frame while in [`AppState::Playing`].
//! Any [`Enemy`] entity whose distance from the player exceeds
//! [`DEFAULT_ENEMY_CULL_DISTANCE`] is despawned immediately **without** dropping an
//! XP gem. This keeps memory bounded when the player moves large distances
//! and leaves enemy clusters behind.

use bevy::prelude::*;

use crate::{
    components::{Enemy, Player},
    config::EnemyParams,
};

// ---------------------------------------------------------------------------
// Fallback constants (used when RON config is not yet loaded)
// ---------------------------------------------------------------------------

/// Distance from the player (pixels) beyond which enemies are despawned.
const DEFAULT_ENEMY_CULL_DISTANCE: f32 = 2000.0;

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Despawns enemies beyond the cull distance from the player.
///
/// The cull distance is read from [`EnemyParams`] when config is loaded;
/// otherwise falls back to [`DEFAULT_ENEMY_CULL_DISTANCE`].
///
/// - No XP gem is dropped — the enemy simply disappears.
/// - If there is no player entity, the system is a no-op.
/// - Uses squared-distance comparison to avoid a `sqrt` per enemy per frame.
pub fn cull_distant_enemies(
    mut commands: Commands,
    player_q: Query<&Transform, With<Player>>,
    enemy_q: Query<(Entity, &Transform), With<Enemy>>,
    enemy_cfg: EnemyParams,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();
    let cull_distance = enemy_cfg
        .get()
        .map(|c| c.cull_distance)
        .unwrap_or(DEFAULT_ENEMY_CULL_DISTANCE);
    let cull_dist_sq = cull_distance * cull_distance;

    for (entity, tf) in enemy_q.iter() {
        let enemy_pos = tf.translation.truncate();
        if player_pos.distance_squared(enemy_pos) > cull_dist_sq {
            commands.entity(entity).despawn();
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce as _;

    use super::*;
    use crate::components::Enemy;
    use crate::types::EnemyType;

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app
    }

    fn spawn_player_at(app: &mut App, pos: Vec2) {
        app.world_mut()
            .spawn((Player, Transform::from_translation(pos.extend(0.0))));
    }

    fn spawn_enemy_at(app: &mut App, pos: Vec2) -> Entity {
        app.world_mut()
            .spawn((
                Enemy::from_type(EnemyType::Bat, 1.0),
                Transform::from_translation(pos.extend(5.0)),
            ))
            .id()
    }

    fn run_cull(app: &mut App) {
        app.world_mut()
            .run_system_once(cull_distant_enemies)
            .expect("cull_distant_enemies should run");
    }

    // -----------------------------------------------------------------------

    /// An enemy beyond the cull threshold must be despawned.
    #[test]
    fn enemy_beyond_threshold_is_despawned() {
        let mut app = build_app();
        spawn_player_at(&mut app, Vec2::ZERO);
        let enemy = spawn_enemy_at(&mut app, Vec2::new(DEFAULT_ENEMY_CULL_DISTANCE + 1.0, 0.0));

        run_cull(&mut app);

        assert!(
            app.world().get_entity(enemy).is_err(),
            "enemy beyond cull distance should be despawned"
        );
    }

    /// An enemy exactly at the threshold must NOT be despawned.
    #[test]
    fn enemy_at_threshold_is_kept() {
        let mut app = build_app();
        spawn_player_at(&mut app, Vec2::ZERO);
        let enemy = spawn_enemy_at(&mut app, Vec2::new(DEFAULT_ENEMY_CULL_DISTANCE, 0.0));

        run_cull(&mut app);

        assert!(
            app.world().get_entity(enemy).is_ok(),
            "enemy exactly at cull distance should survive"
        );
    }

    /// An enemy within range must not be despawned.
    #[test]
    fn enemy_within_range_is_kept() {
        let mut app = build_app();
        spawn_player_at(&mut app, Vec2::ZERO);
        let enemy = spawn_enemy_at(&mut app, Vec2::new(100.0, 0.0));

        run_cull(&mut app);

        assert!(
            app.world().get_entity(enemy).is_ok(),
            "enemy within cull distance should survive"
        );
    }

    /// Only the distant enemy is removed when there are multiple enemies.
    #[test]
    fn only_distant_enemy_is_culled() {
        let mut app = build_app();
        spawn_player_at(&mut app, Vec2::ZERO);
        let near = spawn_enemy_at(&mut app, Vec2::new(500.0, 0.0));
        let far = spawn_enemy_at(
            &mut app,
            Vec2::new(DEFAULT_ENEMY_CULL_DISTANCE + 100.0, 0.0),
        );

        run_cull(&mut app);

        assert!(
            app.world().get_entity(near).is_ok(),
            "near enemy should survive"
        );
        assert!(
            app.world().get_entity(far).is_err(),
            "far enemy should be despawned"
        );
    }

    /// Without a player, no enemies are removed.
    #[test]
    fn no_player_no_cull() {
        let mut app = build_app();
        let enemy = spawn_enemy_at(&mut app, Vec2::new(9999.0, 0.0));

        run_cull(&mut app);

        assert!(
            app.world().get_entity(enemy).is_ok(),
            "no player → no cull should occur"
        );
    }

    /// Multiple distant enemies are all removed in a single pass.
    #[test]
    fn multiple_distant_enemies_all_culled() {
        let mut app = build_app();
        spawn_player_at(&mut app, Vec2::ZERO);

        let far1 = spawn_enemy_at(&mut app, Vec2::new(DEFAULT_ENEMY_CULL_DISTANCE + 1.0, 0.0));
        let far2 = spawn_enemy_at(&mut app, Vec2::new(0.0, DEFAULT_ENEMY_CULL_DISTANCE + 1.0));
        let far3 = spawn_enemy_at(&mut app, Vec2::new(-DEFAULT_ENEMY_CULL_DISTANCE - 1.0, 0.0));

        run_cull(&mut app);

        assert!(
            app.world().get_entity(far1).is_err(),
            "far1 should be culled"
        );
        assert!(
            app.world().get_entity(far2).is_err(),
            "far2 should be culled"
        );
        assert!(
            app.world().get_entity(far3).is_err(),
            "far3 should be culled"
        );
    }
}
