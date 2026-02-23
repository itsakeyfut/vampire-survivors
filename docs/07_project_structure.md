# Vampire Survivors クローン - プロジェクト構造書

## 1. ディレクトリ構造全体

```
vampire-survivors/                    # プロジェクトルート
├── Cargo.toml                        # ワークスペース定義
├── Cargo.lock
├── rust-toolchain.toml               # Rustツールチェーン固定
├── justfile                          # Just コマンドランナー
├── .gitignore
├── LICENSE
├── README.md
│
├── app/                              # アプリケーションクレート群
│   ├── core/                         # vs-core クレート
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                # GameCorePlugin のエクスポート
│   │       ├── components.rs         # 全コンポーネント定義
│   │       ├── resources.rs          # 全リソース定義
│   │       ├── states.rs             # AppState 定義
│   │       ├── constants.rs          # ゲーム定数
│   │       ├── types.rs              # WeaponType, EnemyType 等の型定義
│   │       ├── spatial_grid.rs       # 空間グリッドパーティショニング
│   │       ├── systems/
│   │       │   ├── mod.rs
│   │       │   ├── player.rs         # プレイヤー移動
│   │       │   ├── enemy.rs          # 敵スポーン・移動AI
│   │       │   ├── boss.rs           # ボスAI・フェーズ管理
│   │       │   ├── weapons.rs        # 武器システム統括
│   │       │   ├── projectile.rs     # 投射体の移動・寿命管理
│   │       │   ├── collision.rs      # 衝突判定システム
│   │       │   ├── xp.rs             # XPジェム・吸収処理
│   │       │   ├── level_up.rs       # レベルアップ・選択肢生成
│   │       │   ├── passive.rs        # パッシブ効果の計算・適用
│   │       │   ├── evolution.rs      # 武器進化ロジック
│   │       │   ├── treasure.rs       # 宝箱スポーン・開封
│   │       │   ├── meta.rs           # メタ進行データ管理
│   │       │   ├── damage.rs         # ダメージ適用システム
│   │       │   ├── camera.rs         # カメラ追従
│   │       │   ├── map.rs            # 背景マップ管理
│   │       │   ├── gold.rs           # ゴールドドロップ・獲得
│   │       │   └── effects.rs        # ビジュアルエフェクト
│   │       └── ui/
│   │           ├── mod.rs
│   │           ├── hud.rs            # ゲームプレイHUD
│   │           ├── title.rs          # タイトル画面
│   │           ├── character_select.rs  # キャラクター選択画面
│   │           ├── level_up.rs       # レベルアップ選択UI
│   │           ├── pause.rs          # ポーズ画面
│   │           ├── game_over.rs      # ゲームオーバー画面
│   │           ├── victory.rs        # 勝利画面
│   │           └── meta_shop.rs      # ゴールドショップUI
│   │
│   ├── audio/                        # vs-audio クレート
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                # GameAudioPlugin のエクスポート
│   │       ├── bgm.rs                # BGM管理・状態遷移
│   │       └── sfx.rs                # SFX再生システム
│   │
│   ├── assets/                       # vs-assets クレート
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                # GameAssetsPlugin のエクスポート
│   │       ├── sprites.rs            # スプライト定義・ロード
│   │       ├── sounds.rs             # オーディオアセット定義・ロード
│   │       └── fonts.rs              # フォント定義・ロード
│   │
│   └── vampire-survivors/            # vs（メインバイナリ）クレート
│       ├── Cargo.toml
│       └── src/
│           └── main.rs               # エントリポイント・プラグイン統合
│
├── assets/                           # ゲームアセット
│   ├── sprites/
│   │   ├── player/
│   │   │   ├── default_player.png    # デフォルトキャラ（プレースホルダー）
│   │   │   ├── magician.png
│   │   │   ├── thief.png
│   │   │   └── knight.png
│   │   ├── enemies/
│   │   │   ├── bat.png
│   │   │   ├── skeleton.png
│   │   │   ├── zombie.png
│   │   │   ├── ghost.png
│   │   │   ├── demon.png
│   │   │   ├── medusa.png
│   │   │   ├── dragon.png
│   │   │   └── boss_death.png
│   │   ├── weapons/
│   │   │   ├── projectile_wand.png
│   │   │   ├── projectile_knife.png
│   │   │   ├── projectile_cross.png
│   │   │   ├── projectile_fireball.png
│   │   │   ├── orbit_bible.png
│   │   │   └── effect_thunder.png
│   │   ├── items/
│   │   │   ├── xp_gem.png
│   │   │   ├── gold_coin.png
│   │   │   └── treasure.png
│   │   └── ui/
│   │       ├── hp_bar.png
│   │       ├── xp_bar.png
│   │       ├── weapon_icons/
│   │       │   ├── icon_whip.png
│   │       │   ├── icon_wand.png
│   │       │   └── ...
│   │       └── passive_icons/
│   │           ├── icon_spinach.png
│   │           └── ...
│   ├── sounds/
│   │   ├── bgm/
│   │   │   ├── bgm_title.ogg
│   │   │   ├── bgm_gameplay_early.ogg
│   │   │   ├── bgm_gameplay_late.ogg
│   │   │   ├── bgm_boss.ogg
│   │   │   ├── bgm_gameover.ogg
│   │   │   └── bgm_victory.ogg
│   │   └── sfx/
│   │       ├── weapons/
│   │       │   └── ...（詳細はオーディオ設計書参照）
│   │       ├── enemies/
│   │       ├── player/
│   │       ├── events/
│   │       └── ui/
│   └── fonts/
│       └── pixel_font.ttf            # ピクセルフォント
│
├── save/                             # セーブデータ（.gitignoreで除外）
│   └── meta.json                     # メタ進行データ（ゴールド・アンロック等）
│
├── docs/
│   └── vampire-survivors/
│       ├── 01_specification.md
│       ├── 02_architecture.md
│       ├── 03_gameplay_systems.md
│       ├── 04_ui_ux.md
│       ├── 05_audio.md
│       ├── 06_implementation_plan.md
│       ├── 07_project_structure.md   ← このファイル
│       ├── 08_crate_architecture.md
│       ├── 09_quick_reference.md
│       └── roadmap/
│           ├── README.md
│           ├── phase-01.md
│           └── ...（phase-17まで）
│
└── .github/
    └── ISSUE_TEMPLATE/
        ├── bug.yml
        ├── feature.yml
        └── config.yml
```

---

## 2. ファイル責務

### 2.1 app/core/src/

#### `lib.rs`
- `GameCorePlugin` の定義とエクスポート
- サブモジュールの宣言

#### `components.rs`
- ゲームに登場する全コンポーネントの定義
- `Player`, `Enemy`, `Projectile`, `ExperienceGem`, etc.

#### `resources.rs`
- グローバルリソースの定義
- `GameData`, `EnemySpawner`, `LevelUpChoices`, `MetaProgress`, etc.

#### `states.rs`
- `AppState` enumの定義
- 状態遷移ルールのコメント

#### `constants.rs`
```rust
// ゲーム定数
pub const PLAYER_BASE_HP: f32 = 100.0;
pub const PLAYER_BASE_SPEED: f32 = 200.0;
pub const PLAYER_PICKUP_RADIUS: f32 = 80.0;
pub const PLAYER_INVINCIBILITY_TIME: f32 = 0.5;

pub const BOSS_SPAWN_TIME: f32 = 30.0 * 60.0;  // 30分
pub const TREASURE_SPAWN_INTERVAL: f32 = 180.0; // 3分

pub const MAX_WEAPONS: usize = 6;
pub const MAX_WEAPON_LEVEL: u8 = 8;
pub const MAX_PASSIVE_LEVEL: u8 = 5;

pub const ENEMY_SPAWN_BASE_INTERVAL: f32 = 0.5;

// カメラ
pub const CAMERA_LERP_FACTOR: f32 = 10.0;

// 空間グリッド
pub const SPATIAL_GRID_CELL_SIZE: f32 = 64.0;
```

#### `types.rs`
```rust
// ゲーム全体で使用する型定義
pub enum WeaponType { ... }
pub enum EnemyType { ... }
pub enum PassiveItemType { ... }
pub enum CharacterType { ... }
pub enum MetaUpgradeType { ... }
pub enum TreasureContent { ... }
pub enum UpgradeChoice { ... }
pub enum AIType { ... }
pub enum BossPhase { ... }
```

### 2.2 app/vampire-survivors/src/main.rs

```rust
use bevy::prelude::*;
use vs_core::GameCorePlugin;
use vs_audio::GameAudioPlugin;
use vs_assets::GameAssetsPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Vampire Survivors Clone".into(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(GameAssetsPlugin)
        .add_plugins(GameCorePlugin)
        .add_plugins(GameAudioPlugin)
        .run();
}
```

---

## 3. モジュール間の依存関係

```
vampire-survivors (main binary)
    │
    ├─→ vs-core        # コアゲームロジック・ECS・UI
    ├─→ vs-audio ────→ vs-core  # BGM/SFX（coreのイベント・状態を参照）
    └─→ vs-assets               # アセットロード（独立）
```

- `vs-core` は他のゲームクレートに依存しない（最も基本的なクレート）
- `vs-audio` は `vs-core` のイベント・状態を参照してBGM切り替えを行う
- `vs-assets` はアセットロードのみで他クレートに依存しない
- メインバイナリが全プラグインを統合して `App::run()` する

---

## 4. セーブデータ

### 4.1 meta.json の構造

```json
{
    "total_gold": 1234,
    "unlocked_characters": ["DefaultCharacter", "Magician"],
    "purchased_upgrades": [
        "BonusHp",
        "XpBonus"
    ]
}
```

### 4.2 セーブタイミング
- ゲームオーバー時（ゴールド保存）
- 勝利時（ゴールド保存）
- ゴールドショップで購入時（即座に保存）
- ゲーム終了時（シャットダウンイベント）

---

## 5. .gitignore の内容

```gitignore
# Rustビルド成果物
/target

# セーブデータ（プライベート）
/save/

# OS固有
.DS_Store
Thumbs.db
```
