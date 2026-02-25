//! Weapon cooldown management system.
//!
//! [`tick_weapon_cooldowns`] runs every frame in [`AppState::Playing`] and
//! decrements the `cooldown_timer` of every weapon in the player's inventory.
//! When a timer reaches zero the system emits a [`WeaponFiredEvent`]; weapon-
//! specific fire systems (tasks 4.4 / 4.5) consume that event to spawn the
//! correct projectiles or effects.
//!
//! ## Cooldown reduction
//!
//! The timer is decremented by raw `delta_secs` each frame, but the reset
//! value after firing uses the player's `cooldown_reduction` stat:
//!
//! ```text
//! reset = base_cooldown × (1 − cooldown_reduction).clamp(0.1, 1.0)
//! ```
//!
//! Using `+=` (additive reset) instead of `=` keeps the fire rate accurate
//! even when a single frame is longer than the cooldown (fast-forward robustness).

use bevy::prelude::*;

use crate::{
    components::{Player, PlayerStats, WeaponInventory},
    events::WeaponFiredEvent,
};

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Ticks weapon cooldowns and fires [`WeaponFiredEvent`] when each expires.
///
/// - Requires the player entity to carry [`WeaponInventory`] and [`PlayerStats`].
/// - If the player is absent the system is a no-op.
/// - The cooldown timer is reset additively so that frames slower than the
///   cooldown do not silently swallow extra fire ticks.
pub fn tick_weapon_cooldowns(
    time: Res<Time>,
    mut player_q: Query<(Entity, &mut WeaponInventory, &PlayerStats), With<Player>>,
    mut fired_events: MessageWriter<WeaponFiredEvent>,
) {
    let delta = time.delta_secs();

    for (player_entity, mut inventory, stats) in player_q.iter_mut() {
        for weapon in inventory.weapons.iter_mut() {
            weapon.cooldown_timer -= delta;

            if weapon.cooldown_timer <= 0.0 {
                fired_events.write(WeaponFiredEvent {
                    player: player_entity,
                    weapon_type: weapon.weapon_type,
                    level: weapon.level,
                });

                // Additive reset: preserves any overshoot into the next cycle.
                weapon.cooldown_timer += weapon.effective_cooldown(stats.cooldown_reduction);
            }
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
    use crate::types::WeaponType;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<WeaponFiredEvent>();
        app
    }

    fn advance_and_run(app: &mut App, delta_secs: f32) {
        use std::time::Duration;
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(delta_secs));
        app.world_mut()
            .run_system_once(tick_weapon_cooldowns)
            .expect("tick_weapon_cooldowns should run");
    }

    fn spawn_player_with_weapon(app: &mut App, weapon_type: WeaponType) -> Entity {
        let mut weapon = crate::types::WeaponState::new(weapon_type);
        // Pre-expire the timer so it fires on the very first tick.
        weapon.cooldown_timer = 0.0;

        app.world_mut()
            .spawn((
                Player,
                PlayerStats::default(),
                WeaponInventory {
                    weapons: vec![weapon],
                },
            ))
            .id()
    }

    fn fired_count(app: &App) -> usize {
        let messages = app.world().resource::<Messages<WeaponFiredEvent>>();
        let mut cursor = messages.get_cursor();
        cursor.read(messages).count()
    }

    fn fired_events(app: &App) -> Vec<WeaponFiredEvent> {
        let messages = app.world().resource::<Messages<WeaponFiredEvent>>();
        let mut cursor = messages.get_cursor();
        cursor.read(messages).cloned().collect()
    }

    // -----------------------------------------------------------------------
    // Tests
    // -----------------------------------------------------------------------

    /// Cooldown timer decreases by `delta_secs` each frame.
    #[test]
    fn cooldown_timer_decreases_by_delta() {
        let mut app = build_app();

        let mut weapon = crate::types::WeaponState::new(WeaponType::MagicWand);
        let base = weapon.base_cooldown();
        // Set timer to base cooldown so it won't fire this tick.
        weapon.cooldown_timer = base;

        app.world_mut().spawn((
            Player,
            PlayerStats::default(),
            WeaponInventory {
                weapons: vec![weapon],
            },
        ));

        let delta = 1.0 / 60.0_f32;
        advance_and_run(&mut app, delta);

        let mut q = app
            .world_mut()
            .query_filtered::<&WeaponInventory, With<Player>>();
        let inv = q.single(app.world()).expect("player should exist");
        let remaining = inv.weapons[0].cooldown_timer;

        assert!(
            (remaining - (base - delta)).abs() < 1e-5,
            "timer should have decreased by delta ({delta}), expected ~{:.4}, got {:.4}",
            base - delta,
            remaining
        );
    }

    /// A weapon whose timer starts at or below 0 fires immediately.
    #[test]
    fn expired_timer_fires_event() {
        let mut app = build_app();
        spawn_player_with_weapon(&mut app, WeaponType::Whip);

        advance_and_run(&mut app, 1.0 / 60.0);

        let events = fired_events(&app);
        assert_eq!(events.len(), 1, "expected exactly one WeaponFiredEvent");
        assert_eq!(events[0].weapon_type, WeaponType::Whip);
        assert_eq!(events[0].level, 1);
    }

    /// After firing the timer resets to a positive value (≤ base_cooldown).
    #[test]
    fn timer_resets_after_firing() {
        let mut app = build_app();
        spawn_player_with_weapon(&mut app, WeaponType::MagicWand);

        advance_and_run(&mut app, 1.0 / 60.0);

        let mut q = app
            .world_mut()
            .query_filtered::<&WeaponInventory, With<Player>>();
        let inv = q.single(app.world()).expect("player should exist");
        let timer = inv.weapons[0].cooldown_timer;
        let base = crate::types::WeaponState::new(WeaponType::MagicWand).base_cooldown();

        assert!(
            timer > 0.0,
            "timer should be positive after reset, got {timer}"
        );
        assert!(
            timer <= base,
            "timer should not exceed base_cooldown ({base}), got {timer}"
        );
    }

    /// `cooldown_reduction = 0.5` halves the effective reset cooldown.
    #[test]
    fn cooldown_reduction_shortens_reset() {
        let mut app = build_app();

        let weapon = crate::types::WeaponState::new(WeaponType::Whip);
        let base = weapon.base_cooldown();

        let mut stats = PlayerStats::default();
        stats.cooldown_reduction = 0.5;

        app.world_mut().spawn((
            Player,
            stats,
            WeaponInventory {
                weapons: vec![weapon],
            },
        ));

        let delta = 1.0 / 60.0_f32;
        advance_and_run(&mut app, delta);

        let mut q = app
            .world_mut()
            .query_filtered::<&WeaponInventory, With<Player>>();
        let inv = q.single(app.world()).expect("player should exist");
        let timer = inv.weapons[0].cooldown_timer;

        // With 50% reduction reset = base * 0.5; minus the delta already consumed.
        let expected_reset = base * 0.5;
        assert!(
            (timer - (expected_reset - delta)).abs() < 1e-4,
            "expected timer ≈ {:.4}, got {:.4}",
            expected_reset - delta,
            timer
        );
    }

    /// A weapon with a positive timer must not emit a fire event.
    #[test]
    fn no_event_before_timer_expires() {
        let mut app = build_app();

        let mut weapon = crate::types::WeaponState::new(WeaponType::FireWand);
        weapon.cooldown_timer = 3.0; // far from expiry

        app.world_mut().spawn((
            Player,
            PlayerStats::default(),
            WeaponInventory {
                weapons: vec![weapon],
            },
        ));

        advance_and_run(&mut app, 1.0 / 60.0);

        assert_eq!(
            fired_count(&app),
            0,
            "no event expected while timer is positive"
        );
    }

    /// No player entity → system is a no-op.
    #[test]
    fn no_player_no_event() {
        let mut app = build_app();
        advance_and_run(&mut app, 1.0 / 60.0);
        assert_eq!(fired_count(&app), 0);
    }

    /// Two weapons tick independently: only the expired one fires.
    #[test]
    fn multiple_weapons_tick_independently() {
        let mut app = build_app();

        let mut whip = crate::types::WeaponState::new(WeaponType::Whip);
        whip.cooldown_timer = 0.0; // fires immediately

        let mut wand = crate::types::WeaponState::new(WeaponType::MagicWand);
        wand.cooldown_timer = 10.0; // does not fire

        app.world_mut().spawn((
            Player,
            PlayerStats::default(),
            WeaponInventory {
                weapons: vec![whip, wand],
            },
        ));

        advance_and_run(&mut app, 1.0 / 60.0);

        let events = fired_events(&app);
        assert_eq!(events.len(), 1, "only the whip should have fired");
        assert_eq!(events[0].weapon_type, WeaponType::Whip);
    }
}
