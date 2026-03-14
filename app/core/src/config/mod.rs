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
//! | Module      | Contents |
//! |-------------|----------|
//! | [`player`]    | `PlayerConfig` + `PlayerParams` SystemParam bundle |
//! | [`enemy`]     | `EnemyConfig`, `EnemyStatsEntry` + `EnemyParams` SystemParam bundle |
//! | [`game`]      | `GameConfig` + `GameParams` SystemParam bundle |
//! | [`weapon`]    | Per-weapon configs (`WhipConfig`, `MagicWandConfig`, `KnifeConfig`, …) |
//! | [`passive`]   | `PassiveConfig` + `PassiveParams` SystemParam bundle |
//! | [`character`] | `CharacterConfig`, `CharacterBaseStats` + `CharacterParams` SystemParam bundle |

pub mod character;
pub mod enemy;
pub mod game;
pub mod passive;
pub mod player;
pub mod stage;
pub mod weapon;

pub use character::*;
pub use enemy::*;
pub use game::*;
pub use passive::*;
pub use player::*;
pub use stage::*;
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
/// # Usage (two-step form: deserialize as `$partial`, then convert to `$asset` via `From`)
/// ```ignore
/// ron_asset_loader!(MyConfigLoader, MyConfigPartial => MyConfig);
/// ```
macro_rules! ron_asset_loader {
    // Two-step form: deserialize as $partial, then convert to $asset via From
    ($loader:ident, $partial:ty => $asset:ty) => {
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
                let options = ron::Options::default()
                    .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME);
                let partial: $partial = options
                    .from_bytes(&bytes)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                Ok(<$asset>::from(partial))
            }

            fn extensions(&self) -> &[&str] {
                &["ron"]
            }
        }
    };
}

// ---------------------------------------------------------------------------
// Font asset paths and preload resource
// ---------------------------------------------------------------------------

/// Path to DotGothic16, the game's sole UI font.
///
/// DotGothic16 is a Japanese pixel font that covers hiragana, katakana, and
/// common kanji, providing the dotted game aesthetic for both JP and EN text.
const FONT_TEXT: &str = "fonts/DotGothic16/DotGothic16-Regular.ttf";

/// Holds a strong font handle so the asset cache is never evicted.
///
/// Also used by [`wait_for_configs`] to block the `Loading → Title` transition
/// until the font is fully loaded — ensuring screens never render with the
/// built-in fallback font.
#[derive(Resource)]
struct FontLoadHandles {
    game: Handle<Font>,
}

// Non-weapon config loaders
ron_asset_loader!(PlayerConfigLoader, PlayerConfigPartial => PlayerConfig);
ron_asset_loader!(EnemyConfigLoader, EnemyConfigPartial => EnemyConfig);
ron_asset_loader!(GameConfigLoader, GameConfigPartial => GameConfig);
ron_asset_loader!(PassiveConfigLoader, PassiveConfigPartial => PassiveConfig);
ron_asset_loader!(CharacterConfigLoader, CharacterConfigPartial => CharacterConfig);
ron_asset_loader!(StageConfigLoader, StageConfigPartial => StageConfig);

// Per-weapon config loaders
ron_asset_loader!(WhipConfigLoader, WhipConfigPartial => WhipConfig);
ron_asset_loader!(MagicWandConfigLoader, MagicWandConfigPartial => MagicWandConfig);
ron_asset_loader!(KnifeConfigLoader, KnifeConfigPartial => KnifeConfig);
ron_asset_loader!(GarlicConfigLoader, GarlicConfigPartial => GarlicConfig);
ron_asset_loader!(BibleConfigLoader, BibleConfigPartial => BibleConfig);
ron_asset_loader!(ThunderRingConfigLoader, ThunderRingConfigPartial => ThunderRingConfig);
ron_asset_loader!(CrossConfigLoader, CrossConfigPartial => CrossConfig);
ron_asset_loader!(FireWandConfigLoader, FireWandConfigPartial => FireWandConfig);

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
    passive_handle: Res<'w, PassiveConfigHandle>,
    passive_assets: Res<'w, Assets<PassiveConfig>>,
    whip_handle: Res<'w, WhipConfigHandle>,
    whip_assets: Res<'w, Assets<WhipConfig>>,
    magic_wand_handle: Res<'w, MagicWandConfigHandle>,
    magic_wand_assets: Res<'w, Assets<MagicWandConfig>>,
    knife_handle: Res<'w, KnifeConfigHandle>,
    knife_assets: Res<'w, Assets<KnifeConfig>>,
    garlic_handle: Res<'w, GarlicConfigHandle>,
    garlic_assets: Res<'w, Assets<GarlicConfig>>,
    bible_handle: Res<'w, BibleConfigHandle>,
    bible_assets: Res<'w, Assets<BibleConfig>>,
    thunder_ring_handle: Res<'w, ThunderRingConfigHandle>,
    thunder_ring_assets: Res<'w, Assets<ThunderRingConfig>>,
    cross_handle: Res<'w, CrossConfigHandle>,
    cross_assets: Res<'w, Assets<CrossConfig>>,
    fire_wand_handle: Res<'w, FireWandConfigHandle>,
    fire_wand_assets: Res<'w, Assets<FireWandConfig>>,
    character_handle: Res<'w, CharacterConfigHandle>,
    character_assets: Res<'w, Assets<CharacterConfig>>,
    stage_handle: Res<'w, StageConfigHandle>,
    stage_assets: Res<'w, Assets<StageConfig>>,
    font_handles: Res<'w, FontLoadHandles>,
    font_assets: Res<'w, Assets<Font>>,
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Plugin for game configuration management.
///
/// Registers all RON asset loaders, loads the config files from
/// `assets/config/`, inserts handles as resources, and wires hot-reload
/// systems. Transitions `Loading → Title` once all configs are ready.
///
/// **Must be registered in the binary** (`main.rs`), after `DefaultPlugins`
/// and before `GameCorePlugin`.
pub struct GameConfigPlugin;

impl Plugin for GameConfigPlugin {
    fn build(&self, app: &mut App) {
        info!("🔧 Initializing GameConfigPlugin...");

        // Register non-weapon asset types and loaders.
        app.init_asset::<PlayerConfig>()
            .register_asset_loader(PlayerConfigLoader)
            .init_asset::<EnemyConfig>()
            .register_asset_loader(EnemyConfigLoader)
            .init_asset::<GameConfig>()
            .register_asset_loader(GameConfigLoader)
            .init_asset::<PassiveConfig>()
            .register_asset_loader(PassiveConfigLoader)
            .init_asset::<CharacterConfig>()
            .register_asset_loader(CharacterConfigLoader)
            .init_asset::<StageConfig>()
            .register_asset_loader(StageConfigLoader);

        // Register per-weapon asset types and loaders.
        app.init_asset::<WhipConfig>()
            .register_asset_loader(WhipConfigLoader)
            .init_asset::<MagicWandConfig>()
            .register_asset_loader(MagicWandConfigLoader)
            .init_asset::<KnifeConfig>()
            .register_asset_loader(KnifeConfigLoader)
            .init_asset::<GarlicConfig>()
            .register_asset_loader(GarlicConfigLoader)
            .init_asset::<BibleConfig>()
            .register_asset_loader(BibleConfigLoader)
            .init_asset::<ThunderRingConfig>()
            .register_asset_loader(ThunderRingConfigLoader)
            .init_asset::<CrossConfig>()
            .register_asset_loader(CrossConfigLoader)
            .init_asset::<FireWandConfig>()
            .register_asset_loader(FireWandConfigLoader);

        // Load all config files and insert handles as resources.
        let asset_server = app.world_mut().resource::<AssetServer>();
        let player_handle: Handle<PlayerConfig> = asset_server.load("config/player.ron");
        let enemy_handle: Handle<EnemyConfig> = asset_server.load("config/enemy.ron");
        let game_handle: Handle<GameConfig> = asset_server.load("config/game.ron");
        let passive_handle: Handle<PassiveConfig> = asset_server.load("config/passive.ron");
        let whip_handle: Handle<WhipConfig> = asset_server.load("config/weapons/whip.ron");
        let magic_wand_handle: Handle<MagicWandConfig> =
            asset_server.load("config/weapons/magic_wand.ron");
        let knife_handle: Handle<KnifeConfig> = asset_server.load("config/weapons/knife.ron");
        let garlic_handle: Handle<GarlicConfig> = asset_server.load("config/weapons/garlic.ron");
        let bible_handle: Handle<BibleConfig> = asset_server.load("config/weapons/bible.ron");
        let thunder_ring_handle: Handle<ThunderRingConfig> =
            asset_server.load("config/weapons/thunder_ring.ron");
        let cross_handle: Handle<CrossConfig> = asset_server.load("config/weapons/cross.ron");
        let fire_wand_handle: Handle<FireWandConfig> =
            asset_server.load("config/weapons/fire_wand.ron");
        let character_handle: Handle<CharacterConfig> = asset_server.load("config/character.ron");
        let stage_handle: Handle<StageConfig> = asset_server.load("config/stage.ron");

        let font_text: Handle<Font> = asset_server.load(FONT_TEXT);

        app.insert_resource(PlayerConfigHandle(player_handle))
            .insert_resource(EnemyConfigHandle(enemy_handle))
            .insert_resource(GameConfigHandle(game_handle))
            .insert_resource(PassiveConfigHandle(passive_handle))
            .insert_resource(WhipConfigHandle(whip_handle))
            .insert_resource(MagicWandConfigHandle(magic_wand_handle))
            .insert_resource(KnifeConfigHandle(knife_handle))
            .insert_resource(GarlicConfigHandle(garlic_handle))
            .insert_resource(BibleConfigHandle(bible_handle))
            .insert_resource(ThunderRingConfigHandle(thunder_ring_handle))
            .insert_resource(CrossConfigHandle(cross_handle))
            .insert_resource(FireWandConfigHandle(fire_wand_handle))
            .insert_resource(CharacterConfigHandle(character_handle))
            .insert_resource(StageConfigHandle(stage_handle))
            .insert_resource(FontLoadHandles { game: font_text });

        // Hot-reload systems run in all states so live-editing always works.
        app.add_systems(
            Update,
            (
                hot_reload_player_config,
                hot_reload_enemy_config,
                hot_reload_game_config,
                hot_reload_character_config,
                hot_reload_stage_config,
            ),
        );

        // Transition Loading → Title once all required configs are ready.
        app.add_systems(Update, wait_for_configs.run_if(in_state(AppState::Loading)));

        info!(
            "✅ GameConfigPlugin initialized (player, enemy, game, passive, character, stage, whip, magic_wand, knife, garlic, bible, thunder_ring, cross, fire_wand configs loading)"
        );
    }
}

// ---------------------------------------------------------------------------
// wait_for_configs
// ---------------------------------------------------------------------------

/// Transitions from `Loading` → `Title` once all required RON configs are ready.
fn wait_for_configs(configs: AllConfigs, mut next_state: ResMut<NextState<AppState>>) {
    let all_ready = configs
        .player_assets
        .get(&configs.player_handle.0)
        .is_some()
        && configs.enemy_assets.get(&configs.enemy_handle.0).is_some()
        && configs.game_assets.get(&configs.game_handle.0).is_some()
        && configs
            .passive_assets
            .get(&configs.passive_handle.0)
            .is_some()
        && configs.whip_assets.get(&configs.whip_handle.0).is_some()
        && configs
            .magic_wand_assets
            .get(&configs.magic_wand_handle.0)
            .is_some()
        && configs.knife_assets.get(&configs.knife_handle.0).is_some()
        && configs
            .garlic_assets
            .get(&configs.garlic_handle.0)
            .is_some()
        && configs.bible_assets.get(&configs.bible_handle.0).is_some()
        && configs
            .thunder_ring_assets
            .get(&configs.thunder_ring_handle.0)
            .is_some()
        && configs.cross_assets.get(&configs.cross_handle.0).is_some()
        && configs
            .fire_wand_assets
            .get(&configs.fire_wand_handle.0)
            .is_some()
        && configs
            .character_assets
            .get(&configs.character_handle.0)
            .is_some()
        && configs.stage_assets.get(&configs.stage_handle.0).is_some()
        && configs
            .font_assets
            .get(&configs.font_handles.game)
            .is_some();

    if all_ready {
        info!("✅ All configs loaded, transitioning to Title");
        next_state.set(AppState::Title);
    }
}
