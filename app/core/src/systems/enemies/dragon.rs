//! Dragon enemy — melee-chase AI with ranged fireball attack.
//!
//! Three systems handle the Dragon's behavior:
//!
//! - [`tick_dragon_attack`] — advances `EnemyAI::attack_timer`; fires a
//!   [`DragonFireball`] toward the player when the interval elapses.
//! - [`move_dragon_fireballs`] — translates every active fireball along its
//!   velocity vector and despawns it when its lifetime expires.
//! - [`dragon_fireball_player_collision`] — checks each fireball against the
//!   player's collider and emits a [`PlayerDamagedEvent`] on contact.

use bevy::prelude::*;

use crate::{
    components::{
        CircleCollider, DragonFireball, Enemy, EnemyAI, GameSessionEntity, InvincibilityTimer,
        Player, PlayerStats,
    },
    config::{EnemyParams, PlayerParams},
    events::PlayerDamagedEvent,
    systems::collision::check_circle_collision,
    types::{AIType, EnemyType},
};

// ---------------------------------------------------------------------------
// Fallback constants (used when RON config is not yet loaded)
// ---------------------------------------------------------------------------

/// Seconds between Dragon fireball shots.
const DEFAULT_DRAGON_ATTACK_INTERVAL: f32 = 3.0;
/// Fireball speed in pixels/second.
const DEFAULT_DRAGON_FIREBALL_SPEED: f32 = 200.0;
/// Fireball lifetime in seconds before despawn.
const DEFAULT_DRAGON_FIREBALL_LIFETIME: f32 = 6.0;
/// Fireball collider radius in pixels.
pub(crate) const DEFAULT_DRAGON_FIREBALL_RADIUS: f32 = 7.0;
/// Invincibility duration granted after a fireball hit (seconds).
const DEFAULT_FIREBALL_INVINCIBILITY: f32 = 0.5;

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Ticks `EnemyAI::attack_timer` for all Dragon enemies and fires a
/// [`DragonFireball`] toward the player whenever the attack interval elapses.
///
/// Dragons use `ChasePlayer` AI for movement; this system adds the ranged
/// attack on top without changing their movement behavior.
pub fn tick_dragon_attack(
    mut commands: Commands,
    time: Res<Time>,
    player_q: Query<&Transform, With<Player>>,
    mut dragon_q: Query<(&Transform, &Enemy, &mut EnemyAI)>,
    enemy_cfg: EnemyParams,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();

    let attack_interval = enemy_cfg
        .get()
        .map(|c| c.dragon_behavior.attack_interval)
        .unwrap_or(DEFAULT_DRAGON_ATTACK_INTERVAL)
        .max(0.1);
    let fireball_speed = enemy_cfg
        .get()
        .map(|c| c.dragon_behavior.fireball_speed)
        .unwrap_or(DEFAULT_DRAGON_FIREBALL_SPEED)
        .max(1.0);
    let fireball_lifetime = enemy_cfg
        .get()
        .map(|c| c.dragon_behavior.fireball_lifetime)
        .unwrap_or(DEFAULT_DRAGON_FIREBALL_LIFETIME)
        .max(0.1);
    let fireball_radius = enemy_cfg
        .get()
        .map(|c| c.dragon_behavior.fireball_radius)
        .unwrap_or(DEFAULT_DRAGON_FIREBALL_RADIUS);

    let dt = time.delta_secs();

    for (dragon_tf, enemy, mut ai) in dragon_q.iter_mut() {
        if enemy.enemy_type != EnemyType::Dragon || ai.ai_type != AIType::ChasePlayer {
            continue;
        }

        ai.attack_timer += dt;
        if ai.attack_timer < attack_interval {
            continue;
        }
        ai.attack_timer = 0.0;

        let origin = dragon_tf.translation.truncate();
        let direction = (player_pos - origin).normalize_or_zero();
        if direction == Vec2::ZERO {
            continue; // player is exactly on Dragon — skip
        }

        commands.spawn((
            DragonFireball {
                damage: enemy.damage,
                velocity: direction * fireball_speed,
                lifetime: fireball_lifetime,
            },
            CircleCollider {
                radius: fireball_radius,
            },
            Sprite {
                color: Color::srgb(1.0, 0.4, 0.0),
                custom_size: Some(Vec2::splat(fireball_radius * 2.0)),
                ..default()
            },
            Transform::from_translation(origin.extend(4.0)),
            GameSessionEntity,
        ));
    }
}

/// Advances every [`DragonFireball`] along its velocity vector and despawns
/// those whose lifetime has expired.
pub fn move_dragon_fireballs(
    mut commands: Commands,
    time: Res<Time>,
    mut fireball_q: Query<(Entity, &mut Transform, &mut DragonFireball)>,
) {
    let dt = time.delta_secs();
    for (entity, mut tf, mut fireball) in fireball_q.iter_mut() {
        tf.translation += (fireball.velocity * dt).extend(0.0);
        fireball.lifetime -= dt;
        if fireball.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Checks every active [`DragonFireball`] against the player's circle
/// collider.  On contact the fireball is despawned and a
/// [`PlayerDamagedEvent`] is emitted.  Respects the player's
/// [`InvincibilityTimer`].
#[allow(clippy::type_complexity)]
pub fn dragon_fireball_player_collision(
    mut commands: Commands,
    player_q: Query<
        (Entity, &Transform, &CircleCollider, &PlayerStats),
        (With<Player>, Without<InvincibilityTimer>),
    >,
    fireball_q: Query<(Entity, &Transform, &CircleCollider, &DragonFireball)>,
    player_cfg: PlayerParams,
    mut damage_events: MessageWriter<PlayerDamagedEvent>,
) {
    let Ok((player_entity, player_tf, player_collider, _)) = player_q.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();

    let invincibility_duration = player_cfg
        .get()
        .map(|c| c.invincibility_time)
        .unwrap_or(DEFAULT_FIREBALL_INVINCIBILITY);

    for (fireball_entity, fireball_tf, fireball_collider, fireball) in fireball_q.iter() {
        let fireball_pos = fireball_tf.translation.truncate();
        if !check_circle_collision(
            player_pos,
            player_collider.radius,
            fireball_pos,
            fireball_collider.radius,
        ) {
            continue;
        }

        // Hit: despawn the fireball, emit damage, and grant invincibility.
        commands.entity(fireball_entity).despawn();
        damage_events.write(PlayerDamagedEvent {
            player: player_entity,
            damage: fireball.damage,
        });
        commands.entity(player_entity).insert(InvincibilityTimer {
            remaining: invincibility_duration,
        });
        return; // one hit per frame
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
        components::{DragonFireball, Enemy, EnemyAI, Player, PlayerStats},
        events::PlayerDamagedEvent,
        types::{AIType, EnemyType},
    };

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<PlayerDamagedEvent>();
        app
    }

    fn spawn_player(app: &mut App, pos: Vec2) -> Entity {
        app.world_mut()
            .spawn((
                Player,
                PlayerStats::default(),
                Transform::from_translation(pos.extend(10.0)),
                CircleCollider { radius: 12.0 },
            ))
            .id()
    }

    fn spawn_dragon(app: &mut App, pos: Vec2) -> Entity {
        app.world_mut()
            .spawn((
                Enemy::from_type(EnemyType::Dragon, 1.0),
                EnemyAI {
                    ai_type: AIType::ChasePlayer,
                    attack_timer: 0.0,
                    attack_range: 20.0,
                },
                Transform::from_translation(pos.extend(5.0)),
            ))
            .id()
    }

    fn advance(app: &mut App, secs: f32) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(secs));
    }

    // -----------------------------------------------------------------------

    /// Dragon does not fire before the attack interval elapses.
    #[test]
    fn dragon_does_not_fire_before_interval() {
        let mut app = build_app();
        spawn_player(&mut app, Vec2::new(200.0, 0.0));
        spawn_dragon(&mut app, Vec2::ZERO);

        advance(&mut app, 0.5); // well below 3.0 s default
        app.world_mut()
            .run_system_once(tick_dragon_attack)
            .expect("tick_dragon_attack should run");

        let mut q = app.world_mut().query::<&DragonFireball>();
        assert_eq!(
            q.iter(app.world()).count(),
            0,
            "no fireball should fire before the interval"
        );
    }

    /// After the attack interval, one fireball is spawned.
    #[test]
    fn dragon_fires_after_interval() {
        let mut app = build_app();
        spawn_player(&mut app, Vec2::new(200.0, 0.0));
        let dragon = spawn_dragon(&mut app, Vec2::ZERO);

        // Pre-advance the attack timer to just past the interval.
        app.world_mut()
            .entity_mut(dragon)
            .get_mut::<EnemyAI>()
            .unwrap()
            .attack_timer = DEFAULT_DRAGON_ATTACK_INTERVAL - 0.01;
        advance(&mut app, 0.05); // crosses threshold
        app.world_mut()
            .run_system_once(tick_dragon_attack)
            .expect("tick_dragon_attack should run");

        let mut q = app.world_mut().query::<&DragonFireball>();
        assert_eq!(
            q.iter(app.world()).count(),
            1,
            "exactly one fireball should be fired"
        );
    }

    /// Fireball moves along its velocity each frame.
    #[test]
    fn fireball_moves_each_frame() {
        let mut app = build_app();
        let fireball = app
            .world_mut()
            .spawn((
                DragonFireball {
                    damage: 25.0,
                    velocity: Vec2::new(200.0, 0.0),
                    lifetime: 6.0,
                },
                CircleCollider { radius: 7.0 },
                Transform::from_translation(Vec3::ZERO),
            ))
            .id();

        let dt = 1.0_f32 / 60.0;
        advance(&mut app, dt);
        app.world_mut()
            .run_system_once(move_dragon_fireballs)
            .expect("move_dragon_fireballs should run");

        let x = app
            .world()
            .get::<Transform>(fireball)
            .unwrap()
            .translation
            .x;
        assert!((x - 200.0 * dt).abs() < 1e-3, "fireball should advance");
    }

    /// Fireball is despawned when lifetime reaches zero.
    #[test]
    fn fireball_despawns_on_lifetime_expiry() {
        let mut app = build_app();
        let fireball = app
            .world_mut()
            .spawn((
                DragonFireball {
                    damage: 25.0,
                    velocity: Vec2::ZERO,
                    lifetime: 0.05, // very short
                },
                CircleCollider { radius: 7.0 },
                Transform::default(),
            ))
            .id();

        advance(&mut app, 0.1); // past lifetime
        app.world_mut()
            .run_system_once(move_dragon_fireballs)
            .expect("move_dragon_fireballs should run");
        app.world_mut().flush();

        assert!(
            app.world().get_entity(fireball).is_err(),
            "fireball should be despawned after lifetime expires"
        );
    }

    /// A fireball overlapping the player deals damage and is despawned.
    #[test]
    fn fireball_hitting_player_deals_damage() {
        let mut app = build_app();
        let player = spawn_player(&mut app, Vec2::ZERO);

        let fireball = app
            .world_mut()
            .spawn((
                DragonFireball {
                    damage: 25.0,
                    velocity: Vec2::ZERO,
                    lifetime: 6.0,
                },
                CircleCollider { radius: 7.0 },
                Transform::from_translation(Vec3::ZERO),
            ))
            .id();

        app.world_mut()
            .run_system_once(dragon_fireball_player_collision)
            .expect("collision should run");
        app.world_mut().flush();

        assert!(
            app.world().get_entity(fireball).is_err(),
            "fireball must be despawned on hit"
        );
        let messages = app.world().resource::<Messages<PlayerDamagedEvent>>();
        let mut cursor = messages.get_cursor();
        let events: Vec<_> = cursor.read(messages).collect();
        assert_eq!(events.len(), 1, "expected one damage event");
        assert_eq!(events[0].player, player);
        assert!((events[0].damage - 25.0).abs() < 1e-6);
    }

    /// An out-of-range fireball does not deal damage.
    #[test]
    fn fireball_far_away_no_damage() {
        let mut app = build_app();
        spawn_player(&mut app, Vec2::ZERO);
        app.world_mut().spawn((
            DragonFireball {
                damage: 25.0,
                velocity: Vec2::ZERO,
                lifetime: 6.0,
            },
            CircleCollider { radius: 7.0 },
            Transform::from_translation(Vec3::new(500.0, 0.0, 0.0)),
        ));

        app.world_mut()
            .run_system_once(dragon_fireball_player_collision)
            .expect("collision should run");

        let messages = app.world().resource::<Messages<PlayerDamagedEvent>>();
        let mut cursor = messages.get_cursor();
        let events: Vec<_> = cursor.read(messages).collect();
        assert!(events.is_empty(), "no damage when fireball is far away");
    }
}
