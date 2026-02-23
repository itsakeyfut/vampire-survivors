# Vampire Survivors クローン - 上級トピック: ホットリロード設計

## 概要

本ドキュメントでは、開発効率を大幅に向上させる**ホットリロード（アセット変更の即時反映）**の設計と実装を解説する。

対象アセット:
1. **スプライト画像** (PNG/WebP) — Bevy 標準機能
2. **音声ファイル** (OGG/WAV) — bevy_kira_audio 経由
3. **RON パラメータファイル** — カスタムアセットとして実装

ホットリロードを活用することで、**ゲームを再起動せずに**以下が可能になる:
- 武器の攻撃力・クールダウンの調整
- 敵の HP・速度のチューニング
- スポーンテーブルの変更
- スプライトの差し替え

---

## 1. Bevy ホットリロードの基本

### 1.1 ホットリロードの仕組み

Bevy の `DefaultPlugins` は、デバッグビルドで `assets/` ディレクトリのファイル変更を監視し、変更を検出すると自動的にアセットをリロードする。

```
assets/ ファイル変更
    ↓ ファイルシステム監視（notify クレート）
AssetServer が検出
    ↓
Handle<T> が差し替わる
    ↓
AssetEvent<T>::Modified が発火
    ↓
リアクティブシステムが変更を適用
```

### 1.2 デバッグ/リリース での切り替え

ホットリロードはデバッグビルドでは自動的に有効（開発コストゼロ）。リリースビルドでは無効になりバイナリサイズが削減される。

```rust
// app/vampire-survivors/src/main.rs

fn main() {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Vampire Survivors Clone".to_string(),
                    resolution: (1280.0, 720.0).into(),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                // デバッグビルド: ホットリロード有効
                // リリースビルド: 無効（デフォルト）
                #[cfg(debug_assertions)]
                watch_for_changes_override: Some(true),
                ..default()
            }),
    );

    // ... プラグイン追加
    app.run();
}
```

**注意**: `watch_for_changes_override` は Bevy 0.15.x の API。バージョンによって API が変わる場合があるため、`cargo doc -p bevy_asset` で確認すること。

---

## 2. スプライトのホットリロード

### 2.1 基本動作

`vs-assets` クレートで管理する `SpriteAssets` リソースは `Handle<Image>` を保持する。`AssetServer::load()` で取得したハンドルは、ファイル変更時に Bevy が自動的に新しいテクスチャに差し替える。

```rust
// app/assets/src/lib.rs

#[derive(Resource)]
pub struct SpriteAssets {
    // プレイヤー
    pub player_default: Handle<Image>,
    pub player_mage: Handle<Image>,
    pub player_thief: Handle<Image>,
    pub player_knight: Handle<Image>,

    // 敵
    pub enemy_bat: Handle<Image>,
    pub enemy_skeleton: Handle<Image>,
    pub enemy_zombie: Handle<Image>,
    pub enemy_ghost: Handle<Image>,
    pub enemy_demon: Handle<Image>,
    pub enemy_medusa: Handle<Image>,
    pub enemy_dragon: Handle<Image>,
    pub enemy_boss_death: Handle<Image>,

    // UI
    pub xp_gem: Handle<Image>,
    pub gold_coin: Handle<Image>,
    pub treasure_chest: Handle<Image>,

    // 武器エフェクト（プレースホルダー期間はなくてもよい）
    pub bullet_magic: Handle<Image>,
    pub bullet_fire: Handle<Image>,
}

pub fn load_sprite_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(SpriteAssets {
        player_default: asset_server.load("sprites/player/default.png"),
        player_mage:    asset_server.load("sprites/player/mage.png"),
        player_thief:   asset_server.load("sprites/player/thief.png"),
        player_knight:  asset_server.load("sprites/player/knight.png"),

        enemy_bat:       asset_server.load("sprites/enemies/bat.png"),
        enemy_skeleton:  asset_server.load("sprites/enemies/skeleton.png"),
        enemy_zombie:    asset_server.load("sprites/enemies/zombie.png"),
        enemy_ghost:     asset_server.load("sprites/enemies/ghost.png"),
        enemy_demon:     asset_server.load("sprites/enemies/demon.png"),
        enemy_medusa:    asset_server.load("sprites/enemies/medusa.png"),
        enemy_dragon:    asset_server.load("sprites/enemies/dragon.png"),
        enemy_boss_death: asset_server.load("sprites/enemies/boss_death.png"),

        xp_gem:         asset_server.load("sprites/items/xp_gem.png"),
        gold_coin:      asset_server.load("sprites/items/gold_coin.png"),
        treasure_chest: asset_server.load("sprites/items/treasure.png"),

        bullet_magic: asset_server.load("sprites/weapons/bullet_magic.png"),
        bullet_fire:  asset_server.load("sprites/weapons/bullet_fire.png"),
    });
}
```

### 2.2 スプライト変更の検知（必要な場合）

スプライトの差し替えに応じてロジックを変える必要がある場合は `AssetEvent<Image>` を使う。多くの場合は Bevy が自動で再描画するため不要。

```rust
// 例: デバッグ目的でスプライト更新をログ出力する
pub fn on_sprite_changed(
    mut events: EventReader<AssetEvent<Image>>,
    sprite_assets: Res<SpriteAssets>,
) {
    for event in events.read() {
        if let AssetEvent::Modified { id } = event {
            // どのスプライトが変更されたか判定
            if *id == sprite_assets.player_default.id() {
                info!("[HotReload] プレイヤースプライトが更新されました");
            } else if *id == sprite_assets.enemy_bat.id() {
                info!("[HotReload] コウモリスプライトが更新されました");
            }
            // 実際には Bevy が自動で再描画するため追加処理は不要
        }
    }
}
```

### 2.3 スプライト開発ワークフロー

```
1. cargo run -p vs でゲームを起動
2. Aseprite などでスプライトを編集・保存
3. ゲーム画面が即座に新しいスプライトに更新される
4. 再起動不要でスプライト確認が可能
```

---

## 3. 音声のホットリロード

### 3.1 bevy_kira_audio のホットリロード

`bevy_kira_audio` は Bevy の `AssetServer` を経由して音声ファイルをロードするため、基本的なホットリロードは動作する。ただし**再生中の音声**は切り替わらず、次回再生時に新しいファイルが使われる。

```rust
// app/audio/src/lib.rs

#[derive(Resource)]
pub struct AudioHandles {
    // BGM
    pub bgm_title:          Handle<AudioSource>,
    pub bgm_gameplay_early: Handle<AudioSource>,
    pub bgm_gameplay_late:  Handle<AudioSource>,
    pub bgm_boss:           Handle<AudioSource>,
    pub bgm_gameover:       Handle<AudioSource>,
    pub bgm_victory:        Handle<AudioSource>,

    // SFX
    pub sfx_weapon_whip:    Handle<AudioSource>,
    pub sfx_weapon_magic:   Handle<AudioSource>,
    pub sfx_weapon_knife:   Handle<AudioSource>,
    pub sfx_enemy_die:      Handle<AudioSource>,
    pub sfx_level_up:       Handle<AudioSource>,
    pub sfx_treasure:       Handle<AudioSource>,
    pub sfx_xp_gem:         Handle<AudioSource>,
    pub sfx_player_hurt:    Handle<AudioSource>,
    pub sfx_boss_appear:    Handle<AudioSource>,
    pub sfx_weapon_evolve:  Handle<AudioSource>,
    pub sfx_ui_click:       Handle<AudioSource>,
}

pub fn load_audio_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(AudioHandles {
        bgm_title:          asset_server.load("sounds/bgm/title.ogg"),
        bgm_gameplay_early: asset_server.load("sounds/bgm/gameplay_early.ogg"),
        bgm_gameplay_late:  asset_server.load("sounds/bgm/gameplay_late.ogg"),
        bgm_boss:           asset_server.load("sounds/bgm/boss.ogg"),
        bgm_gameover:       asset_server.load("sounds/bgm/gameover.ogg"),
        bgm_victory:        asset_server.load("sounds/bgm/victory.ogg"),

        sfx_weapon_whip:   asset_server.load("sounds/sfx/weapon_whip.ogg"),
        sfx_weapon_magic:  asset_server.load("sounds/sfx/weapon_magic.ogg"),
        sfx_weapon_knife:  asset_server.load("sounds/sfx/weapon_knife.ogg"),
        sfx_enemy_die:     asset_server.load("sounds/sfx/enemy_die.ogg"),
        sfx_level_up:      asset_server.load("sounds/sfx/level_up.ogg"),
        sfx_treasure:      asset_server.load("sounds/sfx/treasure.ogg"),
        sfx_xp_gem:        asset_server.load("sounds/sfx/xp_gem.ogg"),
        sfx_player_hurt:   asset_server.load("sounds/sfx/player_hurt.ogg"),
        sfx_boss_appear:   asset_server.load("sounds/sfx/boss_appear.ogg"),
        sfx_weapon_evolve: asset_server.load("sounds/sfx/weapon_evolve.ogg"),
        sfx_ui_click:      asset_server.load("sounds/sfx/ui_click.ogg"),
    });
}
```

### 3.2 音声ホットリロードの制限事項

| 状況 | 動作 |
|------|------|
| BGM再生中にファイル変更 | 現在の再生は変わらない。次の曲切り替えで新ファイルが使われる |
| SFX ファイル変更 | 次のSFX再生から新ファイルが使われる |
| 新しい音声ファイルを追加 | コードに `asset_server.load()` を追加する必要あり（ホットリロード非対応） |

### 3.3 音声変更の検知

BGM を即座に切り替えたい場合は `AssetEvent<AudioSource>` を使う。

```rust
pub fn on_bgm_changed(
    mut events: EventReader<AssetEvent<AudioSource>>,
    audio: Res<Audio>,
    handles: Res<AudioHandles>,
    current_bgm: Res<CurrentBgm>,
) {
    for event in events.read() {
        if let AssetEvent::Modified { id } = event {
            // 再生中のBGMが変更された場合に再起動
            if let Some(current_handle) = &current_bgm.handle {
                if *id == current_handle.id() {
                    audio.stop();
                    audio.play(current_handle.clone()).looped();
                    info!("[HotReload] BGMが更新され、再起動しました");
                }
            }
        }
    }
}

#[derive(Resource, Default)]
pub struct CurrentBgm {
    pub handle: Option<Handle<AudioSource>>,
    pub key: String,
}
```

---

## 4. RON パラメータファイルの設計

### 4.1 RON とは

RON（Rusty Object Notation）は Rust の構造体を表現するためのシリアライズフォーマット。JSON より Rust 親和性が高く、コメントや末尾カンマが使えるため設定ファイルに適している。

```ron
// weapon_params.ron の例
WeaponParamsAsset(
    weapons: {
        "Whip": WeaponParams(
            damage: 20.0,
            cooldown: 1.5,
            projectile_count: 1,
            level_bonuses: [
                LevelBonus(damage_add: 5.0),
                LevelBonus(area_mult: 1.1),
                // ...
            ],
        ),
        // ...
    },
)
```

### 4.2 RON ファイル構成

```
assets/
└── params/
    ├── weapon_params.ron   # 武器パラメータ（ダメージ・クールダウン等）
    ├── enemy_params.ron    # 敵パラメータ（HP・速度・XP報酬等）
    ├── spawn_table.ron     # 敵スポーンテーブル（時刻→出現敵種別）
    └── balance.ron         # バランス調整値（XP曲線・倍率等）
```

---

## 5. カスタム RON アセットの実装

### 5.1 Cargo.toml の設定

`vs-assets` クレートに必要な依存関係を追加する。

```toml
# app/assets/Cargo.toml
[package]
name = "vs-assets"
version = "0.1.0"
edition = "2024"

[dependencies]
bevy = { workspace = true }
serde = { version = "1", features = ["derive"] }
ron = "0.12.0"
```

### 5.2 武器パラメータアセットの定義

```rust
// app/assets/src/params/weapon_params.rs

use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

/// 武器の1レベルごとのボーナス
#[derive(Debug, Clone, Deserialize)]
pub struct LevelBonus {
    /// ダメージ加算（省略可）
    #[serde(default)]
    pub damage_add: f32,
    /// ダメージ乗算（省略可、1.0 = 変化なし）
    #[serde(default = "default_one")]
    pub damage_mult: f32,
    /// クールダウン乗算（省略可）
    #[serde(default = "default_one")]
    pub cooldown_mult: f32,
    /// 発射数加算（省略可）
    #[serde(default)]
    pub projectile_count_add: u32,
    /// 範囲乗算（省略可）
    #[serde(default = "default_one")]
    pub area_mult: f32,
    /// 持続時間乗算（省略可）
    #[serde(default = "default_one")]
    pub duration_mult: f32,
}

fn default_one() -> f32 { 1.0 }

/// 武器の基本パラメータ（Lv1時点）
#[derive(Debug, Clone, Deserialize)]
pub struct WeaponParams {
    /// 基本ダメージ
    pub damage: f32,
    /// 攻撃クールダウン（秒）
    pub cooldown: f32,
    /// 発射数
    #[serde(default = "default_u32_one")]
    pub projectile_count: u32,
    /// 投射体速度
    #[serde(default = "default_150")]
    pub projectile_speed: f32,
    /// 投射体寿命（秒）
    #[serde(default = "default_two")]
    pub projectile_lifetime: f32,
    /// 最大レベル（通常8）
    #[serde(default = "default_eight")]
    pub max_level: u32,
    /// レベルアップ時のボーナス（Lv2〜Lv8 の7個分）
    pub level_bonuses: Vec<LevelBonus>,
}

fn default_u32_one() -> u32 { 1 }
fn default_150() -> f32 { 150.0 }
fn default_two() -> f32 { 2.0 }
fn default_eight() -> u32 { 8 }

/// 全武器パラメータをまとめたアセット
#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct WeaponParamsAsset {
    pub weapons: HashMap<String, WeaponParams>,
}
```

### 5.3 敵パラメータアセットの定義

```rust
// app/assets/src/params/enemy_params.rs

use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

/// 敵の基本パラメータ
#[derive(Debug, Clone, Deserialize)]
pub struct EnemyParams {
    /// 基本 HP
    pub base_hp: f32,
    /// 移動速度（px/s）
    pub move_speed: f32,
    /// プレイヤーへの接触ダメージ
    pub contact_damage: f32,
    /// 撃破時の XP 報酬
    pub xp_reward: u32,
    /// ゴールドドロップ確率（0.0〜1.0）
    pub gold_drop_chance: f32,
    /// 衝突半径（px）
    pub collision_radius: f32,
    /// 出現開始時間（分）
    pub spawn_start_minute: f32,
    /// 難易度スケーリング係数（HP・速度に乗算）
    #[serde(default = "default_one")]
    pub scale_per_minute: f32,
}

fn default_one() -> f32 { 1.0 }

/// 全敵パラメータをまとめたアセット
#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct EnemyParamsAsset {
    pub enemies: HashMap<String, EnemyParams>,
}
```

### 5.4 スポーンテーブルアセットの定義

```rust
// app/assets/src/params/spawn_table.rs

use bevy::prelude::*;
use serde::Deserialize;

/// 特定の時刻に出現する敵の設定
#[derive(Debug, Clone, Deserialize)]
pub struct SpawnEntry {
    /// 出現開始時間（分）
    pub minute: f32,
    /// 出現させる敵の種類名（EnemyParamsAsset のキーと一致させる）
    pub enemy_type: String,
    /// 最大同時出現数
    pub max_count: u32,
    /// スポーン間隔（秒）
    pub interval_secs: f32,
}

/// バランス調整値
#[derive(Debug, Clone, Deserialize)]
pub struct BalanceParams {
    /// 時間経過による敵 HP 倍率（1分ごとに加算）
    pub hp_scale_per_minute: f32,
    /// 時間経過による敵速度倍率
    pub speed_scale_per_minute: f32,
    /// XP 必要量の成長率
    pub xp_growth_rate: f32,
    /// XP ジェム吸収範囲（px）
    pub xp_pickup_radius: f32,
    /// 宝箱スポーン間隔（分）
    pub treasure_interval_minutes: f32,
}

/// スポーンテーブルアセット
#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct SpawnTableAsset {
    pub entries: Vec<SpawnEntry>,
    pub balance: BalanceParams,
}
```

### 5.5 カスタム AssetLoader の実装

Bevy にRONファイルを読み込む方法を教えるカスタムローダーを実装する。

```rust
// app/assets/src/params/loader.rs

use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
};
use serde::de::DeserializeOwned;
use std::marker::PhantomData;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RonLoaderError {
    #[error("IOエラー: {0}")]
    Io(#[from] std::io::Error),
    #[error("RONパースエラー: {0}")]
    Ron(#[from] ron::error::SpannedError),
}

/// 汎用 RON アセットローダー
pub struct RonLoader<T> {
    _phantom: PhantomData<T>,
}

impl<T> Default for RonLoader<T> {
    fn default() -> Self {
        Self { _phantom: PhantomData }
    }
}

impl<T> AssetLoader for RonLoader<T>
where
    T: Asset + for<'de> serde::Deserialize<'de> + Send + Sync + 'static,
{
    type Asset = T;
    type Settings = ();
    type Error = RonLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let asset = ron::de::from_bytes::<T>(&bytes)?;
        Ok(asset)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}
```

### 5.6 プラグインへの登録

```rust
// app/assets/src/lib.rs

mod params;
pub use params::{
    weapon_params::{WeaponParamsAsset, WeaponParams, LevelBonus},
    enemy_params::{EnemyParamsAsset, EnemyParams},
    spawn_table::{SpawnTableAsset, SpawnEntry, BalanceParams},
    loader::RonLoader,
};

/// RON パラメータのハンドル管理リソース
#[derive(Resource)]
pub struct ParamHandles {
    pub weapon_params: Handle<WeaponParamsAsset>,
    pub enemy_params:  Handle<EnemyParamsAsset>,
    pub spawn_table:   Handle<SpawnTableAsset>,
}

pub struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        // カスタムローダーを登録
        app.init_asset::<WeaponParamsAsset>()
           .init_asset::<EnemyParamsAsset>()
           .init_asset::<SpawnTableAsset>()
           .register_asset_loader(RonLoader::<WeaponParamsAsset>::default())
           .register_asset_loader(RonLoader::<EnemyParamsAsset>::default())
           .register_asset_loader(RonLoader::<SpawnTableAsset>::default());

        // Startup でアセットをロード
        app.add_systems(Startup, (load_sprite_assets, load_audio_assets, load_param_assets));
    }
}

fn load_param_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(ParamHandles {
        weapon_params: asset_server.load("params/weapon_params.ron"),
        enemy_params:  asset_server.load("params/enemy_params.ron"),
        spawn_table:   asset_server.load("params/spawn_table.ron"),
    });
}
```

---

## 6. RON パラメータのリアクティブ適用

### 6.1 武器パラメータの変更検知と適用

RONファイルが変更された時、ゲーム中の武器エンティティに新しいパラメータを即座に反映する。

```rust
// app/core/src/weapons/hot_reload.rs

use bevy::prelude::*;
use vs_assets::{ParamHandles, WeaponParamsAsset};
use crate::weapons::{WeaponInstance, WeaponType};

/// 武器パラメータのホットリロードシステム
pub fn on_weapon_params_changed(
    mut events: EventReader<AssetEvent<WeaponParamsAsset>>,
    param_handles: Res<ParamHandles>,
    weapon_params: Res<Assets<WeaponParamsAsset>>,
    mut weapons: Query<(&WeaponType, &mut WeaponInstance)>,
) {
    for event in events.read() {
        let AssetEvent::Modified { id } = event else { continue };

        // 変更されたのが weapon_params.ron か確認
        if *id != param_handles.weapon_params.id() { continue; }

        let Some(params) = weapon_params.get(&param_handles.weapon_params) else { continue };

        // 全武器エンティティに新しいパラメータを適用
        for (weapon_type, mut weapon) in weapons.iter_mut() {
            let key = weapon_type.as_param_key();
            if let Some(wp) = params.weapons.get(key) {
                // 現在のレベルに基づいて実効値を再計算
                weapon.apply_params(wp);
                info!(
                    "[HotReload] 武器 {:?} のパラメータを更新: damage={}, cooldown={}",
                    weapon_type, weapon.damage, weapon.cooldown
                );
            }
        }
    }
}

impl WeaponType {
    pub fn as_param_key(&self) -> &'static str {
        match self {
            WeaponType::Whip       => "Whip",
            WeaponType::MagicWand  => "MagicWand",
            WeaponType::Knife      => "Knife",
            WeaponType::Garlic     => "Garlic",
            WeaponType::Bible      => "Bible",
            WeaponType::ThunderRing => "ThunderRing",
            WeaponType::Cross      => "Cross",
            WeaponType::FireWand   => "FireWand",
        }
    }
}

impl WeaponInstance {
    /// RON パラメータを現在レベルに適用して実効値を再計算
    pub fn apply_params(&mut self, params: &vs_assets::WeaponParams) {
        // Lv1 の基本値
        let mut damage   = params.damage;
        let mut cooldown = params.cooldown;
        let mut count    = params.projectile_count;

        // Lv2 以上: ボーナスを累積適用
        let bonuses_to_apply = (self.level as usize).saturating_sub(1);
        for bonus in params.level_bonuses.iter().take(bonuses_to_apply) {
            damage    = damage * bonus.damage_mult + bonus.damage_add;
            cooldown  *= bonus.cooldown_mult;
            count     += bonus.projectile_count_add;
        }

        self.damage          = damage;
        self.cooldown        = cooldown;
        self.projectile_count = count;
    }
}
```

### 6.2 敵パラメータの変更検知と適用

```rust
// app/core/src/enemies/hot_reload.rs

use bevy::prelude::*;
use vs_assets::{ParamHandles, EnemyParamsAsset};
use crate::enemies::{Enemy, EnemyType};

/// 敵パラメータのホットリロードシステム
pub fn on_enemy_params_changed(
    mut events: EventReader<AssetEvent<EnemyParamsAsset>>,
    param_handles: Res<ParamHandles>,
    enemy_params: Res<Assets<EnemyParamsAsset>>,
    game_data: Res<crate::resources::GameData>,
    mut enemies: Query<(&EnemyType, &mut Enemy)>,
) {
    for event in events.read() {
        let AssetEvent::Modified { id } = event else { continue };
        if *id != param_handles.enemy_params.id() { continue; }

        let Some(params) = enemy_params.get(&param_handles.enemy_params) else { continue };

        let elapsed_minutes = game_data.elapsed_time / 60.0;

        for (enemy_type, mut enemy) in enemies.iter_mut() {
            let key = enemy_type.as_param_key();
            if let Some(ep) = params.enemies.get(key) {
                // 現在の経過時間に応じてスケールを再計算
                let scale = 1.0 + ep.scale_per_minute * elapsed_minutes;
                enemy.max_hp       = ep.base_hp * scale;
                enemy.move_speed   = ep.move_speed * scale;
                enemy.contact_damage = ep.contact_damage;
                info!(
                    "[HotReload] 敵 {:?} のパラメータを更新: hp={:.1}, speed={:.1}",
                    enemy_type, enemy.max_hp, enemy.move_speed
                );
            }
        }
    }
}
```

### 6.3 スポーンテーブルの変更検知

スポーンテーブルが変更された場合、`EnemySpawner` リソースを即座に更新する。

```rust
// app/core/src/spawning/hot_reload.rs

use bevy::prelude::*;
use vs_assets::{ParamHandles, SpawnTableAsset};
use crate::resources::EnemySpawner;

pub fn on_spawn_table_changed(
    mut events: EventReader<AssetEvent<SpawnTableAsset>>,
    param_handles: Res<ParamHandles>,
    spawn_table: Res<Assets<SpawnTableAsset>>,
    mut spawner: ResMut<EnemySpawner>,
    game_data: Res<crate::resources::GameData>,
) {
    for event in events.read() {
        let AssetEvent::Modified { id } = event else { continue };
        if *id != param_handles.spawn_table.id() { continue; }

        let Some(table) = spawn_table.get(&param_handles.spawn_table) else { continue };

        // バランス値を更新
        let balance = &table.balance;
        spawner.hp_scale_factor    = balance.hp_scale_per_minute;
        spawner.speed_scale_factor = balance.speed_scale_per_minute;

        info!("[HotReload] スポーンテーブルを更新しました");
    }
}
```

### 6.4 システムの登録

```rust
// app/core/src/lib.rs または weapons/mod.rs

use crate::weapons::hot_reload::on_weapon_params_changed;
use crate::enemies::hot_reload::on_enemy_params_changed;
use crate::spawning::hot_reload::on_spawn_table_changed;

impl Plugin for GameCorePlugin {
    fn build(&self, app: &mut App) {
        // ... 既存のシステム登録 ...

        // ホットリロードシステム（Playing 状態でのみ動作）
        app.add_systems(
            Update,
            (
                on_weapon_params_changed,
                on_enemy_params_changed,
                on_spawn_table_changed,
            )
            .run_if(in_state(AppState::Playing)),
        );
    }
}
```

---

## 7. RON ファイルのサンプル

### 7.1 weapon_params.ron

```ron
// assets/params/weapon_params.ron
WeaponParamsAsset(
    weapons: {
        "Whip": WeaponParams(
            damage: 20.0,
            cooldown: 1.5,
            projectile_count: 1,
            projectile_speed: 0.0,    // ムチは投射体なし
            projectile_lifetime: 0.3,
            max_level: 8,
            level_bonuses: [
                LevelBonus(damage_add: 5.0),                   // Lv2
                LevelBonus(damage_add: 5.0, area_mult: 1.1),   // Lv3
                LevelBonus(damage_add: 5.0),                   // Lv4
                LevelBonus(damage_add: 5.0, cooldown_mult: 0.9), // Lv5
                LevelBonus(damage_add: 10.0),                  // Lv6
                LevelBonus(projectile_count_add: 1),           // Lv7
                LevelBonus(damage_add: 10.0, area_mult: 1.2),  // Lv8
            ],
        ),
        "MagicWand": WeaponParams(
            damage: 15.0,
            cooldown: 0.6,
            projectile_count: 1,
            projectile_speed: 200.0,
            projectile_lifetime: 2.0,
            max_level: 8,
            level_bonuses: [
                LevelBonus(damage_add: 5.0),
                LevelBonus(projectile_count_add: 1),
                LevelBonus(damage_add: 5.0, cooldown_mult: 0.9),
                LevelBonus(damage_add: 5.0),
                LevelBonus(projectile_count_add: 1),
                LevelBonus(damage_add: 10.0, cooldown_mult: 0.85),
                LevelBonus(damage_add: 10.0, projectile_count_add: 1),
            ],
        ),
        "Knife": WeaponParams(
            damage: 10.0,
            cooldown: 0.4,
            projectile_count: 1,
            projectile_speed: 400.0,
            projectile_lifetime: 0.8,
            max_level: 8,
            level_bonuses: [
                LevelBonus(damage_add: 3.0),
                LevelBonus(projectile_count_add: 1),
                LevelBonus(damage_add: 3.0, cooldown_mult: 0.9),
                LevelBonus(damage_add: 3.0, projectile_count_add: 1),
                LevelBonus(damage_add: 5.0),
                LevelBonus(projectile_count_add: 2),
                LevelBonus(damage_add: 5.0, cooldown_mult: 0.85),
            ],
        ),
        // ... 他の武器
    },
)
```

### 7.2 enemy_params.ron

```ron
// assets/params/enemy_params.ron
EnemyParamsAsset(
    enemies: {
        "Bat": EnemyParams(
            base_hp: 10.0,
            move_speed: 80.0,
            contact_damage: 5.0,
            xp_reward: 1,
            gold_drop_chance: 0.05,
            collision_radius: 8.0,
            spawn_start_minute: 0.0,
            scale_per_minute: 0.05,    // 1分ごとに5%強化
        ),
        "Skeleton": EnemyParams(
            base_hp: 20.0,
            move_speed: 55.0,
            contact_damage: 8.0,
            xp_reward: 2,
            gold_drop_chance: 0.08,
            collision_radius: 12.0,
            spawn_start_minute: 0.0,
            scale_per_minute: 0.05,
        ),
        "Zombie": EnemyParams(
            base_hp: 50.0,
            move_speed: 35.0,
            contact_damage: 12.0,
            xp_reward: 4,
            gold_drop_chance: 0.1,
            collision_radius: 14.0,
            spawn_start_minute: 5.0,
            scale_per_minute: 0.06,
        ),
        "BossDeath": EnemyParams(
            base_hp: 50000.0,
            move_speed: 40.0,
            contact_damage: 1000.0,    // 即死級
            xp_reward: 0,              // 勝利条件なので XP 不要
            gold_drop_chance: 0.0,
            collision_radius: 40.0,
            spawn_start_minute: 30.0,
            scale_per_minute: 0.0,     // ボスはスケールしない
        ),
        // ... 他の敵
    },
)
```

### 7.3 spawn_table.ron

```ron
// assets/params/spawn_table.ron
SpawnTableAsset(
    entries: [
        SpawnEntry(minute: 0.0,  enemy_type: "Bat",      max_count: 50,  interval_secs: 2.0),
        SpawnEntry(minute: 0.0,  enemy_type: "Skeleton", max_count: 30,  interval_secs: 3.0),
        SpawnEntry(minute: 5.0,  enemy_type: "Zombie",   max_count: 20,  interval_secs: 4.0),
        SpawnEntry(minute: 10.0, enemy_type: "Ghost",    max_count: 15,  interval_secs: 5.0),
        SpawnEntry(minute: 15.0, enemy_type: "Demon",    max_count: 10,  interval_secs: 6.0),
        SpawnEntry(minute: 20.0, enemy_type: "Medusa",   max_count: 8,   interval_secs: 8.0),
        SpawnEntry(minute: 25.0, enemy_type: "Dragon",   max_count: 5,   interval_secs: 10.0),
        SpawnEntry(minute: 30.0, enemy_type: "BossDeath", max_count: 1,  interval_secs: 999.0),
    ],
    balance: BalanceParams(
        hp_scale_per_minute: 0.05,
        speed_scale_per_minute: 0.02,
        xp_growth_rate: 1.15,
        xp_pickup_radius: 80.0,
        treasure_interval_minutes: 2.0,
    ),
)
```

---

## 8. アセットロード状態のトラッキング

### 8.1 起動時のロード待機

RON ファイルが全てロードされてからゲームを開始するため、ロード状態を監視する。

```rust
// app/core/src/loading.rs

use bevy::prelude::*;
use vs_assets::ParamHandles;

/// アセットロードが完了するまで Loading 状態に留まる
pub fn check_params_loaded(
    param_handles: Res<ParamHandles>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    use bevy::asset::LoadState;

    let all_loaded = [
        asset_server.get_load_state(param_handles.weapon_params.id()),
        asset_server.get_load_state(param_handles.enemy_params.id()),
        asset_server.get_load_state(param_handles.spawn_table.id()),
    ]
    .iter()
    .all(|state| matches!(state, Some(LoadState::Loaded)));

    if all_loaded {
        info!("[Assets] 全パラメータファイルのロード完了");
        next_state.set(AppState::Title);
    }
}

/// ロードに失敗した場合の処理
pub fn check_params_failed(
    param_handles: Res<ParamHandles>,
    asset_server: Res<AssetServer>,
) {
    use bevy::asset::LoadState;

    let handles = [
        ("weapon_params.ron", param_handles.weapon_params.id()),
        ("enemy_params.ron",  param_handles.enemy_params.id()),
        ("spawn_table.ron",   param_handles.spawn_table.id()),
    ];

    for (name, id) in handles {
        if matches!(asset_server.get_load_state(id), Some(LoadState::Failed(_))) {
            error!("[Assets] {} のロードに失敗しました。デフォルト値で続行します", name);
            // フォールバック処理（必要なら）
        }
    }
}
```

### 8.2 AppState への Loading 状態追加

```rust
// app/core/src/states.rs

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Loading,        // アセットロード中（新規追加）
    Title,
    CharacterSelect,
    Playing,
    LevelUp,
    Paused,
    GameOver,
    Victory,
    MetaShop,
}

// Plugin 登録
app.add_systems(
    Update,
    (check_params_loaded, check_params_failed)
        .run_if(in_state(AppState::Loading)),
);
```

---

## 9. 開発ワークフロー

### 9.1 パラメータ調整の手順

```
1. cargo run -p vs でゲームを起動（デバッグビルド）
2. ゲームを Playing 状態にする
3. テキストエディタで assets/params/weapon_params.ron を開く
4. Whip の damage を変更して保存
   → 即座にゲーム内の武器ダメージが変わる
5. 敵に当てて効果を確認
6. 満足のいくバランスになったら完了
```

### 9.2 スプライト制作ワークフロー（Phase 17 向け）

```
1. cargo run -p vs でゲームを起動
2. プレースホルダー（単色円）の状態でゲームプレイ確認
3. Aseprite で assets/sprites/enemies/bat.png を制作・保存
   → ゲーム内でコウモリのスプライトが即座に更新される
4. サイズ・位置を目視確認
5. 問題なければ次のスプライトへ
```

### 9.3 デバッグコマンドの追加（オプション）

ホットリロードと組み合わせて使えるデバッグコマンドを実装すると、さらに効率が上がる。

```rust
// app/core/src/debug.rs
// デバッグビルドのみコンパイル
#[cfg(debug_assertions)]
pub fn debug_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_data: ResMut<GameData>,
    mut player_q: Query<&mut crate::components::PlayerStats>,
) {
    // F1: プレイヤー HP を全回復
    if keyboard.just_pressed(KeyCode::F1) {
        if let Ok(mut stats) = player_q.get_single_mut() {
            stats.hp = stats.max_hp;
            info!("[Debug] HP全回復");
        }
    }

    // F2: 経過時間を5分進める
    if keyboard.just_pressed(KeyCode::F2) {
        game_data.elapsed_time += 5.0 * 60.0;
        info!("[Debug] 経過時間 +5分 → {:.1}分", game_data.elapsed_time / 60.0);
    }

    // F3: レベルアップ強制発動
    if keyboard.just_pressed(KeyCode::F3) {
        game_data.xp = game_data.xp_to_next_level;
        info!("[Debug] レベルアップ強制");
    }

    // F4: ゴールド +1000
    if keyboard.just_pressed(KeyCode::F4) {
        game_data.gold_earned += 1000;
        info!("[Debug] ゴールド +1000");
    }
}
```

### 9.4 ビルドプロファイル設定

`Cargo.toml` にデバッグビルドの最適化設定を追加して、ホットリロード使用時も快適なフレームレートを維持する。

```toml
# Cargo.toml (workspace root)

[profile.dev]
opt-level = 1           # 依存クレートより低め（ビルド速度優先）

[profile.dev.package."*"]
opt-level = 3           # 依存クレート（bevy等）は最適化（実行速度優先）

[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
strip = true
```

---

## 10. まとめ：ホットリロード対応チェックリスト

| 対象 | 実装方法 | 変更即時反映 |
|------|---------|------------|
| スプライト (PNG) | `AssetServer::load()` + `Handle<Image>` | ✅ 自動 |
| 音声 (OGG/WAV) | `AssetServer::load()` + `Handle<AudioSource>` | ✅ 次回再生時 |
| 武器パラメータ RON | `RonLoader` + `AssetEvent<WeaponParamsAsset>` | ✅ 全武器に即適用 |
| 敵パラメータ RON | `RonLoader` + `AssetEvent<EnemyParamsAsset>` | ✅ 全敵に即適用 |
| スポーンテーブル RON | `RonLoader` + `AssetEvent<SpawnTableAsset>` | ✅ Spawner に即適用 |
| コード変更 | cargo-watch など外部ツール | ✅ 再ビルドが必要 |

### 実装優先順位

1. **P0（Phase 1〜2 で実装）**: RON ローダー基盤（`vs-assets` クレートに追加）
2. **P0（Phase 1〜2 で実装）**: `weapon_params.ron` と `enemy_params.ron` の基本版
3. **P1（Phase 7〜 で拡充）**: 全武器・全敵のRONパラメータ整備
4. **P2（Phase 16 で実装）**: `spawn_table.ron` によるスポーン管理
5. **P3（Phase 17 で実施）**: スプライトのホットリロード活用（ピクセルアート制作中）
