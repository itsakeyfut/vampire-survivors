# Vampire Survivors クローン - 技術アーキテクチャ設計書

## 1. 技術スタック

### 1.1 コアテクノロジー
- **ゲームエンジン**: Bevy 0.17.3
  - ECS（Entity Component System）アーキテクチャ
  - 高性能な並列システム処理
  - Rustの安全性と速度を活用
  - 2D機能セット使用

### 1.2 主要依存クレート

```toml
[dependencies]
# ゲームエンジン
bevy = { version = "0.17.3", features = ["file_watcher"] }

# オーディオ（Bevyの標準オーディオより高機能）
bevy_kira_audio = { version = "0.24.0", features = ["wav"] }

# ユーティリティ
rand = "0.10.0"        # ランダム生成（敵スポーン・武器選択肢など）

# データ永続化（メタ進行セーブ）
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.149"

[profile.dev]
opt-level = 1  # 開発中のパフォーマンス向上

[profile.dev.package."*"]
opt-level = 3  # 依存クレートは最適化

[profile.release]
opt-level = 3
lto = true
```

### 1.3 非採用技術の理由
- **bevy_rapier2d（物理エンジン）**: Vampire Survivors系は物理挙動が単純（円形衝突のみ）なため不要。手動衝突判定の方が制御しやすくパフォーマンスも優れる。

---

## 2. ECSアーキテクチャ設計

### 2.1 Entity（エンティティ）一覧

主要エンティティ：
- **Player**: プレイヤーキャラクター（1体）
- **Enemy**: 敵エンティティ（最大数百体）
- **Projectile**: 武器の弾/エフェクト（多数）
- **WeaponAura**: ガーリック等の範囲武器（プレイヤーに付属）
- **WeaponOrbit**: 聖書等の周回武器（プレイヤーに付属）
- **ExperienceGem**: 経験値ジェム（多数）
- **GoldCoin**: ゴールドコイン（少数）
- **Treasure**: 宝箱（マップ上に点在）
- **Camera**: プレイヤー追従カメラ
- **UI Elements**: HUD・メニュー等

### 2.2 Component（コンポーネント）設計

#### 2.2.1 プレイヤー関連コンポーネント

```rust
/// プレイヤーマーカー
#[derive(Component)]
pub struct Player;

/// プレイヤーの基本ステータス
#[derive(Component)]
pub struct PlayerStats {
    pub max_hp: f32,
    pub current_hp: f32,
    pub move_speed: f32,
    pub luck: f32,                    // ラック倍率
    pub damage_multiplier: f32,       // 攻撃力倍率
    pub cooldown_reduction: f32,      // クールダウン削減率 (0.0〜0.9)
    pub projectile_speed_mult: f32,   // 弾速倍率
    pub duration_multiplier: f32,     // 武器持続時間倍率
    pub pickup_radius: f32,           // XPジェム吸収範囲
    pub area_multiplier: f32,         // 武器範囲倍率
    pub extra_projectiles: u32,       // 追加発射数
}

/// 所持武器一覧（最大6種）
#[derive(Component)]
pub struct WeaponInventory {
    pub weapons: Vec<WeaponState>,
}

/// 個別武器の状態
#[derive(Clone)]
pub struct WeaponState {
    pub weapon_type: WeaponType,
    pub level: u8,              // 1〜8
    pub cooldown_timer: f32,    // 現在のクールダウン残り時間
    pub evolved: bool,          // 進化済みか
}

/// 所持パッシブアイテム一覧
#[derive(Component)]
pub struct PassiveInventory {
    pub items: Vec<PassiveState>,
}

/// 個別パッシブアイテムの状態
#[derive(Clone)]
pub struct PassiveState {
    pub item_type: PassiveItemType,
    pub level: u8,  // 1〜5
}

/// 無敵時間（被ダメージ後）
#[derive(Component)]
pub struct InvincibilityTimer {
    pub remaining: f32,
}
```

#### 2.2.2 敵関連コンポーネント

```rust
/// 敵タイプ
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum EnemyType {
    Bat,
    Skeleton,
    Zombie,
    Ghost,
    Demon,
    Medusa,
    Dragon,
    BossDeath,
}

/// 敵の基本ステータス
#[derive(Component)]
pub struct Enemy {
    pub enemy_type: EnemyType,
    pub max_hp: f32,
    pub current_hp: f32,
    pub move_speed: f32,
    pub damage: f32,        // プレイヤーへの接触ダメージ
    pub xp_value: u32,      // 倒した時のXP
    pub gold_chance: f32,   // ゴールドドロップ確率 (0.0〜1.0)
}

/// 敵のAI状態
#[derive(Component)]
pub struct EnemyAI {
    pub ai_type: AIType,
    pub attack_timer: f32,   // 遠距離攻撃用タイマー
    pub attack_range: f32,   // 攻撃射程
}

#[derive(Clone, Copy)]
pub enum AIType {
    ChasePlayer,       // プレイヤーに直進
    KeepDistance,      // 距離を保ちながら遠距離攻撃（メデューサ）
    ChargeAttack,      // 突進攻撃（ドラゴン）
    BossMultiPhase,    // ボス用マルチフェーズ
}

/// 被ダメージ点滅エフェクト用
#[derive(Component)]
pub struct DamageFlash {
    pub timer: f32,
}
```

#### 2.2.3 弾・武器エフェクト関連コンポーネント

```rust
/// 投射体（弾）
#[derive(Component)]
pub struct Projectile {
    pub damage: f32,
    pub piercing: u32,          // 貫通可能残り回数（0 = 貫通なし）
    pub hit_enemies: Vec<Entity>,  // 既にヒットした敵（貫通時の重複防止）
    pub lifetime: f32,           // 残り存在時間
    pub weapon_type: WeaponType,
}

/// 投射体の速度
#[derive(Component)]
pub struct ProjectileVelocity(pub Vec2);

/// 周回武器（聖書など）
#[derive(Component)]
pub struct OrbitWeapon {
    pub damage: f32,
    pub orbit_radius: f32,
    pub orbit_speed: f32,  // rad/s
    pub orbit_angle: f32,  // 現在の角度
    pub hit_cooldown: HashMap<Entity, f32>,  // 敵ごとのヒットクールダウン
}

/// オーラ武器（ガーリックなど）
#[derive(Component)]
pub struct AuraWeapon {
    pub damage: f32,
    pub radius: f32,
    pub tick_timer: f32,  // ダメージ頻度タイマー
    pub tick_interval: f32,
}
```

#### 2.2.4 ゲームオブジェクト関連コンポーネント

```rust
/// 経験値ジェム
#[derive(Component)]
pub struct ExperienceGem {
    pub value: u32,
}

/// ゴールドコイン
#[derive(Component)]
pub struct GoldCoin {
    pub value: u32,
}

/// 宝箱
#[derive(Component)]
pub struct Treasure;

/// 物理的な存在（衝突判定に使用）
#[derive(Component)]
pub struct CircleCollider {
    pub radius: f32,
}

/// アイテム（XPジェム・ゴールド）の吸引アニメーション用
#[derive(Component)]
pub struct AttractedToPlayer {
    pub speed: f32,
}
```

### 2.3 Resource（リソース）設計

```rust
/// ゲームの進行状態
#[derive(Resource)]
pub struct GameData {
    pub elapsed_time: f32,      // ゲーム経過秒数
    pub current_level: u32,
    pub current_xp: u32,
    pub xp_to_next_level: u32,
    pub kill_count: u32,
    pub gold_earned: u32,       // このセッションで獲得したゴールド
    pub is_paused: bool,
}

/// メタ進行データ（セーブ対象）
#[derive(Resource, serde::Serialize, serde::Deserialize)]
pub struct MetaProgress {
    pub total_gold: u32,
    pub unlocked_characters: Vec<CharacterType>,
    pub purchased_upgrades: Vec<MetaUpgradeType>,
}

impl MetaProgress {
    pub fn load() -> Self { /* save/meta.json から読み込み */ }
    pub fn save(&self) { /* save/meta.json に保存 */ }
}

/// 選択中のキャラクタータイプ
#[derive(Resource)]
pub struct SelectedCharacter(pub CharacterType);

/// 敵のスポーン管理
#[derive(Resource)]
pub struct EnemySpawner {
    pub spawn_timer: f32,
    pub spawn_interval: f32,    // 現在のスポーン間隔
    pub difficulty_multiplier: f32,  // 時間に応じた難易度倍率
}

/// レベルアップ時の選択肢
#[derive(Resource)]
pub struct LevelUpChoices {
    pub choices: Vec<UpgradeChoice>,
}

#[derive(Clone)]
pub enum UpgradeChoice {
    NewWeapon(WeaponType),
    WeaponUpgrade(WeaponType),
    PassiveItem(PassiveItemType),
    PassiveUpgrade(PassiveItemType),
}

/// オーディオハンドル
#[derive(Resource)]
pub struct AudioHandles {
    pub bgm_title: Handle<AudioSource>,
    pub bgm_gameplay: Handle<AudioSource>,
    pub bgm_boss: Handle<AudioSource>,
    pub bgm_gameover: Handle<AudioSource>,
    pub bgm_victory: Handle<AudioSource>,
    pub sfx_attack_whip: Handle<AudioSource>,
    pub sfx_attack_projectile: Handle<AudioSource>,
    pub sfx_enemy_death: Handle<AudioSource>,
    pub sfx_level_up: Handle<AudioSource>,
    pub sfx_treasure: Handle<AudioSource>,
    pub sfx_boss_appear: Handle<AudioSource>,
    pub sfx_player_damage: Handle<AudioSource>,
}
```

### 2.4 System（システム）設計

#### 2.4.1 システムカテゴリと実行タイミング

**Startupシステム:**
- `setup_camera`: カメラのセットアップ（**vs-ui** が担当）
- `load_assets`: アセット読み込み（スプライト・BGM・SFX）
- `setup_map`: 背景マップの初期生成

**Updateシステム（Playing状態のみ）:**

1. **入力処理**
   - `handle_player_movement`: WASD/矢印キー入力→プレイヤー移動

2. **武器システム**
   - `update_weapon_cooldowns`: 全武器のクールダウンを更新
   - `fire_projectile_weapons`: 弾発射型武器の処理
   - `update_orbit_weapons`: 周回武器の位置更新
   - `update_aura_weapons`: オーラ武器のダメージ処理

3. **投射体システム**
   - `move_projectiles`: 投射体の移動
   - `despawn_expired_projectiles`: 寿命切れ投射体の削除

4. **敵システム**
   - `spawn_enemies`: 難易度に応じた敵スポーン
   - `update_enemy_ai`: 各AIタイプの行動処理
   - `move_enemies`: 敵の移動

5. **衝突判定システム**
   - `projectile_enemy_collision`: 弾 vs 敵の衝突判定
   - `aura_enemy_collision`: オーラ vs 敵の衝突判定
   - `orbit_enemy_collision`: 周回武器 vs 敵の衝突判定
   - `enemy_player_collision`: 敵 vs プレイヤーの衝突判定
   - `player_gem_pickup`: プレイヤー vs XPジェム/コインの吸収判定
   - `player_treasure_pickup`: プレイヤー vs 宝箱の開封判定

6. **ゲームロジックシステム**
   - `update_game_timer`: 経過時間更新
   - `update_xp_and_level`: XP・レベル管理
   - `apply_damage_to_player`: プレイヤーへのダメージ適用
   - `apply_damage_to_enemy`: 敵へのダメージ適用
   - `check_enemy_death`: 敵の死亡判定・ドロップ処理
   - `check_player_death`: プレイヤー死亡判定
   - `check_boss_spawn`: ボス出現タイミング判定
   - `attract_gems_to_player`: XPジェムの吸引処理
   - `spawn_treasure`: 宝箱のスポーン
   - `update_difficulty`: 時間に応じた難易度更新

7. **カメラシステム**（**vs-ui** が担当）
   - `camera_follow_player`: カメラのプレイヤー追従

8. **UIシステム**（**vs-ui** が担当）
   - `update_hud`: HUD表示更新（HP・XP・タイマー・武器アイコン等）

9. **エフェクトシステム**
   - `update_damage_flash`: 被ダメージ点滅
   - `despawn_dead_entities`: 死亡エンティティの削除

#### 2.4.2 システム実行順序

```rust
// システムセットの定義
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSystemSet {
    Input,
    Weapons,
    EnemyAI,
    Movement,
    Collision,
    GameLogic,
    Camera,
    UI,
    Effects,
}

// 実行順序の設定
app.configure_sets(
    Update,
    (
        GameSystemSet::Input,
        GameSystemSet::Weapons,
        GameSystemSet::EnemyAI,
        GameSystemSet::Movement,
        GameSystemSet::Collision,
        GameSystemSet::GameLogic,
        GameSystemSet::Camera,
        GameSystemSet::UI,
        GameSystemSet::Effects,
    ).chain().run_if(in_state(AppState::Playing))
);
```

### 2.5 State（ゲーム状態）管理

```rust
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Title,           // タイトル画面
    CharacterSelect, // キャラクター選択画面
    Playing,         // ゲームプレイ中
    LevelUp,         // レベルアップ選択中（Playingの上にオーバーレイ）
    Paused,          // ポーズ中
    GameOver,        // ゲームオーバー画面
    Victory,         // 勝利画面
    MetaShop,        // ゴールドショップ（タイトルから遷移）
}
```

**状態遷移:**
```
Title ──────────────────→ MetaShop
  │                         │
  ↓                         ↓
CharacterSelect ─────────→ Title
  │
  ↓
Playing ←──────────────── LevelUp（選択後に復帰）
  │  ↑
  │  │ ESC
  ↓  │
Paused
  │  │
  │  └→ Playing（再開）
  │  └→ Title（タイトルに戻る）
  │
  ├──→ GameOver（HP=0）
  └──→ Victory（ボス撃破）
```

---

## 3. 衝突判定設計

### 3.1 手動円形衝突判定

物理エンジン（Rapier）を使用せず、シンプルな円形衝突判定を実装：

```rust
/// 2つの円が衝突しているか判定
pub fn check_circle_collision(
    pos1: Vec2,
    radius1: f32,
    pos2: Vec2,
    radius2: f32,
) -> bool {
    let distance_sq = pos1.distance_squared(pos2);
    let radius_sum = radius1 + radius2;
    distance_sq < radius_sum * radius_sum
}
```

### 3.2 衝突半径設計

| エンティティ | 衝突半径 |
|------------|---------|
| プレイヤー | 12 px |
| コウモリ | 8 px |
| スケルトン | 12 px |
| ゾンビ | 14 px |
| ゴースト | 10 px |
| デーモン | 14 px |
| メデューサ | 12 px |
| ドラゴン | 20 px |
| ボスデス | 30 px |
| 弾（小） | 5 px |
| 弾（大） | 10 px |
| XPジェム | 6 px |
| ゴールドコイン | 6 px |
| 宝箱 | 20 px |
| XP吸収範囲 | 80 px（強化可能） |

### 3.3 空間最適化（グリッドパーティショニング）

敵が多数（300体以上）になる場合、全敵との総当たり衝突判定は O(n²) になる。グリッドパーティショニングで最適化：

```rust
/// 空間グリッド（衝突判定の最適化）
#[derive(Resource)]
pub struct SpatialGrid {
    pub cell_size: f32,                          // グリッドセルサイズ（例: 64px）
    pub cells: HashMap<(i32, i32), Vec<Entity>>, // セル座標 → エンティティリスト
}

impl SpatialGrid {
    /// エンティティをグリッドに登録
    pub fn insert(&mut self, entity: Entity, pos: Vec2) {
        let cell = self.pos_to_cell(pos);
        self.cells.entry(cell).or_default().push(entity);
    }

    /// 近傍エンティティを取得（1セル分の余裕を持たせる）
    pub fn get_nearby(&self, pos: Vec2, radius: f32) -> Vec<Entity> {
        let cells = self.get_nearby_cells(pos, radius);
        cells.iter()
            .flat_map(|c| self.cells.get(c).cloned().unwrap_or_default())
            .collect()
    }

    fn pos_to_cell(&self, pos: Vec2) -> (i32, i32) {
        (
            (pos.x / self.cell_size).floor() as i32,
            (pos.y / self.cell_size).floor() as i32,
        )
    }
}
```

---

## 4. カメラシステム

カメラ関連の実装は **vs-ui クレート** (`app/ui/src/camera.rs`) に配置する。
UIとカメラを vs-core から分離することで、テスタビリティと今後のUI拡張性を確保する。

### 4.1 プレイヤー追従カメラ

```rust
// app/ui/src/camera.rs
use bevy::prelude::*;
use vs_core::components::Player;
use vs_core::constants::CAMERA_LERP_SPEED;

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Transform::from_xyz(0.0, 0.0, 999.9)));
}

pub fn camera_follow_player(
    time: Res<Time>,
    player_q: Query<&Transform, With<Player>>,
    mut camera_q: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    let Ok(player_tf) = player_q.single() else { return; };
    let Ok(mut camera_tf) = camera_q.single_mut() else { return; };

    let target = player_tf.translation.truncate();
    let current = camera_tf.translation.truncate();
    // 指数補間（CAMERA_LERP_SPEED × Δt）
    let lerped = current.lerp(target, CAMERA_LERP_SPEED * time.delta_secs());
    camera_tf.translation.x = lerped.x;
    camera_tf.translation.y = lerped.y;
}
```

### 4.2 マップ境界クランプ（将来的な有限マップの場合）
- 無限スクロールマップの場合はクランプ不要
- 有限マップの場合はカメラをマップ境界内にクランプ

---

## 5. イベント駆動設計

### 5.1 カスタムイベント一覧

```rust
/// プレイヤーがダメージを受けた
#[derive(Event)]
pub struct PlayerDamagedEvent {
    pub damage: f32,
    pub source_position: Vec2,
}

/// 敵が死亡した
#[derive(Event)]
pub struct EnemyDiedEvent {
    pub entity: Entity,
    pub enemy_type: EnemyType,
    pub position: Vec2,
    pub xp_value: u32,
    pub gold_chance: f32,
}

/// レベルアップした
#[derive(Event)]
pub struct LevelUpEvent {
    pub new_level: u32,
}

/// 武器が攻撃した（エフェクト・SFX用）
#[derive(Event)]
pub struct WeaponFiredEvent {
    pub weapon_type: WeaponType,
    pub position: Vec2,
}

/// 宝箱を開けた
#[derive(Event)]
pub struct TreasureOpenedEvent {
    pub position: Vec2,
    pub content: TreasureContent,
}

/// ボスが出現した
#[derive(Event)]
pub struct BossSpawnedEvent;

/// ゲームオーバー
#[derive(Event)]
pub struct GameOverEvent {
    pub survived_time: f32,
    pub kill_count: u32,
    pub gold_earned: u32,
}

/// 勝利
#[derive(Event)]
pub struct VictoryEvent {
    pub clear_time: f32,
    pub kill_count: u32,
    pub gold_earned: u32,
}
```

---

## 6. プラグイン構成

5クレート構成。カメラ・UIは `vs-ui` クレートの `GameUIPlugin` が担当する。

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
        // Load assets first (other plugins may reference them)
        .add_plugins(GameAssetsPlugin)
        // Core game logic (ECS, systems)
        .add_plugins(GameCorePlugin)
        // UI and camera (depends on GameCorePlugin for AppState + Player)
        .add_plugins(GameUIPlugin)
        // Audio (receives core events for BGM/SFX switching)
        .add_plugins(GameAudioPlugin)
        .run();
}

// GameCorePlugin: コアゲームロジック（カメラ・UIは含まない）
pub struct GameCorePlugin;
impl Plugin for GameCorePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<AppState>()
            // リソース初期化
            .insert_resource(GameData::default())
            .insert_resource(EnemySpawner::default())
            .insert_resource(SpatialGrid::default())
            // イベント登録
            .add_event::<PlayerDamagedEvent>()
            .add_event::<EnemyDiedEvent>()
            .add_event::<LevelUpEvent>()
            .add_event::<WeaponFiredEvent>()
            .add_event::<TreasureOpenedEvent>()
            .add_event::<BossSpawnedEvent>()
            .add_event::<GameOverEvent>()
            .add_event::<VictoryEvent>()
            // システム登録（プレイヤー・敵・武器・衝突・XP・メタ等）
            .add_systems(OnEnter(AppState::Playing), spawn_player)
            .add_systems(Update, (/* 各システム */));
    }
}

// GameUIPlugin: カメラ・UI画面（vs-ui クレート）
pub struct GameUIPlugin;
impl Plugin for GameUIPlugin {
    fn build(&self, app: &mut App) {
        app
            // カメラは常駐（タイトル・メニューでも使用）
            .add_systems(Startup, camera::setup_camera)
            // プレイヤー追従はPlaying状態でのみ実行
            .add_systems(
                Update,
                camera::camera_follow_player.run_if(in_state(AppState::Playing)),
            );
    }
}
```

---

## 7. パフォーマンス最適化戦略

### 7.1 ECSベストプラクティス
- クエリフィルタの活用（`With<Enemy>`, `Without<Player>`）
- コンポーネントのキャッシュ効率を意識した設計
- 死亡エンティティは即座にDespawn（メモリリーク防止）
- 弾・エフェクトはライフタイム管理で確実に削除

### 7.2 衝突判定最適化
- **空間グリッドパーティショニング**: O(n²) → O(n) に削減
- **グリッドセルサイズ**: 最大衝突半径の2倍程度（例: 64px）
- 毎フレームグリッドを再構築（エンティティが動くため）

### 7.3 レンダリング最適化
- スプライトバッチング（同じテクスチャのスプライトをまとめて描画）
- 画面外エンティティの早期カリング
- 弾・エフェクトのオブジェクトプール（将来最適化）

### 7.4 大量エンティティ処理
- 敵移動はシンプルな直進計算（物理演算なし）
- 並列システム処理（Bevyの自動並列化を活用）
- 空間グリッドでの近傍クエリ

---

## 8. テスト戦略

### 8.1 単体テスト
- 武器ダメージ計算
- 衝突判定ロジック
- XP・レベル計算
- メタ進行データの保存/読み込み

### 8.2 統合テスト
- システム間のイベントフロー
- 武器進化条件の確認

### 8.3 手動テスト
- 大量エンティティ時のパフォーマンス（300体以上）
- ゲームバランスの確認
- 30分プレイテスト
