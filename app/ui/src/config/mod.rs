//! UI configuration loaded from RON files.
//!
//! Each logical group of UI settings lives in its own submodule so it can be
//! tuned independently:
//!
//! | File                         | Config type        | Controls                         |
//! |------------------------------|--------------------|----------------------------------|
//! | `config/ui/styles.ron`       | [`UiStyleConfig`]  | Colors, font sizes, button sizes |
//!
//! All files are watched by Bevy's asset server, so edits take effect while
//! the game is running (hot-reload).

pub mod styles;

pub use styles::{
    SrgbColor, TitleButtonLabel, TitleHeadingText, TitleScreenBg, TitleStartButton, UiStyleConfig,
    UiStyleConfigHandle, UiStyleParams, hot_reload_ui_style,
};

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;

// ---------------------------------------------------------------------------
// RON asset loader macro (mirrors the pattern in vs-core/src/config/mod.rs)
// ---------------------------------------------------------------------------

/// Generates a RON-based [`AssetLoader`] implementation for a config type.
///
/// All UI config assets use identical loading logic (read bytes → `ron::de::from_bytes`),
/// so this macro eliminates the repetition while keeping each loader a distinct type.
macro_rules! ron_asset_loader {
    ($loader:ident, $asset:ty) => {
        #[derive(Default)]
        struct $loader;

        impl AssetLoader for $loader {
            type Asset = $asset;
            type Settings = ();
            type Error = std::io::Error;

            async fn load(
                &self,
                reader: &mut dyn Reader,
                _settings: &Self::Settings,
                _load_context: &mut LoadContext<'_>,
            ) -> Result<Self::Asset, Self::Error> {
                let mut bytes = Vec::new();
                reader.read_to_end(&mut bytes).await?;
                ron::de::from_bytes(&bytes)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
            }

            fn extensions(&self) -> &[&str] {
                &["ron"]
            }
        }
    };
}

// Loader types (defined here so the macro stays local to this module)
ron_asset_loader!(UiStyleConfigLoader, UiStyleConfig);

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Plugin that registers all UI config asset types, loads the RON files, and
/// adds hot-reload systems.
///
/// Added automatically by [`crate::GameUIPlugin`].
pub struct UiConfigPlugin;

impl Plugin for UiConfigPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<UiStyleConfig>()
            .register_asset_loader(UiStyleConfigLoader);

        let asset_server = app.world_mut().resource::<AssetServer>();
        let style_handle: Handle<UiStyleConfig> = asset_server.load("config/ui/styles.ron");

        app.insert_resource(UiStyleConfigHandle(style_handle));

        app.add_systems(Update, hot_reload_ui_style);

        info!("✅ UiConfigPlugin initialized");
    }
}
