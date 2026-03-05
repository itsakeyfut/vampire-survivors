//! Weapon evolution notification widget.
//!
//! Spawns a short-lived text overlay at the top-center of the screen when a
//! weapon evolves.  The text displays the evolved weapon's name, holds full
//! opacity for the first [`DEFAULT_FADE_START`] seconds, then fades linearly
//! to transparent before being despawned.
//!
//! ## Lifecycle
//!
//! ```text
//! WeaponEvolvedTrigger  →  on_weapon_evolved (observer)  →  spawn Node + Text
//!                                  ↓
//!                    update_evolution_notification (Update system)
//!                          ticks elapsed, fades alpha, despawns
//! ```
//!
//! The root `Node` entity carries [`DespawnOnExit`]`(`[`AppState::Playing`]`)`
//! so any notification still alive when the player dies or wins is cleaned up
//! automatically.

use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;
use vs_core::states::AppState;
use vs_core::systems::xp::treasure::WeaponEvolvedTrigger;
use vs_core::types::WeaponType;

use crate::config::EvolutionNotificationHudParams;

// ---------------------------------------------------------------------------
// Fallback constants
// ---------------------------------------------------------------------------

/// Display duration of the notification in seconds.
const DEFAULT_DISPLAY_DURATION: f32 = 3.0;
/// Time after which the alpha fade begins (seconds).
const DEFAULT_FADE_START: f32 = 1.5;
/// Font size for the notification text.
const DEFAULT_FONT_SIZE: f32 = 40.0;
/// Vertical position of the notification as a percentage of the screen height.
const DEFAULT_TOP_PERCENT: f32 = 38.0;
/// Text color: golden yellow.
const DEFAULT_TEXT_COLOR: Color = Color::srgb(1.0, 0.85, 0.2);

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/// Drives the fade-and-despawn animation of an evolution notification overlay.
///
/// Placed on the root [`Node`] entity.  [`update_evolution_notification`]
/// advances `elapsed` each frame, updates the text alpha, and despawns the
/// entity once `elapsed >= duration`.
#[derive(Component, Debug)]
pub struct EvolutionNotification {
    /// Elapsed time since the notification was spawned (seconds).
    pub elapsed: f32,
    /// Total lifetime before despawn (seconds).
    pub duration: f32,
    /// Time at which the alpha fade-out begins (seconds).
    pub fade_start: f32,
    /// Entity holding [`TextColor`]; its alpha is updated each frame.
    pub text_entity: Entity,
    /// Base text color (fully opaque); alpha is overridden during fade-out.
    pub text_color: Color,
}

// ---------------------------------------------------------------------------
// Display name helper
// ---------------------------------------------------------------------------

/// Returns a human-readable display name for the given [`WeaponType`].
pub fn weapon_display_name(weapon: WeaponType) -> &'static str {
    match weapon {
        WeaponType::Whip => "Whip",
        WeaponType::MagicWand => "Magic Wand",
        WeaponType::Knife => "Knife",
        WeaponType::Garlic => "Garlic",
        WeaponType::Bible => "Bible",
        WeaponType::ThunderRing => "Thunder Ring",
        WeaponType::Cross => "Cross",
        WeaponType::FireWand => "Fire Wand",
        WeaponType::BloodyTear => "Bloody Tear",
        WeaponType::HolyWand => "Holy Wand",
        WeaponType::ThousandEdge => "Thousand Edge",
        WeaponType::SoulEater => "Soul Eater",
        WeaponType::UnholyVespers => "Unholy Vespers",
        WeaponType::LightningRing => "Lightning Ring",
    }
}

// ---------------------------------------------------------------------------
// Observer
// ---------------------------------------------------------------------------

/// Spawns an evolution notification when [`WeaponEvolvedTrigger`] fires.
///
/// Registered as a global observer so it reacts to every trigger regardless
/// of which entity fires it.  The notification is positioned in the top-center
/// of the screen via an absolute-positioned flex [`Node`].
pub fn on_weapon_evolved(
    trigger: On<WeaponEvolvedTrigger>,
    mut commands: Commands,
    cfg: EvolutionNotificationHudParams,
) {
    let cfg = cfg.get();
    let duration = cfg.map_or(DEFAULT_DISPLAY_DURATION, |c| c.display_duration);
    let fade_start = cfg.map_or(DEFAULT_FADE_START, |c| c.fade_start);
    let font_size = cfg.map_or(DEFAULT_FONT_SIZE, |c| c.font_size);
    let top_percent = cfg.map_or(DEFAULT_TOP_PERCENT, |c| c.top_percent);
    let text_color = cfg.map_or(DEFAULT_TEXT_COLOR, |c| Color::from(&c.text_color));

    let name = weapon_display_name(trigger.event().evolved_type);
    let text = format!("{name}\nEvolved!");

    // Spawn the text as a separate entity so its TextColor can be mutated
    // without conflicting with the parent Node query.
    let text_entity = commands
        .spawn((
            Text::new(text),
            TextFont {
                font_size,
                ..default()
            },
            TextColor(text_color),
            TextLayout::new_with_justify(Justify::Center),
        ))
        .id();

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                top: Val::Percent(top_percent),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                ..default()
            },
            EvolutionNotification {
                elapsed: 0.0,
                duration,
                fade_start,
                text_entity,
                text_color,
            },
            DespawnOnExit(AppState::Playing),
        ))
        .add_child(text_entity);
}

// ---------------------------------------------------------------------------
// Update system
// ---------------------------------------------------------------------------

/// Advances all active [`EvolutionNotification`] animations each frame.
///
/// - Ticks `elapsed` by `Δt`.
/// - Keeps full opacity while `elapsed < fade_start`.
/// - Fades alpha linearly from 1.0 → 0.0 over the remaining lifetime.
/// - Despawns the root entity (and its text child) once `elapsed >= duration`.
pub fn update_evolution_notification(
    mut commands: Commands,
    time: Res<Time>,
    mut notif_q: Query<(Entity, &mut EvolutionNotification)>,
    mut text_q: Query<&mut TextColor>,
) {
    let dt = time.delta_secs();
    for (entity, mut notif) in notif_q.iter_mut() {
        notif.elapsed += dt;

        if notif.elapsed >= notif.duration {
            commands.entity(entity).despawn();
            continue;
        }

        let alpha = if notif.elapsed < notif.fade_start {
            1.0_f32
        } else {
            let fade_progress =
                (notif.elapsed - notif.fade_start) / (notif.duration - notif.fade_start);
            (1.0 - fade_progress).max(0.0)
        };

        if let Ok(mut text_color) = text_q.get_mut(notif.text_entity) {
            text_color.0 = notif.text_color.with_alpha(alpha);
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

    // -----------------------------------------------------------------------
    // weapon_display_name
    // -----------------------------------------------------------------------

    /// All evolved weapon types return non-empty display names.
    #[test]
    fn evolved_weapons_have_display_names() {
        let evolved = [
            WeaponType::BloodyTear,
            WeaponType::HolyWand,
            WeaponType::ThousandEdge,
            WeaponType::SoulEater,
            WeaponType::UnholyVespers,
            WeaponType::LightningRing,
        ];
        for wt in evolved {
            let name = weapon_display_name(wt);
            assert!(
                !name.is_empty(),
                "{wt:?} should have a non-empty display name"
            );
        }
    }

    /// All base weapon types also return non-empty display names.
    #[test]
    fn base_weapons_have_display_names() {
        let base = [
            WeaponType::Whip,
            WeaponType::MagicWand,
            WeaponType::Knife,
            WeaponType::Garlic,
            WeaponType::Bible,
            WeaponType::ThunderRing,
            WeaponType::Cross,
            WeaponType::FireWand,
        ];
        for wt in base {
            let name = weapon_display_name(wt);
            assert!(
                !name.is_empty(),
                "{wt:?} should have a non-empty display name"
            );
        }
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app
    }

    fn advance(app: &mut App, secs: f32) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(secs));
    }

    /// Spawns an `EvolutionNotification` root with a dummy text entity.
    fn spawn_notification(app: &mut App, elapsed: f32) -> Entity {
        let text_entity = app.world_mut().spawn(TextColor(DEFAULT_TEXT_COLOR)).id();
        app.world_mut()
            .spawn(EvolutionNotification {
                elapsed,
                duration: DEFAULT_DISPLAY_DURATION,
                fade_start: DEFAULT_FADE_START,
                text_entity,
                text_color: DEFAULT_TEXT_COLOR,
            })
            .id()
    }

    // -----------------------------------------------------------------------
    // update_evolution_notification
    // -----------------------------------------------------------------------

    /// Notification is despawned once elapsed reaches the duration.
    #[test]
    fn notification_despawns_when_elapsed_reaches_duration() {
        let mut app = build_app();
        // Pre-set elapsed to just below duration so one tick pushes it over.
        let entity = spawn_notification(&mut app, DEFAULT_DISPLAY_DURATION - 0.001);

        advance(&mut app, 0.1);
        app.world_mut()
            .run_system_once(update_evolution_notification)
            .unwrap();
        app.world_mut().flush();

        assert!(
            app.world().get_entity(entity).is_err(),
            "notification should be despawned when elapsed >= duration"
        );
    }

    /// Notification still exists before its duration is reached.
    #[test]
    fn notification_survives_before_duration() {
        let mut app = build_app();
        let entity = spawn_notification(&mut app, 0.0);

        advance(&mut app, 0.1);
        app.world_mut()
            .run_system_once(update_evolution_notification)
            .unwrap();

        assert!(
            app.world().get_entity(entity).is_ok(),
            "notification should survive before its duration"
        );
    }

    /// Alpha remains 1.0 while elapsed is before fade_start.
    #[test]
    fn alpha_is_full_before_fade_start() {
        let mut app = build_app();
        let entity = spawn_notification(&mut app, 0.0);
        let text_entity = app
            .world()
            .get::<EvolutionNotification>(entity)
            .unwrap()
            .text_entity;

        // Advance to just before fade_start.
        advance(&mut app, DEFAULT_FADE_START - 0.1);
        app.world_mut()
            .run_system_once(update_evolution_notification)
            .unwrap();

        let color = app.world().get::<TextColor>(text_entity).unwrap().0;
        let alpha = color.to_srgba().alpha;
        assert!(
            (alpha - 1.0).abs() < 1e-4,
            "alpha should be 1.0 before fade_start, got {alpha}"
        );
    }

    /// Alpha is less than 1.0 after fade_start has passed.
    #[test]
    fn alpha_decreases_after_fade_start() {
        let mut app = build_app();
        let entity = spawn_notification(&mut app, DEFAULT_FADE_START + 0.1);
        let text_entity = app
            .world()
            .get::<EvolutionNotification>(entity)
            .unwrap()
            .text_entity;

        advance(&mut app, 0.0); // just needs a non-zero dt; already past fade_start
        // Actually we need a positive dt; advance a small amount.
        advance(&mut app, 0.01);
        app.world_mut()
            .run_system_once(update_evolution_notification)
            .unwrap();

        let color = app.world().get::<TextColor>(text_entity).unwrap().0;
        let alpha = color.to_srgba().alpha;
        assert!(
            alpha < 1.0,
            "alpha should be less than 1.0 after fade_start, got {alpha}"
        );
    }

    /// elapsed advances by the time delta each frame.
    #[test]
    fn elapsed_advances_each_frame() {
        let mut app = build_app();
        let entity = spawn_notification(&mut app, 0.0);

        advance(&mut app, 0.5);
        app.world_mut()
            .run_system_once(update_evolution_notification)
            .unwrap();

        let notif = app.world().get::<EvolutionNotification>(entity).unwrap();
        assert!(
            notif.elapsed >= 0.5 - f32::EPSILON,
            "elapsed should advance by delta_secs; got {}",
            notif.elapsed
        );
    }
}
