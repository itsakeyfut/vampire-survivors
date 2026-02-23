# Phase 4: 武器システム基本

## フェーズ概要

**ステータス**: 🔲 未着手
**推定工数**: 4-6時間
**依存関係**: Phase 3

### 目的
武器インベントリシステムを構築し、ムチとマジックワンドを実装する。プレイヤーが自動的に攻撃できる状態にする。

### スコープ
- `WeaponInventory`, `WeaponState` の実装
- 武器クールダウン管理システム
- ムチ（扇形スイング）の実装
- マジックワンド（追尾弾）の実装
- 投射体（`Projectile`）システム

---

## タスクリスト

### タスク 4.1: 武器データ構造の定義

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-4

**説明**:
武器インベントリ・個別武器状態の構造体を定義する。

**受け入れ基準**:
- [ ] `WeaponInventory` コンポーネントが定義されている
- [ ] `WeaponState` 構造体が定義されている（武器タイプ・レベル・クールダウンタイマー）
- [ ] `WeaponType` enumに全8武器が定義されている

**実装ガイド**:
```rust
#[derive(Component, Default)]
pub struct WeaponInventory {
    pub weapons: Vec<WeaponState>,
}

#[derive(Clone)]
pub struct WeaponState {
    pub weapon_type: WeaponType,
    pub level: u8,
    pub cooldown_timer: f32,
    pub evolved: bool,
}

impl WeaponState {
    pub fn new(weapon_type: WeaponType) -> Self {
        Self {
            weapon_type,
            level: 1,
            cooldown_timer: 0.0,
            evolved: false,
        }
    }
}
```

---

### タスク 4.2: 武器クールダウン管理システム

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-4

**説明**:
全武器のクールダウンタイマーを毎フレーム更新するシステム。クールダウンが0になったら発射トリガーをセットする。

**受け入れ基準**:
- [ ] 毎フレーム `WeaponState.cooldown_timer` が減少する
- [ ] タイマーが0になったら該当武器を発射し、クールダウンをリセットする
- [ ] プレイヤーのクールダウン削減率（`PlayerStats.cooldown_reduction`）が適用される

---

### タスク 4.3: 投射体システム

**優先度**: P0
**推定工数**: 1時間
**ラベル**: task, phase-4

**説明**:
弾（`Projectile`）エンティティの生成・移動・寿命管理を実装する。

**受け入れ基準**:
- [ ] `Projectile` コンポーネントが定義されている
- [ ] 投射体が発射方向に毎フレーム移動する
- [ ] 寿命（`lifetime`）が0になった投射体が削除される

**実装ガイド**:
```rust
#[derive(Component)]
pub struct Projectile {
    pub damage: f32,
    pub piercing: u32,
    pub hit_entities: Vec<Entity>,
    pub lifetime: f32,
    pub weapon_type: WeaponType,
}

#[derive(Component)]
pub struct ProjectileVelocity(pub Vec2);

pub fn move_projectiles(
    mut query: Query<(&mut Transform, &ProjectileVelocity)>,
    time: Res<Time>,
) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += (velocity.0 * time.delta_secs()).extend(0.0);
    }
}

pub fn despawn_expired_projectiles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Projectile)>,
    time: Res<Time>,
) {
    for (entity, mut projectile) in query.iter_mut() {
        projectile.lifetime -= time.delta_secs();
        if projectile.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
```

---

### タスク 4.4: ムチ（Whip）の実装

**優先度**: P0
**推定工数**: 1.5時間
**ラベル**: task, phase-4

**説明**:
ムチの扇形スイング攻撃を実装する。左右交互に即時ダメージ判定。

**受け入れ基準**:
- [ ] クールダウン後、プレイヤー左右交互に扇形範囲攻撃が発生する
- [ ] 範囲内の敵にダメージイベントが発行される
- [ ] 視覚的なフィードバック（スイングエフェクト）がある（シンプルな短命スプライトでOK）

**実装ガイド**:
```rust
// 疑似コード
fn fire_whip(player_pos, direction, range, damage, nearby_enemies) {
    for each enemy in nearby_enemies {
        let rel = enemy_pos - player_pos;
        if rel.x * direction > 0 && rel.length < range && rel.y.abs() < range * 0.6 {
            send DamageEnemyEvent { entity, damage }
        }
    }
    flip direction (left↔right)
}
```

---

### タスク 4.5: マジックワンド（Magic Wand）の実装

**優先度**: P0
**推定工数**: 1時間
**ラベル**: task, phase-4

**説明**:
最も近い敵に向かって追尾弾（Projectile）を発射するシステムを実装する。

**受け入れ基準**:
- [ ] クールダウン後、最も近い敵に向かって弾が発射される
- [ ] 弾がその方向に直進する
- [ ] 弾がマップ外に出たりライフタイム切れで消える

---

### タスク 4.6: 初期武器のセットアップ

**優先度**: P1
**推定工数**: 0.5時間
**ラベル**: task, phase-4

**説明**:
キャラクター種別に応じた初期武器をプレイヤーにセットする。

**受け入れ基準**:
- [ ] デフォルトキャラクターはムチ（Whip）から開始する
- [ ] 武器がWeaponInventoryに追加された状態でゲームが始まる

---

## フェーズ検証

### 検証項目
- [ ] プレイヤーがムチで左右交互に自動攻撃する
- [ ] マジックワンドが近くの敵に弾を発射する
- [ ] 弾が直進して消える
- [ ] 武器クールダウンが正しく機能する

## 次のフェーズ

Phase 4 完了 → 次は **Phase 5: 衝突・ダメージシステム** に進む
