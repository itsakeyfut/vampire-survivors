# Vampire Survivors クローン - クレートアーキテクチャ設計書

## 1. クレート構成概要

5クレート構成を採用。責務ごとにクレートを分離し、将来的な再利用性と並列コンパイルの恩恵を得る。

```
vampire-survivors (workspace)
├── app/core/           → クレート名: vs-core
├── app/ui/             → クレート名: vs-ui
├── app/audio/          → クレート名: vs-audio
├── app/assets/         → クレート名: vs-assets
└── app/vampire-survivors/ → クレート名: vs（バイナリ）
```

---

## 2. 各クレートの詳細

### 2.1 vs-core（コアゲームロジック）

**責務**: ゲームの本質的なロジック全体を担当。ECSのコンポーネント・システム・リソース・状態・UIの定義と実装。

```toml
# app/core/Cargo.toml
[package]
name = "vs-core"
version = "0.1.0"
edition = "2024"

[dependencies]
bevy = { workspace = true }
rand = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
```

**公開API（lib.rs）:**

```rust
pub mod components;
pub mod resources;
pub mod states;
pub mod constants;
pub mod types;
pub mod spatial_grid;
pub mod systems;

use bevy::prelude::*;

pub struct GameCorePlugin;

impl Plugin for GameCorePlugin {
    fn build(&self, app: &mut App) {
        app
            // 状態定義
            .init_state::<states::AppState>()
            // リソース初期化
            .insert_resource(resources::GameData::default())
            .insert_resource(resources::EnemySpawner::default())
            .insert_resource(resources::TreasureSpawner::default())
            .insert_resource(resources::SpatialGrid::default())
            .insert_resource(resources::MetaProgress::load())
            // イベント登録
            .add_event::<events::PlayerDamagedEvent>()
            .add_event::<events::EnemyDiedEvent>()
            .add_event::<events::LevelUpEvent>()
            .add_event::<events::WeaponFiredEvent>()
            .add_event::<events::TreasureOpenedEvent>()
            .add_event::<events::BossSpawnedEvent>()
            .add_event::<events::GameOverEvent>()
            .add_event::<events::VictoryEvent>()
            // システム登録（詳細はsystems/mod.rsを参照）
            .add_plugins(systems::GameSystemsPlugin);
    }
}
```

**内部モジュール構成:**

```
vs-core
├── components.rs     コンポーネント定義
│   └── Player, Enemy, Projectile, OrbitWeapon, AuraWeapon,
│       ExperienceGem, GoldCoin, Treasure, CircleCollider,
│       WeaponInventory, PassiveInventory, PlayerStats,
│       EnemyAI, DamageFlash, InvincibilityTimer, etc.
│
├── resources.rs      リソース定義
│   └── GameData, EnemySpawner, TreasureSpawner, LevelUpChoices,
│       MetaProgress, SelectedCharacter, SpatialGrid, etc.
│
├── states.rs         ゲーム状態
│   └── AppState { Title, CharacterSelect, Playing, LevelUp,
│                  Paused, GameOver, Victory, MetaShop }
│
├── types.rs          型定義
│   └── WeaponType, EnemyType, PassiveItemType, CharacterType,
│       MetaUpgradeType, TreasureContent, UpgradeChoice,
│       AIType, BossPhase, WhipSide, WeaponState, PassiveState
│
├── constants.rs      ゲーム定数
│   └── PLAYER_BASE_HP, BOSS_SPAWN_TIME, MAX_WEAPONS, etc.
│
├── spatial_grid.rs   空間パーティショニング
│
└── systems/          ゲームシステム群
    ├── mod.rs        GameSystemsPlugin（全システムの登録）
    ├── player.rs     移動・入力処理
    ├── enemy.rs      スポーン・AI・移動
    ├── boss.rs       ボスAI・フェーズ
    ├── weapons.rs    武器クールダウン・発射
    ├── projectile.rs 投射体移動・寿命
    ├── collision.rs  衝突判定
    ├── damage.rs     ダメージ適用
    ├── xp.rs         XP・吸収
    ├── level_up.rs   レベルアップ・選択肢
    ├── passive.rs    パッシブ効果計算
    ├── evolution.rs  武器進化
    ├── treasure.rs   宝箱
    ├── meta.rs       メタ進行
    ├── gold.rs       ゴールド
    ├── map.rs        背景マップ
    └── effects.rs    ビジュアルエフェクト
```

---

### 2.2 vs-ui（UI・カメラ管理）

**責務**: 直交投影カメラのセットアップと、全UI画面・HUDの実装。`vs-core` のコンポーネント・状態に依存するため、独立クレートとして分離。

```toml
# app/ui/Cargo.toml
[package]
name = "vs-ui"
version = "0.1.0"
edition = "2024"

[dependencies]
bevy = { workspace = true }
vs-core = { path = "../core" }
```

**公開API:**

```rust
// app/ui/src/lib.rs
use bevy::prelude::*;
use vs_core::states::AppState;

pub mod camera;

pub struct GameUIPlugin;

impl Plugin for GameUIPlugin {
    fn build(&self, app: &mut App) {
        app
            // カメラはStartupで常駐（タイトル・メニューでも使用）
            .add_systems(Startup, camera::setup_camera)
            // プレイヤー追従はPlaying状態でのみ実行
            .add_systems(
                Update,
                camera::camera_follow_player.run_if(in_state(AppState::Playing)),
            );
    }
}
```

**内部モジュール構成:**

```
vs-ui
├── camera.rs           カメラセットアップ・プレイヤー追従
├── hud.rs              ゲームプレイHUD（HP・XP・タイマー・武器アイコン）
├── title.rs            タイトル画面
├── character_select.rs キャラクター選択画面
├── level_up.rs         レベルアップ選択UI（カード形式3択）
├── pause.rs            ポーズ画面
├── game_over.rs        ゲームオーバー画面
├── victory.rs          勝利画面
└── meta_shop.rs        ゴールドショップUI
```

**vs-coreとの連携:**
- `vs_core::components::Player` を参照してカメラ追従を実装
- `vs_core::states::AppState` を参照してUI画面を切り替え
- `vs_core::resources::GameData` を参照してHUDを更新

---

### 2.3 vs-audio（オーディオ管理）

**責務**: BGMのシーン別切り替えと、ゲームイベントに応じたSFX再生を管理する。

```toml
# app/audio/Cargo.toml
[package]
name = "vs-audio"
version = "0.1.0"
edition = "2024"

[dependencies]
bevy = { workspace = true }
bevy_kira_audio = { workspace = true }
vs-core = { path = "../core" }
```

**公開API:**

```rust
// app/audio/src/lib.rs
pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(AudioPlugin)
            .add_systems(Startup, load_audio_assets)
            .add_systems(Update, (
                manage_bgm,
                play_sfx_on_events,
                play_gem_pickup_sfx,
            ).run_if(resource_exists::<AudioHandles>))
    }
}
```

**vs-coreとの連携:**
- `vs-core` の `AppState` を参照してBGMを切り替える
- `vs-core` のイベント（`EnemyDiedEvent`, `LevelUpEvent`, 等）を受け取ってSFXを再生する

---

### 2.4 vs-assets（アセット管理）

**責務**: ゲームで使用するアセット（スプライト・フォント）のロードと管理。オーディオアセットは vs-audio が直接ロードするため、ここではスプライトとフォントのみ扱う。

```toml
# app/assets/Cargo.toml
[package]
name = "vs-assets"
version = "0.1.0"
edition = "2024"

[dependencies]
bevy = { workspace = true }
```

**公開API:**

```rust
// app/assets/src/lib.rs
pub struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, load_sprite_assets)
            .add_systems(Startup, load_font_assets);
    }
}

// アセットハンドル（リソースとして共有）
#[derive(Resource)]
pub struct SpriteAssets {
    pub player_default: Handle<Image>,
    pub player_magician: Handle<Image>,
    pub player_thief: Handle<Image>,
    pub player_knight: Handle<Image>,
    // 敵スプライト
    pub enemy_bat: Handle<Image>,
    pub enemy_skeleton: Handle<Image>,
    pub enemy_zombie: Handle<Image>,
    pub enemy_ghost: Handle<Image>,
    pub enemy_demon: Handle<Image>,
    pub enemy_medusa: Handle<Image>,
    pub enemy_dragon: Handle<Image>,
    pub enemy_boss_death: Handle<Image>,
    // アイテム
    pub xp_gem: Handle<Image>,
    pub gold_coin: Handle<Image>,
    pub treasure: Handle<Image>,
    // 武器エフェクト
    pub proj_wand: Handle<Image>,
    pub proj_knife: Handle<Image>,
    // ... etc.
    pub fallback: Handle<Image>,     // アセット未ロード時のフォールバック
}

#[derive(Resource)]
pub struct FontAssets {
    pub pixel_font: Handle<Font>,
}
```

---

### 2.5 vs（メインバイナリ）

**責務**: Bevyアプリの初期化と全プラグインの統合のみ。ゲームロジックはここに書かない。

```toml
# app/vampire-survivors/Cargo.toml
[package]
name = "vs"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "vs"
path = "src/main.rs"

[dependencies]
bevy = { workspace = true }
vs-core   = { path = "../core" }
vs-ui     = { path = "../ui" }
vs-audio  = { path = "../audio" }
vs-assets = { path = "../assets" }
```

```rust
// app/vampire-survivors/src/main.rs
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
        // アセットを最初にロード（他プラグインが参照するため）
        .add_plugins(GameAssetsPlugin)
        // コアゲームロジック（ECS、システム）
        .add_plugins(GameCorePlugin)
        // UI・カメラ（GameCorePluginのAppState + Playerに依存）
        .add_plugins(GameUIPlugin)
        // オーディオ（coreのイベントを受け取るため後で追加）
        .add_plugins(GameAudioPlugin)
        .run();
}
```

---

## 3. クレート間のインポート例

### 3.1 vs-ui から vs-core を参照

```rust
// app/ui/src/camera.rs
use bevy::prelude::*;
use vs_core::components::Player;
use vs_core::constants::CAMERA_LERP_SPEED;

pub fn camera_follow_player(
    time: Res<Time>,
    player_q: Query<&Transform, With<Player>>,
    mut camera_q: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    let Ok(player_tf) = player_q.single() else { return; };
    let Ok(mut camera_tf) = camera_q.single_mut() else { return; };
    let target = player_tf.translation.truncate();
    let current = camera_tf.translation.truncate();
    let lerped = current.lerp(target, CAMERA_LERP_SPEED * time.delta_secs());
    camera_tf.translation.x = lerped.x;
    camera_tf.translation.y = lerped.y;
}
```

### 3.2 vs-audio から vs-core を参照

```rust
// app/audio/src/bgm.rs
use bevy::prelude::*;
use vs_core::states::AppState;
use vs_core::resources::GameData;

pub fn manage_bgm(
    state: Res<State<AppState>>,
    game_data: Res<GameData>,
    audio_handles: Res<crate::AudioHandles>,
    audio: Res<Audio>,
    mut current_bgm: Local<Option<Handle<AudioSource>>>,
) {
    // AppStateに応じてBGMを切り替え
}
```

### 3.3 vs-audio でイベントを受け取る

```rust
// app/audio/src/sfx.rs
use bevy::prelude::*;
use vs_core::events::{EnemyDiedEvent, LevelUpEvent, WeaponFiredEvent};

pub fn play_sfx_on_events(
    mut level_up_events: EventReader<LevelUpEvent>,
    mut enemy_death_events: EventReader<EnemyDiedEvent>,
    audio: Res<Audio>,
    handles: Res<crate::AudioHandles>,
) {
    // イベントに応じてSFXを再生
}
```

---

## 4. ワークスペース Cargo.toml

```toml
[workspace]
resolver = "2"
members = [
    "app/core",
    "app/ui",
    "app/audio",
    "app/assets",
    "app/vampire-survivors",
]

[workspace.dependencies]
bevy = { version = "0.17.3", features = ["file_watcher"] }
bevy_kira_audio = { version = "0.24.0", features = ["wav"] }
rand = "0.10.0"
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.149"

[profile.dev]
opt-level = 1

[profile.dev.package."*"]
opt-level = 3

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

---

## 5. 開発上の注意点

### 5.1 クレート境界での型の扱い
- `vs-core` で定義した型（`WeaponType`, `EnemyType` 等）は `pub use` で再エクスポートすること
- `vs-audio` が `vs-core` の型に依存する場合は `pub use vs_core::types::*;` で参照

### 5.2 循環依存の防止
- `vs-core` → 他のゲームクレートへの依存は禁止（最も基本的なクレート）
- 依存方向:
  - `vs` → `vs-core`, `vs-ui`, `vs-audio`, `vs-assets`
  - `vs-ui` → `vs-core`（Player コンポーネント・AppState を参照）
  - `vs-audio` → `vs-core`（イベント・状態を参照）
  - `vs-assets` → （他ゲームクレートへの依存なし）

### 5.3 プラグイン追加順序
アセットが他プラグインより先にロードされるよう、`GameAssetsPlugin` を最初に追加する：
1. `GameAssetsPlugin`（アセットロード）
2. `GameCorePlugin`（コアゲームロジック・ECS）
3. `GameUIPlugin`（UI・カメラ、CoreのAppState + Playerに依存）
4. `GameAudioPlugin`（オーディオ、Coreのイベントを受け取る）
