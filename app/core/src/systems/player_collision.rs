//! Enemy-to-player contact detection and player damage application.
//!
//! Three systems handle the full player-hit flow:
//!
//! - [`enemy_player_collision`] — each frame, checks whether any enemy
//!   overlaps the player.  If the player is not currently invincible, emits a
//!   [`PlayerDamagedEvent`] and inserts an [`InvincibilityTimer`].
//! - [`apply_damage_to_player`] — reads [`PlayerDamagedEvent`] and reduces
//!   [`PlayerStats::current_hp`], clamped to zero.
//! - [`tick_invincibility`] — decrements [`InvincibilityTimer`] each frame and
//!   removes the component when it expires.

use bevy::prelude::*;

use crate::{
    components::{CircleCollider, Enemy, InvincibilityTimer, Player, PlayerStats},
    events::PlayerDamagedEvent,
    resources::SpatialGrid,
    systems::collision::check_circle_collision,
};

// ---------------------------------------------------------------------------
// Fallback constants
// ---------------------------------------------------------------------------

/// Invincibility window granted after taking contact damage (seconds).
pub(crate) const DEFAULT_INVINCIBILITY_DURATION: f32 = 0.5;

/// Conservative upper bound on any enemy's collider radius (pixels).
///
/// Used to widen the spatial-grid query so large enemies whose centres lie
/// just outside the player's radius are still returned as candidates.
const MAX_ENEMY_COLLIDER_RADIUS: f32 = 32.0;

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Detects enemy–player overlaps and emits [`PlayerDamagedEvent`].
///
/// If the player currently carries an [`InvincibilityTimer`] the check is
/// skipped entirely (the query filter returns no entity).  On the first
/// overlapping enemy found, a [`PlayerDamagedEvent`] is emitted and an
/// [`InvincibilityTimer`] is inserted so the player cannot be hit again until
/// the timer expires.
/// Query filter for a non-invincible player entity.
type VulnerablePlayer = (With<Player>, Without<InvincibilityTimer>);

pub fn enemy_player_collision(
    mut commands: Commands,
    player_q: Query<(Entity, &Transform, &CircleCollider), VulnerablePlayer>,
    enemy_q: Query<(&Transform, &CircleCollider, &Enemy)>,
    spatial_grid: Res<SpatialGrid>,
    mut damage_events: MessageWriter<PlayerDamagedEvent>,
) {
    let Ok((player_entity, player_tf, player_collider)) = player_q.single() else {
        return; // player absent or currently invincible
    };

    let player_pos = player_tf.translation.truncate();
    let query_radius = player_collider.radius + MAX_ENEMY_COLLIDER_RADIUS;

    let candidates = spatial_grid.get_nearby(player_pos, query_radius);

    for candidate in candidates {
        let Ok((enemy_tf, enemy_collider, enemy)) = enemy_q.get(candidate) else {
            continue;
        };

        let enemy_pos = enemy_tf.translation.truncate();
        if !check_circle_collision(
            player_pos,
            player_collider.radius,
            enemy_pos,
            enemy_collider.radius,
        ) {
            continue;
        }

        // Hit confirmed: emit damage event and start invincibility window.
        damage_events.write(PlayerDamagedEvent {
            player: player_entity,
            damage: enemy.damage,
        });
        commands.entity(player_entity).insert(InvincibilityTimer {
            remaining: DEFAULT_INVINCIBILITY_DURATION,
        });
        return; // only one hit per frame
    }
}

/// Reads every [`PlayerDamagedEvent`] and reduces the player's current HP.
///
/// HP is clamped to zero; it cannot go negative.  The event targets a
/// specific player entity so future multi-player support is straightforward.
pub fn apply_damage_to_player(
    mut events: MessageReader<PlayerDamagedEvent>,
    mut player_q: Query<&mut PlayerStats, With<Player>>,
) {
    for event in events.read() {
        let Ok(mut stats) = player_q.get_mut(event.player) else {
            continue;
        };
        stats.current_hp = (stats.current_hp - event.damage).max(0.0);
    }
}

/// Counts down the [`InvincibilityTimer`] and removes it when it expires.
///
/// Must run every frame so the invincibility window doesn't outlast its 0.5 s
/// duration.
pub fn tick_invincibility(
    mut commands: Commands,
    mut timer_q: Query<(Entity, &mut InvincibilityTimer)>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    for (entity, mut timer) in timer_q.iter_mut() {
        timer.remaining -= delta;
        if timer.remaining <= 0.0 {
            commands.entity(entity).remove::<InvincibilityTimer>();
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
        components::{CircleCollider, Enemy, InvincibilityTimer, Player, PlayerStats},
        events::PlayerDamagedEvent,
        resources::SpatialGrid,
        systems::spatial::update_spatial_grid,
        types::EnemyType,
    };

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<PlayerDamagedEvent>();
        app.insert_resource(SpatialGrid::default());
        app
    }

    fn spawn_player(app: &mut App, pos: Vec2, radius: f32) -> Entity {
        app.world_mut()
            .spawn((
                Player,
                PlayerStats::default(),
                Transform::from_xyz(pos.x, pos.y, 10.0),
                CircleCollider { radius },
            ))
            .id()
    }

    fn spawn_enemy(app: &mut App, pos: Vec2, radius: f32) -> Entity {
        app.world_mut()
            .spawn((
                Enemy::from_type(EnemyType::Bat, 1.0),
                Transform::from_xyz(pos.x, pos.y, 1.0),
                CircleCollider { radius },
            ))
            .id()
    }

    fn update_grid(app: &mut App) {
        app.world_mut()
            .run_system_once(update_spatial_grid)
            .expect("update_spatial_grid should run");
    }

    fn run_collision(app: &mut App) {
        app.world_mut()
            .run_system_once(enemy_player_collision)
            .expect("enemy_player_collision should run");
    }

    fn damage_events(app: &App) -> Vec<PlayerDamagedEvent> {
        let messages = app.world().resource::<Messages<PlayerDamagedEvent>>();
        let mut cursor = messages.get_cursor();
        cursor.read(messages).cloned().collect()
    }

    // -----------------------------------------------------------------------
    // enemy_player_collision tests
    // -----------------------------------------------------------------------

    /// An overlapping enemy emits a PlayerDamagedEvent.
    #[test]
    fn contact_emits_player_damaged_event() {
        let mut app = build_app();
        let player = spawn_player(&mut app, Vec2::ZERO, 12.0);
        spawn_enemy(&mut app, Vec2::new(5.0, 0.0), 10.0); // overlaps player

        update_grid(&mut app);
        run_collision(&mut app);

        let events = damage_events(&app);
        assert_eq!(events.len(), 1, "expected one PlayerDamagedEvent");
        assert_eq!(events[0].player, player);
        assert!(events[0].damage > 0.0, "damage should be positive");
    }

    /// After contact, the player receives an InvincibilityTimer.
    #[test]
    fn contact_inserts_invincibility_timer() {
        let mut app = build_app();
        let player = spawn_player(&mut app, Vec2::ZERO, 12.0);
        spawn_enemy(&mut app, Vec2::new(5.0, 0.0), 10.0);

        update_grid(&mut app);
        run_collision(&mut app);
        app.world_mut().flush();

        let timer = app.world().get::<InvincibilityTimer>(player);
        assert!(
            timer.is_some(),
            "InvincibilityTimer should be inserted after hit"
        );
        let t = timer.unwrap();
        assert!(
            (t.remaining - DEFAULT_INVINCIBILITY_DURATION).abs() < 1e-6,
            "timer should start at {DEFAULT_INVINCIBILITY_DURATION} s"
        );
    }

    /// A player with InvincibilityTimer already present takes no damage.
    #[test]
    fn invincible_player_takes_no_damage() {
        let mut app = build_app();
        let player = spawn_player(&mut app, Vec2::ZERO, 12.0);
        // Pre-insert timer to simulate being invincible.
        app.world_mut()
            .entity_mut(player)
            .insert(InvincibilityTimer { remaining: 0.3 });
        spawn_enemy(&mut app, Vec2::new(5.0, 0.0), 10.0);

        update_grid(&mut app);
        run_collision(&mut app);

        assert!(
            damage_events(&app).is_empty(),
            "invincible player must not take damage"
        );
    }

    /// Only one damage event is emitted even when multiple enemies overlap.
    #[test]
    fn multiple_overlapping_enemies_emit_one_event() {
        let mut app = build_app();
        spawn_player(&mut app, Vec2::ZERO, 12.0);
        spawn_enemy(&mut app, Vec2::new(5.0, 0.0), 10.0);
        spawn_enemy(&mut app, Vec2::new(5.0, 0.0), 10.0);

        update_grid(&mut app);
        run_collision(&mut app);

        assert_eq!(
            damage_events(&app).len(),
            1,
            "only one event per frame even with multiple overlapping enemies"
        );
    }

    /// An enemy outside the player's radius does not trigger damage.
    #[test]
    fn non_overlapping_enemy_no_event() {
        let mut app = build_app();
        spawn_player(&mut app, Vec2::ZERO, 12.0);
        spawn_enemy(&mut app, Vec2::new(500.0, 0.0), 10.0); // far away

        update_grid(&mut app);
        run_collision(&mut app);

        assert!(
            damage_events(&app).is_empty(),
            "enemy out of range must not deal damage"
        );
    }

    // -----------------------------------------------------------------------
    // apply_damage_to_player tests
    // -----------------------------------------------------------------------

    /// apply_damage_to_player reduces current_hp by the event's damage amount.
    #[test]
    fn apply_damage_reduces_player_hp() {
        let mut app = build_app();
        let player = spawn_player(&mut app, Vec2::ZERO, 12.0);
        let initial_hp = app.world().get::<PlayerStats>(player).unwrap().current_hp;

        app.world_mut().write_message(PlayerDamagedEvent {
            player,
            damage: 10.0,
        });
        app.world_mut()
            .run_system_once(apply_damage_to_player)
            .expect("apply_damage_to_player should run");

        let hp = app.world().get::<PlayerStats>(player).unwrap().current_hp;
        assert_eq!(hp, initial_hp - 10.0);
    }

    /// Lethal damage clamps current_hp to zero rather than going negative.
    #[test]
    fn lethal_damage_clamps_to_zero() {
        let mut app = build_app();
        let player = spawn_player(&mut app, Vec2::ZERO, 12.0);

        app.world_mut().write_message(PlayerDamagedEvent {
            player,
            damage: 9999.0,
        });
        app.world_mut()
            .run_system_once(apply_damage_to_player)
            .expect("apply_damage_to_player should run");

        let hp = app.world().get::<PlayerStats>(player).unwrap().current_hp;
        assert_eq!(hp, 0.0, "HP must not go below zero");
    }

    // -----------------------------------------------------------------------
    // tick_invincibility tests
    // -----------------------------------------------------------------------

    /// Timer remaining above zero is not removed.
    #[test]
    fn timer_not_removed_while_positive() {
        let mut app = build_app();
        let player = spawn_player(&mut app, Vec2::ZERO, 12.0);
        app.world_mut()
            .entity_mut(player)
            .insert(InvincibilityTimer { remaining: 0.4 });

        // Advance by 0.1 s — timer should still be positive.
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(0.1));
        app.world_mut()
            .run_system_once(tick_invincibility)
            .expect("tick_invincibility should run");
        app.world_mut().flush();

        assert!(
            app.world().get::<InvincibilityTimer>(player).is_some(),
            "timer should remain while time has not elapsed"
        );
    }

    /// Timer is removed once it reaches zero.
    #[test]
    fn timer_removed_when_expired() {
        let mut app = build_app();
        let player = spawn_player(&mut app, Vec2::ZERO, 12.0);
        app.world_mut()
            .entity_mut(player)
            .insert(InvincibilityTimer { remaining: 0.1 });

        // Advance past the remaining duration.
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(0.2));
        app.world_mut()
            .run_system_once(tick_invincibility)
            .expect("tick_invincibility should run");
        app.world_mut().flush();

        assert!(
            app.world().get::<InvincibilityTimer>(player).is_none(),
            "timer should be removed after expiry"
        );
    }
}
