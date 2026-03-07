//! Boss scythe systems — Phase3 ranged projectile attack.
//!
//! Three systems handle the boss's Phase3 scythe behavior:
//!
//! - [`tick_boss_scythe_attack`] — advances `EnemyAI::attack_timer`; fires a
//!   [`BossScythe`] toward the player when the attack interval elapses.
//! - [`move_boss_scythes`] — translates every active scythe along its velocity
//!   vector and despawns it when its lifetime expires.
//! - [`boss_scythe_player_collision`] — checks each scythe against the player's
//!   collider and emits a [`PlayerDamagedEvent`] on contact.

use bevy::prelude::*;

use crate::{
    components::{
        BossScythe, CircleCollider, Enemy, EnemyAI, GameSessionEntity, InvincibilityTimer, Player,
        PlayerStats,
    },
    config::{GameParams, PlayerParams},
    events::PlayerDamagedEvent,
    systems::collision::check_circle_collision,
    types::{BossPhase, EnemyType},
};

// ---------------------------------------------------------------------------
// Fallback constants (used when RON config is not yet loaded)
// ---------------------------------------------------------------------------

/// Seconds between Boss Phase3 scythe shots.
const DEFAULT_BOSS_SCYTHE_INTERVAL: f32 = 3.0;
/// Scythe travel speed in pixels/second.
const DEFAULT_BOSS_SCYTHE_SPEED: f32 = 250.0;
/// Scythe lifetime in seconds before despawn.
const DEFAULT_BOSS_SCYTHE_LIFETIME: f32 = 8.0;
/// Damage dealt to the player on scythe contact.
const DEFAULT_BOSS_SCYTHE_DAMAGE: f32 = 80.0;
/// Scythe collider radius in pixels.
pub(crate) const DEFAULT_BOSS_SCYTHE_RADIUS: f32 = 15.0;
/// Invincibility duration granted to the player after a scythe hit (seconds).
const DEFAULT_SCYTHE_INVINCIBILITY: f32 = 0.5;

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Ticks `EnemyAI::attack_timer` for Boss Death in Phase3 and fires a
/// [`BossScythe`] toward the player whenever the attack interval elapses.
///
/// Only entities with [`BossPhase::Phase3`] and [`EnemyType::BossDeath`] are
/// affected.  The system is a no-op when no player entity exists.
pub fn tick_boss_scythe_attack(
    mut commands: Commands,
    time: Res<Time>,
    player_q: Query<&Transform, With<Player>>,
    mut boss_q: Query<(&Transform, &Enemy, &mut EnemyAI, &BossPhase)>,
    game_cfg: GameParams,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();

    let scythe_interval = game_cfg
        .get()
        .map(|c| c.boss_scythe_interval)
        .unwrap_or(DEFAULT_BOSS_SCYTHE_INTERVAL)
        .max(0.1);
    let scythe_speed = game_cfg
        .get()
        .map(|c| c.boss_scythe_speed)
        .unwrap_or(DEFAULT_BOSS_SCYTHE_SPEED)
        .max(1.0);
    let scythe_lifetime = game_cfg
        .get()
        .map(|c| c.boss_scythe_lifetime)
        .unwrap_or(DEFAULT_BOSS_SCYTHE_LIFETIME)
        .max(0.1);
    let scythe_damage = game_cfg
        .get()
        .map(|c| c.boss_scythe_damage)
        .unwrap_or(DEFAULT_BOSS_SCYTHE_DAMAGE);
    let scythe_radius = game_cfg
        .get()
        .map(|c| c.boss_scythe_radius)
        .unwrap_or(DEFAULT_BOSS_SCYTHE_RADIUS);

    let dt = time.delta_secs();

    for (boss_tf, enemy, mut ai, phase) in boss_q.iter_mut() {
        if *phase != BossPhase::Phase3 || enemy.enemy_type != EnemyType::BossDeath {
            continue;
        }

        ai.attack_timer += dt;
        if ai.attack_timer < scythe_interval {
            continue;
        }
        ai.attack_timer = 0.0;

        let origin = boss_tf.translation.truncate();
        let direction = (player_pos - origin).normalize_or_zero();
        if direction == Vec2::ZERO {
            continue; // player is exactly on boss — skip
        }

        commands.spawn((
            BossScythe {
                damage: scythe_damage,
                velocity: direction * scythe_speed,
                lifetime: scythe_lifetime,
            },
            CircleCollider {
                radius: scythe_radius,
            },
            Sprite {
                color: Color::srgb(0.8, 0.0, 0.8),
                custom_size: Some(Vec2::splat(scythe_radius * 2.0)),
                ..default()
            },
            Transform::from_translation(origin.extend(4.0)),
            GameSessionEntity,
        ));
    }
}

/// Advances every [`BossScythe`] along its velocity vector and despawns
/// those whose lifetime has expired.
pub fn move_boss_scythes(
    mut commands: Commands,
    time: Res<Time>,
    mut scythe_q: Query<(Entity, &mut Transform, &mut BossScythe)>,
) {
    let dt = time.delta_secs();
    for (entity, mut tf, mut scythe) in scythe_q.iter_mut() {
        tf.translation += (scythe.velocity * dt).extend(0.0);
        scythe.lifetime -= dt;
        if scythe.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Checks every active [`BossScythe`] against the player's circle collider.
/// On contact the scythe is despawned and a [`PlayerDamagedEvent`] is emitted.
/// Respects the player's [`InvincibilityTimer`].
#[allow(clippy::type_complexity)]
pub fn boss_scythe_player_collision(
    mut commands: Commands,
    player_q: Query<
        (Entity, &Transform, &CircleCollider, &PlayerStats),
        (With<Player>, Without<InvincibilityTimer>),
    >,
    scythe_q: Query<(Entity, &Transform, &CircleCollider, &BossScythe)>,
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
        .unwrap_or(DEFAULT_SCYTHE_INVINCIBILITY);

    for (scythe_entity, scythe_tf, scythe_collider, scythe) in scythe_q.iter() {
        let scythe_pos = scythe_tf.translation.truncate();
        if !check_circle_collision(
            player_pos,
            player_collider.radius,
            scythe_pos,
            scythe_collider.radius,
        ) {
            continue;
        }

        // Hit: despawn the scythe, emit damage, and grant invincibility.
        commands.entity(scythe_entity).despawn();
        damage_events.write(PlayerDamagedEvent {
            player: player_entity,
            damage: scythe.damage,
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
        components::{BossScythe, Enemy, EnemyAI, Player, PlayerStats},
        events::PlayerDamagedEvent,
        types::{AIType, BossPhase, EnemyType},
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

    fn spawn_boss_phase3(app: &mut App, pos: Vec2) -> Entity {
        app.world_mut()
            .spawn((
                Enemy::from_type(EnemyType::BossDeath, 1.0),
                BossPhase::Phase3,
                EnemyAI {
                    ai_type: AIType::BossMultiPhase,
                    attack_timer: 0.0,
                    attack_range: 300.0,
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
    // tick_boss_scythe_attack
    // -----------------------------------------------------------------------

    /// Boss does not fire before the attack interval elapses.
    #[test]
    fn boss_does_not_fire_scythe_before_interval() {
        let mut app = build_app();
        spawn_player(&mut app, Vec2::new(200.0, 0.0));
        spawn_boss_phase3(&mut app, Vec2::ZERO);

        advance(&mut app, 0.5); // well below 3.0 s default
        app.world_mut()
            .run_system_once(tick_boss_scythe_attack)
            .expect("tick_boss_scythe_attack should run");

        let mut q = app.world_mut().query::<&BossScythe>();
        assert_eq!(
            q.iter(app.world()).count(),
            0,
            "no scythe should fire before the interval"
        );
    }

    /// After the attack interval, one scythe is spawned.
    #[test]
    fn boss_fires_scythe_after_interval() {
        let mut app = build_app();
        spawn_player(&mut app, Vec2::new(200.0, 0.0));
        let boss = spawn_boss_phase3(&mut app, Vec2::ZERO);

        // Pre-advance the attack timer to just past the interval.
        app.world_mut()
            .entity_mut(boss)
            .get_mut::<EnemyAI>()
            .unwrap()
            .attack_timer = DEFAULT_BOSS_SCYTHE_INTERVAL - 0.01;
        advance(&mut app, 0.05); // crosses threshold
        app.world_mut()
            .run_system_once(tick_boss_scythe_attack)
            .expect("tick_boss_scythe_attack should run");

        let mut q = app.world_mut().query::<&BossScythe>();
        assert_eq!(
            q.iter(app.world()).count(),
            1,
            "exactly one scythe should be fired"
        );
    }

    /// Boss in Phase1 does not fire scythes.
    #[test]
    fn phase1_boss_does_not_fire_scythe() {
        let mut app = build_app();
        spawn_player(&mut app, Vec2::new(200.0, 0.0));
        let boss = app
            .world_mut()
            .spawn((
                Enemy::from_type(EnemyType::BossDeath, 1.0),
                BossPhase::Phase1,
                EnemyAI {
                    ai_type: AIType::BossMultiPhase,
                    attack_timer: DEFAULT_BOSS_SCYTHE_INTERVAL + 1.0, // past interval
                    attack_range: 300.0,
                },
                Transform::from_translation(Vec3::ZERO),
            ))
            .id();

        advance(&mut app, 0.05);
        app.world_mut()
            .run_system_once(tick_boss_scythe_attack)
            .expect("tick_boss_scythe_attack should run");

        // Phase1 boss should not fire scythes even if timer is past interval.
        // (The system skips non-Phase3 entities.)
        let _ = boss;
        let mut q = app.world_mut().query::<&BossScythe>();
        assert_eq!(
            q.iter(app.world()).count(),
            0,
            "Phase1 boss must not fire scythes"
        );
    }

    // -----------------------------------------------------------------------
    // move_boss_scythes
    // -----------------------------------------------------------------------

    /// Scythe moves along its velocity each frame.
    #[test]
    fn scythe_moves_each_frame() {
        let mut app = build_app();
        let scythe = app
            .world_mut()
            .spawn((
                BossScythe {
                    damage: 80.0,
                    velocity: Vec2::new(250.0, 0.0),
                    lifetime: 8.0,
                },
                CircleCollider { radius: 15.0 },
                Transform::from_translation(Vec3::ZERO),
            ))
            .id();

        let dt = 1.0_f32 / 60.0;
        advance(&mut app, dt);
        app.world_mut()
            .run_system_once(move_boss_scythes)
            .expect("move_boss_scythes should run");

        let x = app.world().get::<Transform>(scythe).unwrap().translation.x;
        assert!((x - 250.0 * dt).abs() < 1e-3, "scythe should advance");
    }

    /// Scythe is despawned when lifetime reaches zero.
    #[test]
    fn scythe_despawns_on_lifetime_expiry() {
        let mut app = build_app();
        let scythe = app
            .world_mut()
            .spawn((
                BossScythe {
                    damage: 80.0,
                    velocity: Vec2::ZERO,
                    lifetime: 0.05, // very short
                },
                CircleCollider { radius: 15.0 },
                Transform::default(),
            ))
            .id();

        advance(&mut app, 0.1); // past lifetime
        app.world_mut()
            .run_system_once(move_boss_scythes)
            .expect("move_boss_scythes should run");
        app.world_mut().flush();

        assert!(
            app.world().get_entity(scythe).is_err(),
            "scythe should be despawned after lifetime expires"
        );
    }

    // -----------------------------------------------------------------------
    // boss_scythe_player_collision
    // -----------------------------------------------------------------------

    /// A scythe overlapping the player deals damage and is despawned.
    #[test]
    fn scythe_hitting_player_deals_damage() {
        let mut app = build_app();
        let player = spawn_player(&mut app, Vec2::ZERO);

        let scythe = app
            .world_mut()
            .spawn((
                BossScythe {
                    damage: 80.0,
                    velocity: Vec2::ZERO,
                    lifetime: 8.0,
                },
                CircleCollider { radius: 15.0 },
                Transform::from_translation(Vec3::ZERO),
            ))
            .id();

        app.world_mut()
            .run_system_once(boss_scythe_player_collision)
            .expect("collision should run");
        app.world_mut().flush();

        assert!(
            app.world().get_entity(scythe).is_err(),
            "scythe must be despawned on hit"
        );
        let messages = app.world().resource::<Messages<PlayerDamagedEvent>>();
        let mut cursor = messages.get_cursor();
        let events: Vec<_> = cursor.read(messages).collect();
        assert_eq!(events.len(), 1, "expected one damage event");
        assert_eq!(events[0].player, player);
        assert!((events[0].damage - 80.0).abs() < 1e-6);
    }

    /// An out-of-range scythe does not deal damage.
    #[test]
    fn scythe_far_away_no_damage() {
        let mut app = build_app();
        spawn_player(&mut app, Vec2::ZERO);
        app.world_mut().spawn((
            BossScythe {
                damage: 80.0,
                velocity: Vec2::ZERO,
                lifetime: 8.0,
            },
            CircleCollider { radius: 15.0 },
            Transform::from_translation(Vec3::new(500.0, 0.0, 0.0)),
        ));

        app.world_mut()
            .run_system_once(boss_scythe_player_collision)
            .expect("collision should run");

        let messages = app.world().resource::<Messages<PlayerDamagedEvent>>();
        let mut cursor = messages.get_cursor();
        let events: Vec<_> = cursor.read(messages).collect();
        assert!(events.is_empty(), "no damage when scythe is far away");
    }
}
