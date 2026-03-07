//! UI configuration loaded from RON files.
//!
//! Each logical group of UI settings lives in its own submodule so it can be
//! tuned independently:
//!
//! | File                                              | Config type                  | Controls                           |
//! |---------------------------------------------------|------------------------------|------------------------------------|
//! | `config/ui/styles.ron`                            | [`UiStyleConfig`]            | Shared background and title colors |
//! | `config/ui/screen/level_up.ron`                   | [`LevelUpScreenConfig`]      | Level-up overlay and heading color |
//! | `config/ui/screen/victory.ron`                    | [`VictoryScreenConfig`]      | Victory screen colors and spacing  |
//! | `config/ui/hud/screen_heading.ron`                | [`ScreenHeadingHudConfig`]   | Heading font size and margin       |
//! | `config/ui/hud/menu_button.ron`                   | [`MenuButtonHudConfig`]      | Button dimensions, colors, font    |
//! | `config/ui/hud/upgrade_card.ron`                  | [`UpgradeCardHudConfig`]     | Card layout, colors, typography    |
//! | `config/ui/hud/gameplay/hp_bar.ron`               | [`HpBarHudConfig`]           | HP bar dimensions, radius, colors  |
//! | `config/ui/hud/gameplay/xp_bar.ron`               | [`XpBarHudConfig`]           | XP bar height and colors           |
//! | `config/ui/hud/gameplay/timer.ron`                | [`TimerHudConfig`]           | Timer font size and color          |
//! | `config/ui/hud/gameplay/level.ron`                | [`LevelHudConfig`]           | Level label font size and color    |
//! | `config/ui/hud/gameplay/layout.ron`               | [`GameplayHudLayoutConfig`]  | Widget anchor positions            |
//! | `config/ui/hud/gameplay/boss_warning.ron`         | [`BossWarningHudConfig`]     | Boss warning overlay settings      |
//! | `config/ui/hud/gameplay/boss_hp_bar.ron`          | [`BossHpBarHudConfig`]       | Boss HP bar dimensions and colors  |
//!
//! All files are watched by Bevy's asset server, so edits take effect while
//! the game is running (hot-reload).

pub mod hud;
pub mod level_up;
pub mod styles;
pub mod victory;

pub use hud::{
    BossHpBarHudConfig, BossHpBarHudConfigHandle, BossHpBarHudParams, BossWarningHudConfig,
    BossWarningHudConfigHandle, BossWarningHudParams, EvolutionNotificationHudConfig,
    EvolutionNotificationHudConfigHandle, EvolutionNotificationHudParams, GameplayHudLayoutConfig,
    GameplayHudLayoutConfigHandle, GameplayHudLayoutParams, HpBarHudConfig, HpBarHudConfigHandle,
    HpBarHudParams, KillCountHudConfig, KillCountHudConfigHandle, KillCountHudParams,
    LevelHudConfig, LevelHudConfigHandle, LevelHudParams, MenuButtonHudConfig,
    MenuButtonHudConfigHandle, MenuButtonHudParams, ScreenHeadingHudConfig,
    ScreenHeadingHudConfigHandle, ScreenHeadingHudParams, TimerHudConfig, TimerHudConfigHandle,
    TimerHudParams, UpgradeCardHudConfig, UpgradeCardHudConfigHandle, UpgradeCardHudParams,
    WeaponSlotsHudConfig, WeaponSlotsHudConfigHandle, WeaponSlotsHudParams, XpBarHudConfig,
    XpBarHudConfigHandle, XpBarHudParams,
};
pub use level_up::{
    LevelUpScreenConfig, LevelUpScreenConfigHandle, LevelUpScreenParams, hot_reload_level_up_screen,
};
pub use styles::{
    SrgbColor, SrgbaColor, TitleHeadingText, TitleScreenBg, UiStyleConfig, UiStyleConfigHandle,
    UiStyleParams, hot_reload_ui_style,
};
pub use victory::{
    VictoryScreenConfig, VictoryScreenConfigHandle, VictoryScreenParams, hot_reload_victory_screen,
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
ron_asset_loader!(VictoryScreenConfigLoader, VictoryScreenConfig);

// HUD config loaders
ron_asset_loader!(ScreenHeadingHudConfigLoader, ScreenHeadingHudConfig);
ron_asset_loader!(MenuButtonHudConfigLoader, MenuButtonHudConfig);
ron_asset_loader!(UpgradeCardHudConfigLoader, UpgradeCardHudConfig);

// Gameplay HUD config loaders
ron_asset_loader!(HpBarHudConfigLoader, HpBarHudConfig);
ron_asset_loader!(XpBarHudConfigLoader, XpBarHudConfig);
ron_asset_loader!(TimerHudConfigLoader, TimerHudConfig);
ron_asset_loader!(LevelHudConfigLoader, LevelHudConfig);
ron_asset_loader!(GameplayHudLayoutConfigLoader, GameplayHudLayoutConfig);
ron_asset_loader!(
    EvolutionNotificationHudConfigLoader,
    EvolutionNotificationHudConfig
);
ron_asset_loader!(BossWarningHudConfigLoader, BossWarningHudConfig);
ron_asset_loader!(BossHpBarHudConfigLoader, BossHpBarHudConfig);
ron_asset_loader!(KillCountHudConfigLoader, KillCountHudConfig);
ron_asset_loader!(WeaponSlotsHudConfigLoader, WeaponSlotsHudConfig);

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
            .register_asset_loader(LevelUpScreenConfigLoader)
            .init_asset::<VictoryScreenConfig>()
            .register_asset_loader(VictoryScreenConfigLoader);

        // HUD configs
        app.init_asset::<ScreenHeadingHudConfig>()
            .register_asset_loader(ScreenHeadingHudConfigLoader)
            .init_asset::<MenuButtonHudConfig>()
            .register_asset_loader(MenuButtonHudConfigLoader)
            .init_asset::<UpgradeCardHudConfig>()
            .register_asset_loader(UpgradeCardHudConfigLoader);

        // Gameplay HUD configs
        app.init_asset::<HpBarHudConfig>()
            .register_asset_loader(HpBarHudConfigLoader)
            .init_asset::<XpBarHudConfig>()
            .register_asset_loader(XpBarHudConfigLoader)
            .init_asset::<TimerHudConfig>()
            .register_asset_loader(TimerHudConfigLoader)
            .init_asset::<LevelHudConfig>()
            .register_asset_loader(LevelHudConfigLoader)
            .init_asset::<GameplayHudLayoutConfig>()
            .register_asset_loader(GameplayHudLayoutConfigLoader)
            .init_asset::<EvolutionNotificationHudConfig>()
            .register_asset_loader(EvolutionNotificationHudConfigLoader)
            .init_asset::<BossWarningHudConfig>()
            .register_asset_loader(BossWarningHudConfigLoader)
            .init_asset::<BossHpBarHudConfig>()
            .register_asset_loader(BossHpBarHudConfigLoader)
            .init_asset::<KillCountHudConfig>()
            .register_asset_loader(KillCountHudConfigLoader)
            .init_asset::<WeaponSlotsHudConfig>()
            .register_asset_loader(WeaponSlotsHudConfigLoader);

        let asset_server = app.world_mut().resource::<AssetServer>();

        // Load screen configs
        let style_handle: Handle<UiStyleConfig> = asset_server.load("config/ui/styles.ron");
        let level_up_handle: Handle<LevelUpScreenConfig> =
            asset_server.load("config/ui/screen/level_up.ron");
        let victory_handle: Handle<VictoryScreenConfig> =
            asset_server.load("config/ui/screen/victory.ron");

        // Load HUD configs
        let screen_heading_handle: Handle<ScreenHeadingHudConfig> =
            asset_server.load("config/ui/hud/screen_heading.ron");
        let menu_button_handle: Handle<MenuButtonHudConfig> =
            asset_server.load("config/ui/hud/menu_button.ron");
        let upgrade_card_handle: Handle<UpgradeCardHudConfig> =
            asset_server.load("config/ui/hud/upgrade_card.ron");

        // Load gameplay HUD configs
        let hp_bar_handle: Handle<HpBarHudConfig> =
            asset_server.load("config/ui/hud/gameplay/hp_bar.ron");
        let xp_bar_handle: Handle<XpBarHudConfig> =
            asset_server.load("config/ui/hud/gameplay/xp_bar.ron");
        let timer_handle: Handle<TimerHudConfig> =
            asset_server.load("config/ui/hud/gameplay/timer.ron");
        let level_handle: Handle<LevelHudConfig> =
            asset_server.load("config/ui/hud/gameplay/level.ron");
        let layout_handle: Handle<GameplayHudLayoutConfig> =
            asset_server.load("config/ui/hud/gameplay/layout.ron");
        let evolution_notif_handle: Handle<EvolutionNotificationHudConfig> =
            asset_server.load("config/ui/hud/gameplay/evolution_notification.ron");
        let boss_warning_handle: Handle<BossWarningHudConfig> =
            asset_server.load("config/ui/hud/gameplay/boss_warning.ron");
        let boss_hp_bar_handle: Handle<BossHpBarHudConfig> =
            asset_server.load("config/ui/hud/gameplay/boss_hp_bar.ron");
        let kill_count_handle: Handle<KillCountHudConfig> =
            asset_server.load("config/ui/hud/gameplay/kill_count.ron");
        let weapon_slots_handle: Handle<WeaponSlotsHudConfig> =
            asset_server.load("config/ui/hud/gameplay/weapon_slots.ron");

        app.insert_resource(UiStyleConfigHandle(style_handle))
            .insert_resource(LevelUpScreenConfigHandle(level_up_handle))
            .insert_resource(VictoryScreenConfigHandle(victory_handle))
            .insert_resource(ScreenHeadingHudConfigHandle(screen_heading_handle))
            .insert_resource(MenuButtonHudConfigHandle(menu_button_handle))
            .insert_resource(UpgradeCardHudConfigHandle(upgrade_card_handle))
            .insert_resource(HpBarHudConfigHandle(hp_bar_handle))
            .insert_resource(XpBarHudConfigHandle(xp_bar_handle))
            .insert_resource(TimerHudConfigHandle(timer_handle))
            .insert_resource(LevelHudConfigHandle(level_handle))
            .insert_resource(GameplayHudLayoutConfigHandle(layout_handle))
            .insert_resource(EvolutionNotificationHudConfigHandle(evolution_notif_handle))
            .insert_resource(BossWarningHudConfigHandle(boss_warning_handle))
            .insert_resource(BossHpBarHudConfigHandle(boss_hp_bar_handle))
            .insert_resource(KillCountHudConfigHandle(kill_count_handle))
            .insert_resource(WeaponSlotsHudConfigHandle(weapon_slots_handle));

        app.add_systems(Update, hot_reload_ui_style)
            .add_systems(Update, hot_reload_level_up_screen)
            .add_systems(Update, hot_reload_victory_screen)
            .add_systems(
                Update,
                crate::hud::screen_heading::hot_reload_screen_heading_hud,
            )
            .add_systems(Update, crate::hud::menu_button::hot_reload_menu_button_hud)
            .add_systems(
                Update,
                crate::hud::upgrade_card::hot_reload_upgrade_card_hud,
            )
            .add_systems(Update, crate::hud::gameplay::hp_bar::hot_reload_hp_bar_hud)
            .add_systems(Update, crate::hud::gameplay::xp_bar::hot_reload_xp_bar_hud)
            .add_systems(Update, crate::hud::gameplay::timer::hot_reload_timer_hud)
            .add_systems(Update, crate::hud::gameplay::level::hot_reload_level_hud)
            .add_systems(Update, crate::hud::gameplay::hot_reload_gameplay_layout)
            .add_systems(
                Update,
                crate::hud::gameplay::kill_count::hot_reload_kill_count_hud,
            )
            .add_systems(
                Update,
                crate::hud::gameplay::weapon_slots::hot_reload_weapon_slots_hud,
            );

        info!("✅ UiConfigPlugin initialized");
    }
}
