//! Medusa enemy — ranged AI systems.
//!
//! Three systems handle the Medusa's behavior:
//!
//! - [`tick_medusa_attack`] — advances `EnemyAI::attack_timer`; fires a
//!   [`MedusaProjectile`] toward the player when the interval elapses.
//! - [`move_medusa_projectiles`] — translates every active projectile along
//!   its velocity vector and despawns it when its lifetime expires.
//! - [`medusa_projectile_player_collision`] — checks each projectile against
//!   the player's collider and emits a [`PlayerDamagedEvent`] on contact.

use bevy::prelude::*;

use crate::{
    components::{
        CircleCollider, Enemy, EnemyAI, GameSessionEntity, InvincibilityTimer, MedusaProjectile,
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

/// Seconds between Medusa shots.
const DEFAULT_MEDUSA_ATTACK_INTERVAL: f32 = 2.0;
/// Projectile speed in pixels/second.
const DEFAULT_MEDUSA_PROJECTILE_SPEED: f32 = 180.0;
/// Projectile lifetime in seconds before despawn.
const DEFAULT_MEDUSA_PROJECTILE_LIFETIME: f32 = 5.0;
/// Projectile collider radius in pixels.
const DEFAULT_MEDUSA_PROJECTILE_RADIUS: f32 = 5.0;
/// Invincibility duration granted after a projectile hit (seconds).
const DEFAULT_PROJECTILE_INVINCIBILITY: f32 = 0.5;

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Ticks `EnemyAI::attack_timer` for all Medusa enemies and fires a
/// [`MedusaProjectile`] toward the player whenever the attack interval elapses.
///
/// The projectile's initial velocity is derived from the player's position at
/// the moment of fire (hitscan-style prediction is not used intentionally).
pub fn tick_medusa_attack(
    mut commands: Commands,
    time: Res<Time>,
    player_q: Query<&Transform, With<Player>>,
    mut medusa_q: Query<(&Transform, &Enemy, &mut EnemyAI)>,
    enemy_cfg: EnemyParams,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();

    let attack_interval = enemy_cfg
        .get()
        .map(|c| c.medusa_behavior.attack_interval)
        .unwrap_or(DEFAULT_MEDUSA_ATTACK_INTERVAL)
        .max(0.1);
    let proj_speed = enemy_cfg
        .get()
        .map(|c| c.medusa_behavior.projectile_speed)
        .unwrap_or(DEFAULT_MEDUSA_PROJECTILE_SPEED)
        .max(1.0);
    let proj_lifetime = enemy_cfg
        .get()
        .map(|c| c.medusa_behavior.projectile_lifetime)
        .unwrap_or(DEFAULT_MEDUSA_PROJECTILE_LIFETIME)
        .max(0.1);

    let dt = time.delta_secs();

    for (medusa_tf, enemy, mut ai) in medusa_q.iter_mut() {
        if enemy.enemy_type != EnemyType::Medusa || ai.ai_type != AIType::KeepDistance {
            continue;
        }

        ai.attack_timer += dt;
        if ai.attack_timer < attack_interval {
            continue;
        }
        ai.attack_timer = 0.0;

        let origin = medusa_tf.translation.truncate();
        let direction = (player_pos - origin).normalize_or_zero();
        if direction == Vec2::ZERO {
            continue; // player is exactly on Medusa — skip
        }

        commands.spawn((
            MedusaProjectile {
                damage: enemy.damage,
                velocity: direction * proj_speed,
                lifetime: proj_lifetime,
            },
            CircleCollider {
                radius: enemy_cfg
                    .get()
                    .map(|c| c.medusa_behavior.projectile_radius)
                    .unwrap_or(DEFAULT_MEDUSA_PROJECTILE_RADIUS),
            },
            Sprite {
                color: Color::srgb(0.9, 0.75, 0.0),
                custom_size: Some(Vec2::splat(
                    enemy_cfg
                        .get()
                        .map(|c| c.medusa_behavior.projectile_radius * 2.0)
                        .unwrap_or(DEFAULT_MEDUSA_PROJECTILE_RADIUS * 2.0),
                )),
                ..default()
            },
            Transform::from_translation(origin.extend(4.0)),
            GameSessionEntity,
        ));
    }
}

/// Advances every [`MedusaProjectile`] along its velocity vector and despawns
/// those whose lifetime has expired.
pub fn move_medusa_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut proj_q: Query<(Entity, &mut Transform, &mut MedusaProjectile)>,
) {
    let dt = time.delta_secs();
    for (entity, mut tf, mut proj) in proj_q.iter_mut() {
        tf.translation += (proj.velocity * dt).extend(0.0);
        proj.lifetime -= dt;
        if proj.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Checks every active [`MedusaProjectile`] against the player's circle
/// collider.  On contact the projectile is despawned and a
/// [`PlayerDamagedEvent`] is emitted.  Respects the player's
/// [`InvincibilityTimer`].
#[allow(clippy::type_complexity)]
pub fn medusa_projectile_player_collision(
    mut commands: Commands,
    player_q: Query<
        (Entity, &Transform, &CircleCollider, &PlayerStats),
        (With<Player>, Without<InvincibilityTimer>),
    >,
    proj_q: Query<(Entity, &Transform, &CircleCollider, &MedusaProjectile)>,
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
        .unwrap_or(DEFAULT_PROJECTILE_INVINCIBILITY);

    for (proj_entity, proj_tf, proj_collider, proj) in proj_q.iter() {
        let proj_pos = proj_tf.translation.truncate();
        if !check_circle_collision(
            player_pos,
            player_collider.radius,
            proj_pos,
            proj_collider.radius,
        ) {
            continue;
        }

        // Hit: despawn the projectile, emit damage, and grant invincibility.
        commands.entity(proj_entity).despawn();
        damage_events.write(PlayerDamagedEvent {
            player: player_entity,
            damage: proj.damage,
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
        components::{Enemy, EnemyAI, MedusaProjectile, Player, PlayerStats},
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

    fn spawn_medusa(app: &mut App, pos: Vec2) -> Entity {
        app.world_mut()
            .spawn((
                Enemy::from_type(EnemyType::Medusa, 1.0),
                EnemyAI {
                    ai_type: AIType::KeepDistance,
                    attack_timer: 0.0,
                    attack_range: 200.0,
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

    /// Medusa does not fire before the attack interval elapses.
    #[test]
    fn medusa_does_not_fire_before_interval() {
        let mut app = build_app();
        spawn_player(&mut app, Vec2::new(200.0, 0.0));
        spawn_medusa(&mut app, Vec2::ZERO);

        advance(&mut app, 0.5); // well below 2.0 s default
        app.world_mut()
            .run_system_once(tick_medusa_attack)
            .expect("tick_medusa_attack should run");

        let mut q = app.world_mut().query::<&MedusaProjectile>();
        assert_eq!(
            q.iter(app.world()).count(),
            0,
            "no projectile should fire before the interval"
        );
    }

    /// After the attack interval, one projectile is spawned.
    #[test]
    fn medusa_fires_after_interval() {
        let mut app = build_app();
        spawn_player(&mut app, Vec2::new(200.0, 0.0));
        let medusa = spawn_medusa(&mut app, Vec2::ZERO);

        // Pre-advance the attack timer to just past the interval.
        app.world_mut()
            .entity_mut(medusa)
            .get_mut::<EnemyAI>()
            .unwrap()
            .attack_timer = DEFAULT_MEDUSA_ATTACK_INTERVAL - 0.01;
        advance(&mut app, 0.05); // crosses threshold
        app.world_mut()
            .run_system_once(tick_medusa_attack)
            .expect("tick_medusa_attack should run");

        let mut q = app.world_mut().query::<&MedusaProjectile>();
        assert_eq!(
            q.iter(app.world()).count(),
            1,
            "exactly one projectile should be fired"
        );
    }

    /// Projectile moves along its velocity each frame.
    #[test]
    fn projectile_moves_each_frame() {
        let mut app = build_app();
        let proj = app
            .world_mut()
            .spawn((
                MedusaProjectile {
                    damage: 10.0,
                    velocity: Vec2::new(180.0, 0.0),
                    lifetime: 5.0,
                },
                CircleCollider { radius: 5.0 },
                Transform::from_translation(Vec3::ZERO),
            ))
            .id();

        let dt = 1.0_f32 / 60.0;
        advance(&mut app, dt);
        app.world_mut()
            .run_system_once(move_medusa_projectiles)
            .expect("move_medusa_projectiles should run");

        let x = app.world().get::<Transform>(proj).unwrap().translation.x;
        assert!((x - 180.0 * dt).abs() < 1e-3, "projectile should advance");
    }

    /// Projectile is despawned when lifetime reaches zero.
    #[test]
    fn projectile_despawns_on_lifetime_expiry() {
        let mut app = build_app();
        let proj = app
            .world_mut()
            .spawn((
                MedusaProjectile {
                    damage: 10.0,
                    velocity: Vec2::ZERO,
                    lifetime: 0.05, // very short
                },
                CircleCollider { radius: 5.0 },
                Transform::default(),
            ))
            .id();

        advance(&mut app, 0.1); // past lifetime
        app.world_mut()
            .run_system_once(move_medusa_projectiles)
            .expect("move_medusa_projectiles should run");
        app.world_mut().flush();

        assert!(
            app.world().get_entity(proj).is_err(),
            "projectile should be despawned after lifetime expires"
        );
    }

    /// A projectile overlapping the player deals damage and is despawned.
    #[test]
    fn projectile_hitting_player_deals_damage() {
        let mut app = build_app();
        let player = spawn_player(&mut app, Vec2::ZERO);

        // Spawn a projectile right on the player.
        let proj = app
            .world_mut()
            .spawn((
                MedusaProjectile {
                    damage: 12.0,
                    velocity: Vec2::ZERO,
                    lifetime: 5.0,
                },
                CircleCollider { radius: 5.0 },
                Transform::from_translation(Vec3::ZERO),
            ))
            .id();

        app.world_mut()
            .run_system_once(medusa_projectile_player_collision)
            .expect("collision should run");
        app.world_mut().flush();

        // Projectile is gone.
        assert!(
            app.world().get_entity(proj).is_err(),
            "projectile must be despawned on hit"
        );
        // Damage event was emitted.
        let messages = app.world().resource::<Messages<PlayerDamagedEvent>>();
        let mut cursor = messages.get_cursor();
        let events: Vec<_> = cursor.read(messages).collect();
        assert_eq!(events.len(), 1, "expected one damage event");
        assert_eq!(events[0].player, player);
        assert!((events[0].damage - 12.0).abs() < 1e-6);
    }

    /// An out-of-range projectile does not deal damage.
    #[test]
    fn projectile_far_away_no_damage() {
        let mut app = build_app();
        spawn_player(&mut app, Vec2::ZERO);
        app.world_mut().spawn((
            MedusaProjectile {
                damage: 12.0,
                velocity: Vec2::ZERO,
                lifetime: 5.0,
            },
            CircleCollider { radius: 5.0 },
            Transform::from_translation(Vec3::new(500.0, 0.0, 0.0)),
        ));

        app.world_mut()
            .run_system_once(medusa_projectile_player_collision)
            .expect("collision should run");

        let messages = app.world().resource::<Messages<PlayerDamagedEvent>>();
        let mut cursor = messages.get_cursor();
        let events: Vec<_> = cursor.read(messages).collect();
        assert!(events.is_empty(), "no damage when projectile is far away");
    }
}
