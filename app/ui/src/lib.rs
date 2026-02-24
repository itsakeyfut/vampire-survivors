//! # vs-ui
//!
//! UI and camera systems for the Vampire Survivors clone.
//!
//! ## Modules
//!
//! - [`camera`]: orthographic camera setup and player-follow system
//! - [`components`]: reusable UI components (`MenuButton`, `ButtonAction`)
//! - [`screens`]: per-state screen implementations
//! - [`styles`]: color, font-size, and layout constants

use bevy::prelude::*;
use vs_core::states::AppState;

pub mod camera;
pub mod components;
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
            // Camera is permanent — needed for title / menu rendering too.
            .add_systems(Startup, camera::setup_camera)
            // Title screen
            .add_systems(OnEnter(AppState::Title), screens::title::setup_title_screen)
            // Smooth player-follow only runs during active gameplay.
            .add_systems(
                Update,
                camera::camera_follow_player.run_if(in_state(AppState::Playing)),
            )
            // Button interaction runs every frame in any state.
            .add_systems(Update, components::handle_button_interaction);
    }
}
