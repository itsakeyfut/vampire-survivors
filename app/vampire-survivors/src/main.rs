use bevy::prelude::*;
use vs_assets::GameAssetsPlugin;
use vs_audio::GameAudioPlugin;
use vs_core::GameCorePlugin;
use vs_ui::GameUIPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Vampire Survivors Clone".into(),
                resolution: (1280, 720).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        // Load assets first (other plugins may reference them)
        .add_plugins(GameAssetsPlugin)
        // Core game logic (ECS, systems)
        .add_plugins(GameCorePlugin)
        // UI and camera (depends on GameCorePlugin for AppState + Player)
        .add_plugins(GameUIPlugin)
        // Audio (receives core events for BGM/SFX switching)
        .add_plugins(GameAudioPlugin)
        .run();
}
