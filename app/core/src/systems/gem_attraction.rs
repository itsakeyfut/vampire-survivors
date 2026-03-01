//! XP gem magnetic attraction and absorption systems.
//!
//! Two systems handle the full gem-collection flow:
//!
//! - [`attract_gems_to_player`] — each frame, scans all un-attracted
//!   [`ExperienceGem`] entities.  Any gem within the player's
//!   [`PlayerStats::pickup_radius`] receives an [`AttractedToPlayer`]
//!   component that drives movement toward the player.
//!
//! - [`move_attracted_gems`] — advances each attracted gem along its vector
//!   toward the player.  When the gem is close enough it is absorbed:
//!   its value is added to [`GameData::current_xp`] and the entity is
//!   despawned.

use bevy::prelude::*;

use crate::{
    components::{AttractedToPlayer, ExperienceGem, Player, PlayerStats},
    resources::GameData,
};

/// Query filter for gems that have not yet started moving toward the player.
type UnattractedGem = (With<ExperienceGem>, Without<AttractedToPlayer>);

/// Query filter for gems that are currently attracted to the player.
///
/// `Without<Player>` makes the `Transform` access disjoint from the player
/// query in [`move_attracted_gems`], satisfying Bevy's borrow checker.
type AttractedGem = (
    With<ExperienceGem>,
    With<AttractedToPlayer>,
    Without<Player>,
);

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Checks every un-attracted [`ExperienceGem`] and starts magnetic attraction
/// for those within the player's [`PlayerStats::pickup_radius`].
///
/// Inserts [`AttractedToPlayer`] on qualifying gems so that
/// [`move_attracted_gems`] can move them each frame.  The query filter
/// `Without<AttractedToPlayer>` prevents re-inserting the component on gems
/// that are already moving.
pub fn attract_gems_to_player(
    mut commands: Commands,
    player_q: Query<(&Transform, &PlayerStats), With<Player>>,
    gem_q: Query<(Entity, &Transform), UnattractedGem>,
) {
    let Ok((player_tf, player_stats)) = player_q.single() else {
        return;
    };

    let pickup_radius_sq = player_stats.pickup_radius * player_stats.pickup_radius;
    let player_pos = player_tf.translation.truncate();

    for (gem_entity, gem_tf) in gem_q.iter() {
        let gem_pos = gem_tf.translation.truncate();
        if gem_pos.distance_squared(player_pos) < pickup_radius_sq {
            commands.entity(gem_entity).insert(AttractedToPlayer {
                speed: player_stats.gem_attraction_speed,
            });
        }
    }
}

/// Moves every [`AttractedToPlayer`] gem toward the player and absorbs it
/// when it arrives.
///
/// On each frame the gem is translated along the normalised direction vector
/// toward the player by `speed × delta_secs` pixels.  When the remaining
/// distance is within `gem_absorption_radius` (from `player.ron`) the gem is
/// despawned and its [`ExperienceGem::value`] is added to [`GameData::current_xp`].
pub fn move_attracted_gems(
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
    player_q: Query<(&Transform, &PlayerStats), With<Player>>,
    mut gem_q: Query<(Entity, &mut Transform, &ExperienceGem, &AttractedToPlayer), AttractedGem>,
    time: Res<Time>,
) {
    let Ok((player_tf, player_stats)) = player_q.single() else {
        return;
    };

    let absorption_radius = player_stats.gem_absorption_radius;
    let player_pos = player_tf.translation.truncate();
    let delta = time.delta_secs();

    for (gem_entity, mut gem_tf, gem, attracted) in gem_q.iter_mut() {
        let gem_pos = gem_tf.translation.truncate();
        let to_player = player_pos - gem_pos;
        let distance = to_player.length();

        if distance <= absorption_radius {
            // Gem has reached the player — absorb it.
            game_data.current_xp += gem.value;
            commands.entity(gem_entity).despawn();
        } else {
            // Move toward the player, clamped so we never overshoot.
            let direction = to_player / distance; // normalised
            let step = attracted.speed * delta;
            let move_dist = step.min(distance - absorption_radius);
            gem_tf.translation += (direction * move_dist).extend(0.0);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::ecs::system::RunSystemOnce as _;

    use super::*;
    use crate::{
        components::{AttractedToPlayer, ExperienceGem, Player, PlayerStats},
        resources::GameData,
    };

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(GameData::default());
        app
    }

    fn spawn_player(app: &mut App, pos: Vec2) -> Entity {
        app.world_mut()
            .spawn((
                Player,
                PlayerStats::default(),
                Transform::from_xyz(pos.x, pos.y, 0.0),
            ))
            .id()
    }

    fn spawn_gem(app: &mut App, pos: Vec2, value: u32) -> Entity {
        app.world_mut()
            .spawn((
                ExperienceGem { value },
                Transform::from_xyz(pos.x, pos.y, 0.5),
            ))
            .id()
    }

    // --- attract_gems_to_player tests ---

    /// Gem within pickup_radius receives AttractedToPlayer.
    #[test]
    fn gem_inside_radius_becomes_attracted() {
        let mut app = build_app();
        spawn_player(&mut app, Vec2::ZERO);
        let gem = spawn_gem(&mut app, Vec2::new(50.0, 0.0), 3); // within 80 px default radius

        app.world_mut()
            .run_system_once(attract_gems_to_player)
            .unwrap();

        assert!(
            app.world().get::<AttractedToPlayer>(gem).is_some(),
            "gem within pickup_radius must be attracted"
        );
    }

    /// Gem outside pickup_radius stays un-attracted.
    #[test]
    fn gem_outside_radius_stays_still() {
        let mut app = build_app();
        spawn_player(&mut app, Vec2::ZERO);
        let gem = spawn_gem(&mut app, Vec2::new(500.0, 0.0), 3); // beyond 80 px radius

        app.world_mut()
            .run_system_once(attract_gems_to_player)
            .unwrap();

        assert!(
            app.world().get::<AttractedToPlayer>(gem).is_none(),
            "gem outside pickup_radius must not be attracted"
        );
    }

    /// Already-attracted gem is not touched by attract system.
    #[test]
    fn already_attracted_gem_not_re_processed() {
        let mut app = build_app();
        spawn_player(&mut app, Vec2::ZERO);
        // Spawn gem already having the component.
        let gem = app
            .world_mut()
            .spawn((
                ExperienceGem { value: 3 },
                Transform::from_xyz(10.0, 0.0, 0.5),
                AttractedToPlayer { speed: 999.0 }, // sentinel speed
            ))
            .id();

        app.world_mut()
            .run_system_once(attract_gems_to_player)
            .unwrap();

        // Speed must remain unchanged (system skips With<AttractedToPlayer> gems).
        let speed = app
            .world()
            .get::<AttractedToPlayer>(gem)
            .map(|a| a.speed)
            .unwrap();
        assert!(
            (speed - 999.0).abs() < 0.001,
            "attract system must not mutate an already-attracted gem"
        );
    }

    /// No player entity — system does not panic.
    #[test]
    fn attract_no_player_no_panic() {
        let mut app = build_app();
        spawn_gem(&mut app, Vec2::ZERO, 3);
        app.world_mut()
            .run_system_once(attract_gems_to_player)
            .unwrap();
    }

    // --- move_attracted_gems tests ---

    /// Gem moves toward player each frame.
    #[test]
    fn attracted_gem_moves_toward_player() {
        let mut app = build_app();
        spawn_player(&mut app, Vec2::ZERO);
        let gem = app
            .world_mut()
            .spawn((
                ExperienceGem { value: 3 },
                Transform::from_xyz(100.0, 0.0, 0.5),
                AttractedToPlayer {
                    speed: PlayerStats::default().gem_attraction_speed,
                },
            ))
            .id();

        // Advance time so delta > 0.
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(1.0 / 60.0));
        app.world_mut()
            .run_system_once(move_attracted_gems)
            .unwrap();

        let x = app
            .world()
            .get::<Transform>(gem)
            .map(|t| t.translation.x)
            .unwrap_or(100.0);
        assert!(
            x < 100.0,
            "gem x must decrease as it moves toward player at origin"
        );
    }

    /// Gem absorbed when close enough — despawned and XP credited.
    #[test]
    fn gem_absorbed_when_at_player_position() {
        let mut app = build_app();
        spawn_player(&mut app, Vec2::ZERO);
        // Place gem very close to player (within ABSORPTION_RADIUS).
        app.world_mut().spawn((
            ExperienceGem { value: 5 },
            Transform::from_xyz(2.0, 0.0, 0.5), // distance 2.0 < gem_absorption_radius (8.0)
            AttractedToPlayer {
                speed: PlayerStats::default().gem_attraction_speed,
            },
        ));

        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(1.0 / 60.0));
        app.world_mut()
            .run_system_once(move_attracted_gems)
            .unwrap();

        let game_data = app.world().resource::<GameData>();
        assert_eq!(
            game_data.current_xp, 5,
            "XP should be credited when gem is absorbed"
        );
    }

    /// Multiple gems absorbed in one frame accumulate XP.
    #[test]
    fn multiple_gems_absorbed_accumulate_xp() {
        let mut app = build_app();
        spawn_player(&mut app, Vec2::ZERO);
        for value in [3, 5, 8] {
            app.world_mut().spawn((
                ExperienceGem { value },
                Transform::from_xyz(1.0, 0.0, 0.5),
                AttractedToPlayer {
                    speed: PlayerStats::default().gem_attraction_speed,
                },
            ));
        }

        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(1.0 / 60.0));
        app.world_mut()
            .run_system_once(move_attracted_gems)
            .unwrap();

        let game_data = app.world().resource::<GameData>();
        assert_eq!(
            game_data.current_xp,
            3 + 5 + 8,
            "all absorbed gems should contribute XP"
        );
    }

    /// No player entity — system does not panic.
    #[test]
    fn move_no_player_no_panic() {
        let mut app = build_app();
        app.world_mut().spawn((
            ExperienceGem { value: 3 },
            Transform::from_xyz(1.0, 0.0, 0.5),
            AttractedToPlayer {
                speed: PlayerStats::default().gem_attraction_speed,
            },
        ));
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(1.0 / 60.0));
        app.world_mut()
            .run_system_once(move_attracted_gems)
            .unwrap();
    }
}
