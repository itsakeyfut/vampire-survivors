# Phase 3: 敵システム基本

## フェーズ概要

**ステータス**: 🔲 未着手
**推定工数**: 4-6時間
**依存関係**: Phase 2

### 目的
基本的な敵（コウモリ・スケルトン）のスポーン・プレイヤーへの追跡AIを実装し、画面上に敵が出現してプレイヤーを追いかける状態にする。

### スコープ
- 敵コンポーネント・EnemyAI の定義
- 敵スポーンシステム（画面外からランダムに出現）
- 基本的な追跡AI（プレイヤーに向かって直進）
- 時間による難易度スケーリング（難易度倍率）
- 敵プレースホルダースプライト（単色の円）

---

## タスクリスト

### タスク 3.1: 敵コンポーネントの定義

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-3

**説明**:
`Enemy`, `EnemyAI`, `EnemyType` 等の敵関連コンポーネントを定義する。

**受け入れ基準**:
- [ ] `components.rs` に `Enemy` コンポーネントが追加されている
- [ ] `types.rs` に `EnemyType`, `AIType` が定義されている
- [ ] `resources.rs` に `EnemySpawner` が定義されている

**実装ガイド**:
```rust
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum EnemyType {
    Bat, Skeleton, Zombie, Ghost, Demon, Medusa, Dragon, BossDeath,
}

#[derive(Component)]
pub struct Enemy {
    pub enemy_type: EnemyType,
    pub max_hp: f32,
    pub current_hp: f32,
    pub move_speed: f32,
    pub damage: f32,
    pub xp_value: u32,
    pub gold_chance: f32,
}

#[derive(Resource)]
pub struct EnemySpawner {
    pub spawn_timer: f32,
    pub base_interval: f32,    // 0.5秒
    pub difficulty_multiplier: f32,
    pub active: bool,
}
```

---

### タスク 3.2: 敵スポーンシステム

**優先度**: P0
**推定工数**: 1.5時間
**ラベル**: task, phase-3

**説明**:
タイマーに基づいて画面外の位置にランダムに敵をスポーンする。

**受け入れ基準**:
- [ ] Playing状態中、一定間隔で画面外から敵が出現する
- [ ] 敵は画面の4辺のいずれかからランダムに出現する
- [ ] 時間経過とともにスポーン間隔が短くなる（難易度倍率）
- [ ] コウモリとスケルトンの2種が出現する（この段階では確率50%ずつ）

**実装ガイド**:
```rust
pub fn spawn_enemies(
    mut spawner: ResMut<EnemySpawner>,
    game_data: Res<GameData>,
    camera_query: Query<&Transform, With<Camera>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    if !spawner.active { return; }
    spawner.spawn_timer += time.delta_secs();

    let effective_interval = spawner.base_interval / spawner.difficulty_multiplier;
    if spawner.spawn_timer < effective_interval { return; }
    spawner.spawn_timer = 0.0;

    let Ok(cam_transform) = camera_query.get_single() else { return };
    let cam_pos = cam_transform.translation.truncate();
    let spawn_pos = get_spawn_position(cam_pos, Vec2::new(1280.0, 720.0));

    let enemy_type = if rand::random::<f32>() < 0.5 {
        EnemyType::Bat
    } else {
        EnemyType::Skeleton
    };

    spawn_enemy_entity(&mut commands, enemy_type, spawn_pos, spawner.difficulty_multiplier);
}

fn spawn_enemy_entity(
    commands: &mut Commands,
    enemy_type: EnemyType,
    position: Vec2,
    difficulty: f32,
) {
    let (max_hp, speed, damage, xp, gold_chance, radius, color) = match enemy_type {
        EnemyType::Bat => (10.0 * difficulty, 150.0, 5.0, 3, 0.05, 8.0, Color::srgb(0.5, 0.1, 0.8)),
        EnemyType::Skeleton => (30.0 * difficulty, 80.0, 8.0, 5, 0.08, 12.0, Color::srgb(0.9, 0.9, 0.8)),
        _ => (20.0, 100.0, 8.0, 4, 0.05, 10.0, Color::srgb(0.7, 0.3, 0.3)),
    };

    commands.spawn((
        enemy_type,
        Enemy {
            enemy_type,
            max_hp,
            current_hp: max_hp,
            move_speed: speed,
            damage,
            xp_value: xp,
            gold_chance,
        },
        EnemyAI { ai_type: AIType::ChasePlayer, attack_timer: 0.0, attack_range: 20.0 },
        CircleCollider { radius },
        Sprite {
            color,
            custom_size: Some(Vec2::splat(radius * 2.0)),
            ..default()
        },
        Transform::from_translation(position.extend(5.0)),
    ));
}
```

---

### タスク 3.3: 敵の追跡AI

**優先度**: P0
**推定工数**: 1時間
**ラベル**: task, phase-3

**説明**:
基本AI（ChasePlayer）を実装する。敵がプレイヤーに向かって直進するシステム。

**受け入れ基準**:
- [ ] 敵がプレイヤーの位置に向かって移動する
- [ ] 移動速度が `Enemy.move_speed` に基づいている
- [ ] 複数の敵が同時に移動しても正常に動作する

**実装ガイド**:
```rust
pub fn move_enemies(
    player_query: Query<&Transform, With<Player>>,
    mut enemy_query: Query<(&Enemy, &mut Transform), (With<Enemy>, Without<Player>)>,
    time: Res<Time>,
) {
    let Ok(player_transform) = player_query.get_single() else { return };
    let player_pos = player_transform.translation.truncate();

    for (enemy, mut transform) in enemy_query.iter_mut() {
        let enemy_pos = transform.translation.truncate();
        let direction = (player_pos - enemy_pos).normalize_or_zero();
        transform.translation += (direction * enemy.move_speed * time.delta_secs()).extend(0.0);
    }
}
```

---

### タスク 3.4: 難易度スケーリング

**優先度**: P1
**推定工数**: 0.5時間
**ラベル**: task, phase-3

**説明**:
時間経過とともに敵の難易度倍率を更新するシステムを実装する。

**受け入れ基準**:
- [ ] 1分ごとに難易度倍率が増加する
- [ ] 難易度倍率が敵HPに乗算される
- [ ] スポーン間隔が難易度倍率に応じて短くなる

---

### タスク 3.5: 画面外敵の削除（オプション最適化）

**優先度**: P2
**推定工数**: 0.5時間
**ラベル**: task, phase-3

**説明**:
プレイヤーから非常に遠い位置にいる敵を削除してメモリを節約する。

**受け入れ基準**:
- [ ] プレイヤーから2000px以上離れた敵が削除される
- [ ] 削除時にXPジェムはドロップしない（消えるだけ）

---

## フェーズ検証

### 検証項目
- [ ] 画面外から敵（コウモリ・スケルトン）が出現する
- [ ] 敵がプレイヤーに向かって移動する
- [ ] 時間経過とともに敵の出現頻度が増える
- [ ] 大量の敵（100体）が同時に動いてもパフォーマンスが問題ない

## 次のフェーズ

Phase 3 完了 → 次は **Phase 4: 武器システム基本** に進む
