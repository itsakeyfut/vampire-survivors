//! UI configuration loaded from RON files.
//!
//! Each logical group of UI settings lives in its own submodule so it can be
//! tuned independently:
//!
//! | File                                    | Config type                | Controls                           |
//! |-----------------------------------------|----------------------------|------------------------------------|
//! | `config/ui/styles.ron`                  | [`UiStyleConfig`]          | Colors, font sizes, button sizes   |
//! | `config/ui/screen/level_up.ron`         | [`LevelUpScreenConfig`]    | Level-up overlay and heading color |
//! | `config/ui/hud/screen_heading.ron`      | [`ScreenHeadingHudConfig`] | Heading font size and margin       |
//! | `config/ui/hud/menu_button.ron`         | [`MenuButtonHudConfig`]    | Button dimensions, colors, font    |
//! | `config/ui/hud/upgrade_card.ron`        | [`UpgradeCardHudConfig`]   | Card layout, colors, typography    |
//!
//! All files are watched by Bevy's asset server, so edits take effect while
//! the game is running (hot-reload).

pub mod hud;
pub mod level_up;
pub mod styles;

pub use hud::{
    MenuButtonHudConfig, MenuButtonHudConfigHandle, MenuButtonHudParams, ScreenHeadingHudConfig,
    ScreenHeadingHudConfigHandle, ScreenHeadingHudParams, UpgradeCardHudConfig,
    UpgradeCardHudConfigHandle, UpgradeCardHudParams,
};
pub use level_up::{
    LevelUpScreenConfig, LevelUpScreenConfigHandle, LevelUpScreenParams, hot_reload_level_up_screen,
};
pub use styles::{
    SrgbColor, SrgbaColor, TitleHeadingText, TitleScreenBg, UiStyleConfig, UiStyleConfigHandle,
    UiStyleParams, hot_reload_ui_style,
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

// Screen config loaders
ron_asset_loader!(UiStyleConfigLoader, UiStyleConfig);
ron_asset_loader!(LevelUpScreenConfigLoader, LevelUpScreenConfig);

// HUD config loaders
ron_asset_loader!(ScreenHeadingHudConfigLoader, ScreenHeadingHudConfig);
ron_asset_loader!(MenuButtonHudConfigLoader, MenuButtonHudConfig);
ron_asset_loader!(UpgradeCardHudConfigLoader, UpgradeCardHudConfig);

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
        // Screen configs
        app.init_asset::<UiStyleConfig>()
            .register_asset_loader(UiStyleConfigLoader)
            .init_asset::<LevelUpScreenConfig>()
            .register_asset_loader(LevelUpScreenConfigLoader);

        // HUD configs
        app.init_asset::<ScreenHeadingHudConfig>()
            .register_asset_loader(ScreenHeadingHudConfigLoader)
            .init_asset::<MenuButtonHudConfig>()
            .register_asset_loader(MenuButtonHudConfigLoader)
            .init_asset::<UpgradeCardHudConfig>()
            .register_asset_loader(UpgradeCardHudConfigLoader);

        let asset_server = app.world_mut().resource::<AssetServer>();

        // Load screen configs
        let style_handle: Handle<UiStyleConfig> = asset_server.load("config/ui/styles.ron");
        let level_up_handle: Handle<LevelUpScreenConfig> =
            asset_server.load("config/ui/screen/level_up.ron");

        // Load HUD configs
        let screen_heading_handle: Handle<ScreenHeadingHudConfig> =
            asset_server.load("config/ui/hud/screen_heading.ron");
        let menu_button_handle: Handle<MenuButtonHudConfig> =
            asset_server.load("config/ui/hud/menu_button.ron");
        let upgrade_card_handle: Handle<UpgradeCardHudConfig> =
            asset_server.load("config/ui/hud/upgrade_card.ron");

        app.insert_resource(UiStyleConfigHandle(style_handle))
            .insert_resource(LevelUpScreenConfigHandle(level_up_handle))
            .insert_resource(ScreenHeadingHudConfigHandle(screen_heading_handle))
            .insert_resource(MenuButtonHudConfigHandle(menu_button_handle))
            .insert_resource(UpgradeCardHudConfigHandle(upgrade_card_handle));

        app.add_systems(Update, hot_reload_ui_style)
            .add_systems(Update, hot_reload_level_up_screen)
            .add_systems(
                Update,
                crate::hud::screen_heading::hot_reload_screen_heading_hud,
            )
            .add_systems(Update, crate::hud::menu_button::hot_reload_menu_button_hud)
            .add_systems(
                Update,
                crate::hud::upgrade_card::hot_reload_upgrade_card_hud,
            );

        info!("✅ UiConfigPlugin initialized");
    }
}
