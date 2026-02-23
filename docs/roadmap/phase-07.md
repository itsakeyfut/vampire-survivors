# Phase 7: 武器拡充（全8種実装）

## フェーズ概要

**ステータス**: 🔲 未着手
**推定工数**: 6-8時間
**依存関係**: Phase 6

### 目的
残り6種の武器（ナイフ・ガーリック・聖書・稲妻の指輪・クロス・ファイアウォンド）を実装し、全8種の武器が動作する状態にする。

---

## タスクリスト

### タスク 7.1: ナイフ（Knife）の実装

**優先度**: P0
**推定工数**: 1時間
**ラベル**: task, phase-7

**説明**:
プレイヤーの移動方向に高速で貫通弾を発射する武器を実装する。

**受け入れ基準**:
- [ ] プレイヤーの移動方向（または最後の移動方向）に向けて弾を発射する
- [ ] 高速（弾速600〜1000）で移動する
- [ ] デフォルトで貫通（piercing）が無限
- [ ] 複数の弾数に対応している（Lv3以上）

**実装ガイド**:
移動方向を `LastMoveDirection` コンポーネントとしてプレイヤーに持たせる。

---

### タスク 7.2: ガーリック（Garlic）の実装

**優先度**: P0
**推定工数**: 1時間
**ラベル**: task, phase-7

**説明**:
プレイヤー周囲の円形範囲で継続ダメージを与えるオーラ武器を実装する。

**受け入れ基準**:
- [ ] プレイヤーを中心に `AuraWeapon` エンティティがアタッチされる
- [ ] 一定間隔（tick_interval）で範囲内の敵に継続ダメージを与える
- [ ] 視覚的にオーラの円が表示される（半透明の紫/白の円）
- [ ] `PlayerStats.area_multiplier` で範囲がスケールする

**実装ガイド**:
```rust
pub fn update_aura_weapons(
    mut aura_query: Query<(&mut AuraWeapon, &Transform)>,
    enemy_query: Query<(Entity, &Transform, &CircleCollider), With<Enemy>>,
    mut damage_events: EventWriter<DamageEnemyEvent>,
    time: Res<Time>,
) {
    for (mut aura, transform) in aura_query.iter_mut() {
        aura.tick_timer -= time.delta_secs();
        if aura.tick_timer > 0.0 { continue; }
        aura.tick_timer = aura.tick_interval;

        let aura_pos = transform.translation.truncate();
        for (enemy_entity, enemy_t, collider) in enemy_query.iter() {
            if check_circle_collision(aura_pos, aura.radius, enemy_t.translation.truncate(), collider.radius) {
                damage_events.send(DamageEnemyEvent { entity: enemy_entity, damage: aura.damage, weapon_type: WeaponType::Garlic });
            }
        }
    }
}
```

---

### タスク 7.3: 聖書（Bible）の実装

**優先度**: P0
**推定工数**: 1.5時間
**ラベル**: task, phase-7

**説明**:
プレイヤーの周りを一定半径で周回するオービット武器を実装する。接触した敵にダメージを与える。

**受け入れ基準**:
- [ ] `OrbitWeapon` エンティティがプレイヤー周囲を周回する
- [ ] 周回半径・速度・数がレベルに応じて変化する
- [ ] 同じ敵への連続ヒットを防ぐクールダウンがある
- [ ] 視覚的に周回体が表示される（聖書アイコン）

**実装ガイド**:
```rust
pub fn update_orbit_weapons(
    player_query: Query<&Transform, With<Player>>,
    mut orbit_query: Query<(&mut OrbitWeapon, &mut Transform), Without<Player>>,
    time: Res<Time>,
) {
    let Ok(player_t) = player_query.get_single() else { return };
    let player_pos = player_t.translation.truncate();

    for (mut orbit, mut transform) in orbit_query.iter_mut() {
        orbit.orbit_angle += orbit.orbit_speed * time.delta_secs();
        let x = orbit.orbit_radius * orbit.orbit_angle.cos();
        let y = orbit.orbit_radius * orbit.orbit_angle.sin();
        transform.translation = (player_pos + Vec2::new(x, y)).extend(transform.translation.z);
    }
}
```

---

### タスク 7.4: 稲妻の指輪（Thunder Ring）の実装

**優先度**: P0
**推定工数**: 1時間
**ラベル**: task, phase-7

**説明**:
画面内のランダムな敵に雷を落とす武器を実装する。

**受け入れ基準**:
- [ ] クールダウン後、ランダムな敵位置に雷エフェクトと共にダメージを与える
- [ ] 雷撃数（count）が複数の場合は異なる敵を選ぶ
- [ ] 視覚的なエフェクト（短命な雷スプライト）が表示される

---

### タスク 7.5: クロス（Cross）の実装

**優先度**: P0
**推定工数**: 1時間
**ラベル**: task, phase-7

**説明**:
前方に投擲し、一定距離で折り返してブーメランのように戻ってくる投射体武器を実装する。

**受け入れ基準**:
- [ ] プレイヤーの移動方向（または直前の移動方向）に投擲される
- [ ] 最大射程に達したら方向が反転してプレイヤーに戻る
- [ ] 往路・復路どちらでも敵にダメージを与える

**実装ガイド**:
`Projectile` に `returning: bool` と `initial_pos: Vec2` フィールドを追加し、投擲距離で方向を反転させる。

---

### タスク 7.6: ファイアウォンド（Fire Wand）の実装

**優先度**: P0
**推定工数**: 1時間
**ラベル**: task, phase-7

**説明**:
現在画面内で最大HPを持つ敵に向けて大型火球を発射し、着弾時に範囲爆発させる武器を実装する。

**受け入れ基準**:
- [ ] 最大HPの敵を追尾弾（発射時に方向固定）として火球を発射する
- [ ] 着弾（ライフタイム切れ時）に周囲の敵にも範囲ダメージを与える
- [ ] 大きめのスプライトで視覚的に分かりやすい

---

### タスク 7.7: 武器レベルアップパラメータテーブル

**優先度**: P1
**推定工数**: 1時間
**ラベル**: task, phase-7

**説明**:
各武器のレベル1〜8のパラメータテーブルを実装する。`WeaponState.level` に応じて正しい数値を返す関数。

**受け入れ基準**:
- [ ] 全8武器のLv1〜8のパラメータが設計書通りに実装されている
- [ ] `WeaponState::base_damage()`, `base_cooldown()`, `base_count()` 等が正しい値を返す

---

## フェーズ検証

### 検証項目
- [ ] 全8種の武器がレベルアップ選択肢に出現する
- [ ] 各武器の攻撃パターンが設計書通りに動作する
- [ ] 武器を複数所持して同時に攻撃できる（最大6種）
- [ ] 武器レベルアップ時にパラメータが変化する

## 次のフェーズ

Phase 7 完了 → 次は **Phase 8: パッシブアイテム・武器進化** に進む
