//! # vs-ui
//!
//! UI and camera systems for the Vampire Survivors clone.
//!
//! ## Modules
//!
//! - [`camera`]: orthographic camera setup and player-follow system
//! - [`components`]: reusable UI components (`MenuButton`, `ButtonAction`)
//! - [`config`]: UI style asset loading, [`UiStyleParams`], and hot-reload
//! - [`screens`]: per-state screen implementations
//! - [`styles`]: `DEFAULT_*` color, font-size, and layout fallback constants

use bevy::prelude::*;
use vs_core::states::AppState;
use vs_core::systems::xp::choices::generate_level_up_choices;

pub mod camera;
pub mod components;
pub mod config;
pub mod hud;
pub mod screens;
pub mod styles;

/// UI plugin.
///
/// Spawns the camera at startup and wires all UI systems.
/// Add this plugin to the app after [`vs_core::GameCorePlugin`].
pub struct GameUIPlugin;

impl Plugin for GameUIPlugin {
    fn build(&self, app: &mut App) {
        app
            // UI style config: asset loading + hot-reload system.
            .add_plugins(config::UiConfigPlugin)
            // Camera is permanent — needed for title / menu rendering too.
            .add_systems(Startup, camera::setup_camera)
            // Title screen
            .add_systems(OnEnter(AppState::Title), screens::title::setup_title_screen)
            // Game-over screen
            .add_systems(
                OnEnter(AppState::GameOver),
                screens::game_over::setup_game_over_screen,
            )
            // Victory screen
            .add_systems(
                OnEnter(AppState::Victory),
                screens::victory::setup_victory_screen,
            )
            // Level-up card selection overlay.
            // Must run after generate_level_up_choices so LevelUpChoices is
            // already populated when the UI reads it.
            .add_systems(
                OnEnter(AppState::LevelUp),
                screens::level_up::setup_level_up_screen.after(generate_level_up_choices),
            )
            // Card-specific hover/press colors run in all states.
            // Must run after handle_button_interaction so card colors take
            // precedence (cards carry both MenuButton and UpgradeCardHud).
            .add_systems(
                Update,
                hud::upgrade_card::handle_card_interaction
                    .after(components::handle_button_interaction),
            )
            // Gameplay HUD — spawned on enter, despawned on exit automatically.
            .add_systems(
                OnEnter(AppState::Playing),
                hud::gameplay::setup_gameplay_hud,
            )
            // Gameplay HUD update systems — run only during active play.
            .add_systems(
                Update,
                (
                    hud::gameplay::hp_bar::update_hp_bar,
                    hud::gameplay::xp_bar::update_xp_bar,
                    hud::gameplay::timer::update_timer,
                    hud::gameplay::level::update_level_text,
                    hud::gameplay::evolution_notification::update_evolution_notification,
                    hud::gameplay::boss_warning::spawn_boss_warning,
                    hud::gameplay::boss_warning::update_boss_warning,
                    (
                        hud::gameplay::boss_hp_bar::maybe_spawn_boss_hp_bar,
                        hud::gameplay::boss_hp_bar::update_boss_hp_bar_world,
                    )
                        .chain(),
                )
                    .run_if(in_state(AppState::Playing)),
            )
            // Evolution notification observer — spawns a toast when a weapon evolves.
            .add_observer(hud::gameplay::evolution_notification::on_weapon_evolved)
            // Smooth player-follow only runs during active gameplay.
            .add_systems(
                Update,
                camera::camera_follow_player.run_if(in_state(AppState::Playing)),
            )
            // Button interaction runs every frame in any state.
            .add_systems(Update, components::handle_button_interaction);
    }
}
