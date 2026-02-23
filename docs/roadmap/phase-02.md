# Phase 2: コアゲームループ

## フェーズ概要

**ステータス**: 🔲 未着手
**推定工数**: 3-4時間
**依存関係**: Phase 1

### 目的
プレイヤーキャラクターの生成・WASD移動・プレイヤー追従カメラを実装し、ゲームの基盤となるループを構築する。

### スコープ
- AppState の定義（Title → Playing → GameOver）
- プレイヤーエンティティのスポーンと移動
- カメラのプレイヤー追従
- 基本的なゲームデータ（GameData）の管理
- タイトル画面の最小実装（スタートボタンのみ）

---

## タスクリスト

### タスク 2.1: コアデータ構造の定義

**優先度**: P0
**推定工数**: 1時間
**ラベル**: task, phase-2

**説明**:
ECSのコアとなるコンポーネント・リソース・状態を定義する。

**受け入れ基準**:
- [ ] `app/core/src/states.rs` に `AppState` が定義されている
- [ ] `app/core/src/components.rs` に `Player`, `PlayerStats` が定義されている
- [ ] `app/core/src/resources.rs` に `GameData`, `SelectedCharacter` が定義されている
- [ ] `app/core/src/constants.rs` にゲーム定数が定義されている
- [ ] `app/core/src/types.rs` に `WeaponType`, `CharacterType` 等が定義されている
- [ ] コンパイルが通る

**実装ガイド**:
```rust
// states.rs
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Title,
    CharacterSelect,
    Playing,
    LevelUp,
    Paused,
    GameOver,
    Victory,
    MetaShop,
}

// components.rs
#[derive(Component)]
pub struct Player;

#[derive(Component, Clone)]
pub struct PlayerStats {
    pub max_hp: f32,
    pub current_hp: f32,
    pub move_speed: f32,
    pub damage_multiplier: f32,
    pub cooldown_reduction: f32,
    pub projectile_speed_mult: f32,
    pub duration_multiplier: f32,
    pub pickup_radius: f32,
    pub area_multiplier: f32,
    pub extra_projectiles: u32,
    pub luck: f32,
    pub hp_regen: f32,
}

// resources.rs
#[derive(Resource, Default)]
pub struct GameData {
    pub elapsed_time: f32,
    pub current_level: u32,
    pub current_xp: u32,
    pub xp_to_next_level: u32,
    pub kill_count: u32,
    pub gold_earned: u32,
    pub boss_spawned: bool,
}
```

---

### タスク 2.2: プレイヤーのスポーンと移動

**優先度**: P0
**推定工数**: 1時間
**ラベル**: task, phase-2

**説明**:
Playing状態開始時にプレイヤーをスポーンし、WASDキーで移動できるようにする。

**受け入れ基準**:
- [ ] Playing状態開始時にプレイヤーエンティティがスポーンされる
- [ ] WASDキー（および矢印キー）でプレイヤーが8方向に移動する
- [ ] 移動速度が `PlayerStats.move_speed` に基づいている
- [ ] プレイヤーのプレースホルダースプライト（単色の円）が表示される

**実装ガイド**:
```rust
pub fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player,
        PlayerStats {
            max_hp: 100.0,
            current_hp: 100.0,
            move_speed: 200.0,
            damage_multiplier: 1.0,
            cooldown_reduction: 0.0,
            projectile_speed_mult: 1.0,
            duration_multiplier: 1.0,
            pickup_radius: 80.0,
            area_multiplier: 1.0,
            extra_projectiles: 0,
            luck: 1.0,
            hp_regen: 0.0,
        },
        Sprite {
            color: Color::srgb(0.2, 0.8, 1.0),
            custom_size: Some(Vec2::splat(24.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 10.0),
        CircleCollider { radius: 12.0 },
        WeaponInventory { weapons: vec![] },
        PassiveInventory { items: vec![] },
    ));
}
```

---

### タスク 2.3: カメラのプレイヤー追従

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-2

**説明**:
2Dカメラをプレイヤーにスムーズに追従させる。

**受け入れ基準**:
- [ ] Playing状態でカメラがプレイヤーを追従する
- [ ] 追従がスムーズ（線形補間）
- [ ] プレイヤーが移動してもカメラが常についてくる

---

### タスク 2.4: ゲームタイマーの実装

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-2

**説明**:
ゲーム経過時間を管理するタイマーを実装する。

**受け入れ基準**:
- [ ] Playing状態中に `GameData.elapsed_time` が増加する
- [ ] LevelUp・Paused状態ではタイマーが停止する

---

### タスク 2.5: 最小限のタイトル画面

**優先度**: P1
**推定工数**: 0.5時間
**ラベル**: task, phase-2

**説明**:
タイトル画面に「スタート」ボタンだけを配置し、クリックでPlaying状態に遷移する。

**受け入れ基準**:
- [ ] Title状態でタイトルテキストとスタートボタンが表示される
- [ ] スタートボタンを押すとPlaying状態に遷移する

---

## フェーズ検証

### 検証項目
- [ ] WASDキーでプレイヤーが移動する
- [ ] カメラがプレイヤーを追従する
- [ ] タイトル画面からゲームを開始できる
- [ ] コンパイルエラーがない

## 次のフェーズ

Phase 2 完了 → 次は **Phase 3: 敵システム基本** に進む
