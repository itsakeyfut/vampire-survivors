//! Game configuration loaded from RON files.
//!
//! This module handles loading and hot-reloading of game configuration
//! from RON (Rusty Object Notation) files in the assets directory.
//!
//! Supports hot-reloading: edit config files while the game is running
//! and changes will be applied automatically.
//!
//! # Sub-modules
//!
//! | Module | Contents |
//! |--------|----------|
//! | [`player`] | `PlayerConfig` + `PlayerParams` SystemParam bundle |
//! | [`enemy`]  | `EnemyConfig`, `EnemyStatsEntry` + `EnemyParams` SystemParam bundle |
//! | [`game`]   | `GameConfig` + `GameParams` SystemParam bundle |
//! | [`weapon`] | `WeaponConfig` + `WeaponParams` SystemParam bundle |

pub mod enemy;
pub mod game;
pub mod player;
pub mod weapon;

pub use enemy::*;
pub use game::*;
pub use player::*;
pub use weapon::*;

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::states::AppState;

// ---------------------------------------------------------------------------
// RON asset loader macro
// ---------------------------------------------------------------------------

/// Generates a RON-based `AssetLoader` implementation for a config type.
///
/// All game config assets use identical loading logic (read bytes → `ron::de::from_bytes`),
/// so this macro eliminates the repetition while keeping each loader a distinct type.
///
/// # Usage
/// ```ignore
/// ron_asset_loader!(MyConfigLoader, MyConfig);
/// ```
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

// Loader types generated from the macro (all in mod.rs so the macro is local here)
ron_asset_loader!(PlayerConfigLoader, PlayerConfig);
ron_asset_loader!(EnemyConfigLoader, EnemyConfig);
ron_asset_loader!(GameConfigLoader, GameConfig);
ron_asset_loader!(WeaponConfigLoader, WeaponConfig);

// ---------------------------------------------------------------------------
// AllConfigs — private SystemParam for wait_for_configs
// ---------------------------------------------------------------------------

/// Bundles all config handle/asset pairs into a single `SystemParam` to stay
/// within Bevy's 16-parameter system limit as more configs are added.
#[derive(SystemParam)]
struct AllConfigs<'w> {
    player_handle: Res<'w, PlayerConfigHandle>,
    player_assets: Res<'w, Assets<PlayerConfig>>,
    enemy_handle: Res<'w, EnemyConfigHandle>,
    enemy_assets: Res<'w, Assets<EnemyConfig>>,
    game_handle: Res<'w, GameConfigHandle>,
    game_assets: Res<'w, Assets<GameConfig>>,
    weapon_handle: Res<'w, WeaponConfigHandle>,
    weapon_assets: Res<'w, Assets<WeaponConfig>>,
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Plugin for game configuration management.
///
/// Registers all RON asset loaders, loads the config files from
/// `assets/config/`, inserts handles as resources, and wires hot-reload
/// systems. Transitions `Loading → Title` once all three configs are ready.
///
/// **Must be registered in the binary** (`main.rs`), after `DefaultPlugins`
/// and before `GameCorePlugin`.
pub struct GameConfigPlugin;

impl Plugin for GameConfigPlugin {
    fn build(&self, app: &mut App) {
        info!("🔧 Initializing GameConfigPlugin...");

        // Register asset types and loaders.
        app.init_asset::<PlayerConfig>()
            .register_asset_loader(PlayerConfigLoader)
            .init_asset::<EnemyConfig>()
            .register_asset_loader(EnemyConfigLoader)
            .init_asset::<GameConfig>()
            .register_asset_loader(GameConfigLoader)
            .init_asset::<WeaponConfig>()
            .register_asset_loader(WeaponConfigLoader);

        // Load all config files and insert handles as resources.
        let asset_server = app.world_mut().resource::<AssetServer>();
        let player_handle: Handle<PlayerConfig> = asset_server.load("config/player.ron");
        let enemy_handle: Handle<EnemyConfig> = asset_server.load("config/enemy.ron");
        let game_handle: Handle<GameConfig> = asset_server.load("config/game.ron");
        let weapon_handle: Handle<WeaponConfig> = asset_server.load("config/weapons.ron");

        app.insert_resource(PlayerConfigHandle(player_handle))
            .insert_resource(EnemyConfigHandle(enemy_handle))
            .insert_resource(GameConfigHandle(game_handle))
            .insert_resource(WeaponConfigHandle(weapon_handle));

        // Hot-reload systems run in all states so live-editing always works.
        app.add_systems(
            Update,
            (
                hot_reload_player_config,
                hot_reload_enemy_config,
                hot_reload_game_config,
            ),
        );

        // Transition Loading → Title once all required configs are ready.
        app.add_systems(Update, wait_for_configs.run_if(in_state(AppState::Loading)));

        info!("✅ GameConfigPlugin initialized (player, enemy, game, weapon configs loading)");
    }
}

// ---------------------------------------------------------------------------
// wait_for_configs
// ---------------------------------------------------------------------------

/// Transitions from `Loading` → `Title` once all required RON configs are ready.
fn wait_for_configs(configs: AllConfigs, mut next_state: ResMut<NextState<AppState>>) {
    if configs
        .player_assets
        .get(&configs.player_handle.0)
        .is_some()
        && configs.enemy_assets.get(&configs.enemy_handle.0).is_some()
        && configs.game_assets.get(&configs.game_handle.0).is_some()
        && configs
            .weapon_assets
            .get(&configs.weapon_handle.0)
            .is_some()
    {
        info!("✅ All configs loaded (player, enemy, game, weapon), transitioning to Title");
        next_state.set(AppState::Title);
    }
}
