use bevy::prelude::*;
use bevy_kira_audio::AudioPlugin;

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AudioPlugin);
    }
}
