# Vampire Survivors クローン - 実装計画書

## 1. 実装フェーズ概要

本プロジェクトは17のフェーズに分けて段階的に実装します。各フェーズは独立してテスト可能で、前のフェーズの成果物を基に構築されます。

### 1.1 フェーズ一覧

| フェーズ | 名称 | 推定工数 | 依存関係 |
|---------|------|---------|---------|
| Phase 1 | プロジェクトセットアップ | 2-3時間 | なし |
| Phase 2 | コアゲームループ（プレイヤー移動・カメラ） | 3-4時間 | Phase 1 |
| Phase 3 | 敵システム基本（スポーン・移動・衝突判定） | 4-6時間 | Phase 2 |
| Phase 4 | 武器システム基本（ムチ・マジックワンド） | 4-6時間 | Phase 3 |
| Phase 5 | 衝突・ダメージシステム | 3-5時間 | Phase 4 |
| Phase 6 | 経験値・レベルアップシステム | 4-6時間 | Phase 5 |
| Phase 7 | 武器拡充（全8種実装） | 6-8時間 | Phase 6 |
| Phase 8 | パッシブアイテム・武器進化 | 5-7時間 | Phase 7 |
| Phase 9 | 敵バリエーション（全7種実装） | 4-6時間 | Phase 5 |
| Phase 10 | ボスシステム（デス実装） | 4-6時間 | Phase 9 |
| Phase 11 | UI/HUD実装 | 6-8時間 | Phase 6 |
| Phase 12 | キャラクター選択画面 | 3-4時間 | Phase 11 |
| Phase 13 | 宝箱システム | 3-4時間 | Phase 8 |
| Phase 14 | メタ進行（ゴールドショップ） | 5-7時間 | Phase 13 |
| Phase 15 | オーディオ統合 | 4-6時間 | Phase 11 |
| Phase 16 | エフェクト・ポリッシュ | 6-8時間 | Phase 15 |
| Phase 17 | ピクセルアート統合（後日） | 10-15時間 | Phase 16 |

**総推定工数**: 80〜120時間（ピクセルアート作成を除く）

---

## 2. Phase 1: プロジェクトセットアップ

### 2.1 目標
Cargoワークスペース・開発環境・ドキュメントを整備し、ビルドが通る状態にする。

### 2.2 作業内容

#### 2.2.1 Cargoワークスペースの設定

```toml
# Cargo.toml (workspace root)
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
```

#### 2.2.2 ディレクトリ構造の作成

```bash
mkdir -p app/core/src
mkdir -p app/audio/src
mkdir -p app/assets/src
mkdir -p app/vampire-survivors/src
mkdir -p assets/sprites/player
mkdir -p assets/sprites/enemies
mkdir -p assets/sprites/weapons
mkdir -p assets/sprites/ui
mkdir -p assets/sounds/bgm
mkdir -p assets/sounds/sfx/weapons
mkdir -p assets/sounds/sfx/enemies
mkdir -p assets/sounds/sfx/player
mkdir -p assets/sounds/sfx/events
mkdir -p assets/sounds/sfx/ui
mkdir -p assets/fonts
mkdir -p save
```

#### 2.2.3 各クレートの基本 Cargo.toml

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

#### 2.2.4 基本的な main.rs

```rust
use bevy::prelude::*;

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
        .run();
}
```

### 2.3 検証
- `cargo build` が成功する
- `cargo run` でウィンドウが開く
- ディレクトリ構造が正しく作成されている

---

## 3. Phase 2: コアゲームループ

### 3.1 目標
プレイヤーキャラクターの移動と、プレイヤーを追従するカメラを実装する。

### 3.2 作業内容

#### 3.2.1 実装ファイル
- `app/core/src/components.rs` - 基本コンポーネント
- `app/core/src/resources.rs` - GameData・選択キャラ等
- `app/core/src/states.rs` - AppState定義
- `app/core/src/constants.rs` - ゲーム定数
- `app/core/src/systems/player.rs` - プレイヤー移動
- `app/core/src/systems/camera.rs` - カメラ追従

#### 3.2.2 プレイヤー移動システム

```rust
pub fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Transform, &PlayerStats), With<Player>>,
    time: Res<Time>,
) {
    let Ok((mut transform, stats)) = player_query.get_single_mut() else { return };

    let mut direction = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    if direction != Vec2::ZERO {
        direction = direction.normalize();
    }

    transform.translation += (direction * stats.move_speed * time.delta_secs()).extend(0.0);
}
```

### 3.3 検証
- WASDキーでプレイヤーが移動する
- カメラがプレイヤーを追従する
- プレイヤーのプレースホルダースプライトが表示される

---

## 4. Phase 3: 敵システム基本

### 4.1 目標
基本的な敵（コウモリ・スケルトン）のスポーン、プレイヤーへの追跡移動を実装する。

### 4.2 作業内容

#### 4.2.1 実装ファイル
- `app/core/src/systems/enemy.rs` - 敵スポーン・移動AI
- `app/core/src/components.rs` への追加 - Enemy・EnemyAI
- `app/core/src/resources.rs` への追加 - EnemySpawner

#### 4.2.2 敵スポーンシステムの実装

```rust
pub fn spawn_enemies(
    mut spawner: ResMut<EnemySpawner>,
    game_data: Res<GameData>,
    camera_query: Query<&Transform, With<Camera>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    spawner.spawn_timer += time.delta_secs();

    let interval = spawner.base_interval / spawner.difficulty_multiplier;
    if spawner.spawn_timer < interval {
        return;
    }
    spawner.spawn_timer = 0.0;

    let Ok(camera_transform) = camera_query.get_single() else { return };
    let camera_pos = camera_transform.translation.truncate();

    // スポーンテーブルから敵タイプを選択
    let enemy_type = select_enemy_type(&game_data, &mut rand::thread_rng());
    let spawn_pos = get_spawn_position(camera_pos, Vec2::new(1280.0, 720.0), &mut rand::thread_rng());

    spawn_enemy(&mut commands, enemy_type, spawn_pos, spawner.difficulty_multiplier);
}
```

### 4.3 検証
- 画面外から敵が湧いてくる
- 敵がプレイヤーに向かって移動する
- 時間経過とともに敵の出現が増える（難易度倍率）

---

## 5. Phase 4: 武器システム基本

### 5.1 目標
ムチとマジックワンドを実装し、武器クールダウンシステムを構築する。

### 5.2 作業内容

#### 5.2.1 実装ファイル
- `app/core/src/systems/weapons.rs` - 武器システム全体
- `app/core/src/systems/projectile.rs` - 投射体の移動・寿命管理

#### 5.2.2 武器クールダウンシステム

```rust
pub fn update_weapon_cooldowns(
    mut player_query: Query<&mut WeaponInventory, With<Player>>,
    time: Res<Time>,
) {
    let Ok(mut inventory) = player_query.get_single_mut() else { return };

    for weapon_state in inventory.weapons.iter_mut() {
        weapon_state.cooldown_timer = (weapon_state.cooldown_timer - time.delta_secs()).max(0.0);
    }
}
```

### 5.3 検証
- プレイヤーがムチで自動攻撃する（左右交互の扇形）
- マジックワンドが最も近い敵に弾を発射する
- 弾が敵に当たる（衝突判定の仮実装）

---

## 6. Phase 5: 衝突・ダメージシステム

### 6.1 目標
手動円形衝突判定を実装し、弾→敵・敵→プレイヤーのダメージシステムを構築する。

### 6.2 作業内容

#### 6.2.1 実装ファイル
- `app/core/src/systems/collision.rs` - 衝突判定システム
- `app/core/src/spatial_grid.rs` - 空間グリッドパーティショニング

#### 6.2.2 衝突判定の実装

```rust
pub fn projectile_enemy_collision(
    projectile_query: Query<(Entity, &Transform, &CircleCollider, &Projectile)>,
    enemy_query: Query<(Entity, &Transform, &CircleCollider), With<Enemy>>,
    mut damage_events: EventWriter<DamageEnemyEvent>,
    mut commands: Commands,
) {
    for (proj_entity, proj_transform, proj_collider, projectile) in projectile_query.iter() {
        let proj_pos = proj_transform.translation.truncate();
        let mut hit_count = 0;

        for (enemy_entity, enemy_transform, enemy_collider) in enemy_query.iter() {
            // 既にヒットした敵はスキップ
            if projectile.hit_enemies.contains(&enemy_entity) { continue; }

            let enemy_pos = enemy_transform.translation.truncate();
            if check_circle_collision(proj_pos, proj_collider.radius, enemy_pos, enemy_collider.radius) {
                damage_events.send(DamageEnemyEvent {
                    entity: enemy_entity,
                    damage: projectile.damage,
                    weapon_type: projectile.weapon_type,
                });
                hit_count += 1;
            }
        }

        // 貫通数を超えたら弾を削除
        // （実際はDamageEnemyEvent処理後にhit_enemiesを更新する）
    }
}
```

### 6.3 検証
- 弾が敵に当たるとダメージが入る
- 敵のHPが0になると死亡する
- 敵がプレイヤーに接触するとプレイヤーがダメージを受ける
- プレイヤーのHPが0でゲームオーバー画面に遷移する
- 300体の敵でも60fps以上を維持（パフォーマンステスト）

---

## 7. Phase 6: 経験値・レベルアップシステム

### 7.1 目標
XPジェムのドロップ・吸収・レベルアップ・選択画面を実装する。

### 7.2 作業内容

#### 7.2.1 実装ファイル
- `app/core/src/systems/xp.rs` - XP・レベルアップシステム
- `app/core/src/systems/level_up.rs` - 選択肢生成・適用

### 7.3 検証
- 敵を倒すとXPジェムがドロップする
- XPジェムに近づくと自動的に吸収される
- XPバーが満タンになるとレベルアップする
- レベルアップ時に選択画面が表示される（ゲームが一時停止）
- 選択後、選択した強化が適用されてゲーム再開する

---

## 8. Phase 7: 武器拡充（全8種）

### 8.1 目標
残り6種の武器（ナイフ・ガーリック・聖書・稲妻の指輪・クロス・ファイアウォンド）を実装する。

### 8.2 作業内容
各武器タイプに対応する発射ロジックを実装。既存のWeaponSystemに追加する形で拡張。

### 8.3 検証
- 全8種の武器が正しく動作する
- 各武器の攻撃パターンが設計書通りである
- 武器レベルアップ時にパラメータが正しく変化する

---

## 9. Phase 8: パッシブアイテム・武器進化

### 9.1 目標
全9種のパッシブアイテムを実装し、武器進化システムを構築する。

### 9.2 作業内容

#### 9.2.1 実装ファイル
- `app/core/src/systems/passive.rs` - パッシブ効果の計算・適用
- `app/core/src/systems/evolution.rs` - 武器進化ロジック

### 9.3 検証
- パッシブアイテムの効果が正しく適用される（攻撃力・速度等が変化）
- 武器がLv8かつ対応パッシブ所持時、宝箱で進化オプションが出る
- 進化後の武器が設計書通りの挙動をする

---

## 10. Phase 9: 敵バリエーション

### 10.1 目標
残り5種の敵（ゾンビ・ゴースト・デーモン・メデューサ・ドラゴン）を実装する。

### 10.2 作業内容
各敵タイプのAIロジック・パラメータを実装。メデューサの遠距離攻撃、ドラゴンの火球なども実装。

### 10.3 検証
- 全7種の敵が正しくスポーンし、それぞれのAIで動作する
- 時間経過に応じて適切な敵タイプが出現する
- 敵パラメータが時間とともにスケールする

---

## 11. Phase 10: ボスシステム

### 11.1 目標
30分後にボスデスが出現し、3フェーズで戦えるようにする。ボス撃破で勝利画面へ。

### 11.2 作業内容

#### 11.2.1 実装ファイル
- `app/core/src/systems/boss.rs` - ボスAI・フェーズ管理

#### 11.2.2 ボス出現イベント

```rust
pub fn check_boss_spawn(
    game_data: Res<GameData>,
    mut boss_events: EventWriter<BossSpawnedEvent>,
    mut commands: Commands,
    mut spawner: ResMut<EnemySpawner>,
) {
    if game_data.elapsed_time >= 30.0 * 60.0 && !game_data.boss_spawned {
        // 通常の敵スポーンを停止
        spawner.active = false;

        // ボスをスポーン
        spawn_boss(&mut commands);
        boss_events.send(BossSpawnedEvent);
    }
}
```

### 11.3 検証
- 30分後にボスが出現する（通常の敵スポーンが停止）
- ボスが3フェーズで行動パターンが変わる
- ボス撃破で勝利画面に遷移する

---

## 12. Phase 11: UI/HUD実装

### 12.1 目標
ゲームプレイ中のHUD（HP・XP・タイマー・武器アイコン）と全画面UIを実装する。

### 12.2 作業内容

#### 12.2.1 実装ファイル
- `app/core/src/ui/hud.rs` - ゲームプレイHUD
- `app/core/src/ui/title.rs` - タイトル画面
- `app/core/src/ui/game_over.rs` - ゲームオーバー画面
- `app/core/src/ui/victory.rs` - 勝利画面
- `app/core/src/ui/level_up.rs` - レベルアップ選択UI
- `app/core/src/ui/pause.rs` - ポーズ画面

### 12.3 検証
- 全画面のUIが表示される
- HPバー・XPバーが正しく更新される
- タイマーが正しく表示される
- レベルアップカードが正しく表示・選択できる
- ボタンのホバー・クリックが機能する

---

## 13. Phase 12: キャラクター選択画面

### 13.1 目標
キャラクター選択画面を実装する。未解放キャラはロック表示。

### 13.2 作業内容

#### 13.2.1 実装ファイル
- `app/core/src/ui/character_select.rs` - キャラクター選択UI

### 13.3 検証
- 解放済みキャラクターが選択できる
- 未解放キャラクターがロック表示される
- 選択したキャラクターでゲームが開始される

---

## 14. Phase 13: 宝箱システム

### 14.1 目標
マップ上に宝箱をスポーンし、開封時の処理（アップグレード・武器進化・HP回復等）を実装する。

### 14.2 作業内容

#### 14.2.1 実装ファイル
- `app/core/src/systems/treasure.rs` - 宝箱スポーン・開封

### 14.3 検証
- 3分ごとに宝箱がマップ上に出現する
- プレイヤーが宝箱に近づくと自動開封される
- 進化条件を満たしている場合、宝箱で武器進化が発動する
- 宝箱の内容が正しく適用される

---

## 15. Phase 14: メタ進行（ゴールドショップ）

### 15.1 目標
ゴールドの取得・保存・ショップでの購入・アンロック機能を実装する。

### 15.2 作業内容

#### 15.2.1 実装ファイル
- `app/core/src/ui/meta_shop.rs` - ゴールドショップUI
- `app/core/src/systems/meta.rs` - メタ進行データ管理

#### 15.2.2 セーブデータの読み書き

```rust
const SAVE_PATH: &str = "save/meta.json";

pub fn load_meta_progress() -> MetaProgress {
    std::fs::read_to_string(SAVE_PATH)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_meta_progress(meta: &MetaProgress) {
    if let Ok(json) = serde_json::to_string_pretty(meta) {
        let _ = std::fs::write(SAVE_PATH, json);
    }
}
```

### 15.3 検証
- ゲーム中にゴールドが獲得できる
- ゲーム終了後にゴールドが保存される
- ゴールドショップで購入できる
- 購入内容が次のゲームに反映される
- アンロックしたキャラクターが選択画面に表示される

---

## 16. Phase 15: オーディオ統合

### 16.1 目標
BGMと効果音を実装し、各シーン・イベントに対応した音が再生されるようにする。

### 16.2 作業内容

#### 16.2.1 実装ファイル
- `app/audio/src/bgm.rs` - BGM管理
- `app/audio/src/sfx.rs` - SFX再生

### 16.3 検証
- 各画面で適切なBGMが再生される
- 武器攻撃・敵死亡・レベルアップ等のSFXが適切なタイミングで鳴る
- 音量バランスが適切
- BGM切り替えが自然（プツ切れしない）

---

## 17. Phase 16: エフェクト・ポリッシュ

### 17.1 目標
視覚的な演出（ヒットエフェクト・パーティクル・数値表示等）を追加してゲームを磨く。

### 17.2 作業内容
- ダメージ数値フロートテキスト
- 敵死亡時のパーティクルエフェクト
- 武器進化時の派手なエフェクト
- ボス出現時の画面演出
- ゲームバランスの微調整
- パフォーマンス最終チェック（300体以上での60fps確認）

### 17.3 検証
- エフェクトが自然で視認しやすい
- パフォーマンスへの影響が最小限
- ゲームバランスが調整されている

---

## 18. Phase 17: ピクセルアート統合（後日）

### 18.1 目標
プレースホルダーを自作ピクセルアートに置き換え、視覚的に完成した状態にする。

### 18.2 作業内容
- キャラクタースプライト（歩行アニメーション付き）
- 敵スプライト（各種）
- 武器エフェクトスプライト
- UIスプライト（ボタン・バー等）

### 18.3 検証
- 全プレースホルダーが置き換えられている
- アニメーションが正しく再生される
- 視覚的な統一感がある

---

## 19. 開発ワークフロー

### 19.1 各フェーズのワークフロー

1. **ロードマップ確認**: `docs/roadmap/phase-XX.md` を開く
2. **実装**: タスクを1つずつ実装
3. **テスト**: 各タスクの受け入れ基準を確認
4. **コミット**: 動作する状態でコミット
5. **次へ**: フェーズ完了確認後、次フェーズへ

### 19.2 Gitコミット戦略

```
feat(player): プレイヤー移動システムを実装
feat(enemy): 基本的な敵スポーンシステムを実装
feat(weapon): ムチの自動攻撃を実装
feat(xp): 経験値・レベルアップシステムを実装
feat(ui): ゲームプレイHUDを実装
fix(collision): 大量エンティティ時の衝突判定パフォーマンスを修正
```

---

## 20. リスク管理

### 20.1 想定されるリスク

| リスク | 影響度 | 発生確率 | 対策 |
|--------|--------|---------|------|
| 大量エンティティでのパフォーマンス問題 | 高 | 中 | Phase 5で早期にパフォーマンステスト実施 |
| Bevy 0.17.3のAPI変更・バグ | 中 | 低 | Changelog・コミュニティを参照 |
| ゲームバランスの調整困難 | 中 | 高 | 定数ファイルに集約、早期にプレイテスト |
| スコープクリープ | 高 | 中 | MVPを明確に定義し、拡張は後回し |

---

## 21. 成功基準

### 21.1 最小成功基準（MVP、Phase 1〜6完了時）
- [ ] プレイヤーがWASDで移動できる
- [ ] 敵が自動スポーンしてプレイヤーを追跡する
- [ ] ムチが自動攻撃して敵にダメージを与える
- [ ] 敵を倒すとXPジェムが出て吸収できる
- [ ] レベルアップして武器/パッシブを選択できる
- [ ] HPが0でゲームオーバー画面になる
- [ ] 30分タイマーが機能する

### 21.2 フル機能版成功基準（Phase 1〜14完了時）
- [ ] 全8種の武器が動作する
- [ ] 武器進化システムが機能する
- [ ] 全7種＋ボスの敵が実装されている
- [ ] 30分でボスが出現し撃破で勝利できる
- [ ] メタ進行（ゴールドショップ）が機能する
- [ ] 300体同時で60fps以上を維持

### 21.3 完成版成功基準（Phase 1〜17完了時）
- [ ] BGMと全SFXが統合されている
- [ ] 視覚エフェクトが充実している
- [ ] ピクセルアートアセットが統合されている
- [ ] バグが修正されゲームバランスが調整されている
