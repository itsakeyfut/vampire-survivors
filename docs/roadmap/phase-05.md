# Phase 5: 衝突・ダメージシステム

## フェーズ概要

**ステータス**: 🔲 未着手
**推定工数**: 3-5時間
**依存関係**: Phase 4

### 目的
手動円形衝突判定を実装し、弾→敵・敵→プレイヤーのダメージシステムを構築する。敵が死亡するようになり、プレイヤーのHPが0でゲームオーバーになる。

### スコープ
- 手動円形衝突判定の実装
- 空間グリッドパーティショニング（パフォーマンス最適化）
- 弾 vs 敵の衝突・ダメージ
- 敵 vs プレイヤーの衝突・ダメージ（無敵時間付き）
- 敵の死亡判定
- プレイヤー死亡 → ゲームオーバー遷移
- パフォーマンステスト（300体以上）

---

## タスクリスト

### タスク 5.1: 手動円形衝突判定の実装

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-5

**説明**:
物理エンジンを使わない、シンプルな円形衝突判定関数を実装する。

**受け入れ基準**:
- [ ] `check_circle_collision(pos1, r1, pos2, r2) -> bool` 関数が実装されている
- [ ] 単体テストで正しく動作することを確認している

**実装ガイド**:
```rust
/// 2つの円が重なっているかチェック（距離の二乗で比較して sqrt を省略）
pub fn check_circle_collision(pos1: Vec2, r1: f32, pos2: Vec2, r2: f32) -> bool {
    let dist_sq = pos1.distance_squared(pos2);
    let radius_sum = r1 + r2;
    dist_sq < radius_sum * radius_sum
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_collision_hit() {
        assert!(check_circle_collision(Vec2::ZERO, 10.0, Vec2::new(15.0, 0.0), 10.0));
    }
    #[test]
    fn test_collision_miss() {
        assert!(!check_circle_collision(Vec2::ZERO, 5.0, Vec2::new(20.0, 0.0), 5.0));
    }
}
```

---

### タスク 5.2: 空間グリッドパーティショニング

**優先度**: P0
**推定工数**: 1.5時間
**ラベル**: task, phase-5

**説明**:
O(n²)の総当たり衝突判定を最適化するため、空間グリッドを実装する。毎フレーム全エンティティをグリッドに登録し、近傍のみを衝突チェック対象とする。

**受け入れ基準**:
- [ ] `SpatialGrid` リソースが実装されている
- [ ] 毎フレーム `update_spatial_grid` システムがグリッドを再構築する
- [ ] `get_nearby_entities(pos, radius)` で近傍エンティティを取得できる
- [ ] 300体の敵でも衝突判定のパフォーマンスが問題ない

**実装ガイド**:
```rust
#[derive(Resource, Default)]
pub struct SpatialGrid {
    pub cell_size: f32,
    pub cells: HashMap<(i32, i32), Vec<Entity>>,
}

impl SpatialGrid {
    pub fn new(cell_size: f32) -> Self {
        Self { cell_size, cells: HashMap::new() }
    }

    pub fn clear(&mut self) {
        self.cells.clear();
    }

    pub fn insert(&mut self, entity: Entity, pos: Vec2) {
        let cell = self.world_to_cell(pos);
        self.cells.entry(cell).or_default().push(entity);
    }

    pub fn get_nearby(&self, pos: Vec2, radius: f32) -> Vec<Entity> {
        let cells_needed = (radius / self.cell_size).ceil() as i32 + 1;
        let center = self.world_to_cell(pos);
        let mut result = Vec::new();
        for dx in -cells_needed..=cells_needed {
            for dy in -cells_needed..=cells_needed {
                let cell = (center.0 + dx, center.1 + dy);
                if let Some(entities) = self.cells.get(&cell) {
                    result.extend_from_slice(entities);
                }
            }
        }
        result
    }

    fn world_to_cell(&self, pos: Vec2) -> (i32, i32) {
        ((pos.x / self.cell_size).floor() as i32,
         (pos.y / self.cell_size).floor() as i32)
    }
}
```

---

### タスク 5.3: ダメージイベントシステム

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-5

**説明**:
ダメージをイベント経由でシステム間を疎結合にして渡すイベント定義と、ダメージ適用システムを実装する。

**受け入れ基準**:
- [ ] `DamageEnemyEvent` が定義されている
- [ ] `PlayerDamagedEvent` が定義されている
- [ ] `apply_damage_to_enemies` システムがイベントを受け取り敵HPを減少させる
- [ ] 敵HPが0以下になると `EnemyDiedEvent` が送信される

---

### タスク 5.4: 弾 vs 敵の衝突判定

**優先度**: P0
**推定工数**: 1時間
**ラベル**: task, phase-5

**説明**:
弾と敵の衝突判定を実装し、ヒット時にダメージイベントを送信する。貫通処理も含む。

**受け入れ基準**:
- [ ] 弾が敵に当たるとダメージイベントが送信される
- [ ] 貫通数（piercing）が0の弾は1体当たると消える
- [ ] 貫通数が1以上の弾は複数の敵を貫通する
- [ ] 同じ敵への連続ヒットを防止する（hit_entitiesリスト）

---

### タスク 5.5: 敵 vs プレイヤーの衝突判定

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-5

**説明**:
敵とプレイヤーの接触判定を実装し、ダメージと無敵時間を付与する。

**受け入れ基準**:
- [ ] 敵がプレイヤーに接触するとプレイヤーがダメージを受ける
- [ ] ダメージ後0.5秒間の無敵時間が付与される
- [ ] 無敵時間中は敵に接触してもダメージを受けない

---

### タスク 5.6: プレイヤー死亡とゲームオーバー遷移

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-5

**説明**:
プレイヤーのHPが0になったらゲームオーバー状態に遷移する。

**受け入れ基準**:
- [ ] プレイヤーHPが0になると `GameOverEvent` が送信される
- [ ] `GameOver` 状態に遷移する
- [ ] 最低限のゲームオーバー画面が表示される（「GAME OVER」テキスト + タイトルへボタン）

---

### タスク 5.7: パフォーマンステスト

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-5, performance

**説明**:
300体以上の敵が同時に存在する状態でパフォーマンスを測定する。

**受け入れ基準**:
- [ ] 300体の敵が同時に存在しても60fps以上を維持する
- [ ] 500体でも許容できるパフォーマンス（50fps以上）
- [ ] 空間グリッドなしの場合と比較してパフォーマンス改善を確認

---

## フェーズ検証

### 検証項目
- [ ] 弾が敵に当たるとダメージが入る（敵の見た目が変化：赤くなる等）
- [ ] 敵のHPが0になると消える
- [ ] 敵がプレイヤーに触れるとHPが減る
- [ ] HPが0でゲームオーバー画面が表示される
- [ ] 300体の敵でも60fps以上を維持

## 次のフェーズ

Phase 5 完了 → 次は **Phase 6: 経験値・レベルアップ** に進む
