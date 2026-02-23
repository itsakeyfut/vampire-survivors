# Phase 6: 経験値・レベルアップシステム

## フェーズ概要

**ステータス**: 🔲 未着手
**推定工数**: 4-6時間
**依存関係**: Phase 5

### 目的
経験値ジェムのドロップ・吸収・レベルアップ・武器/パッシブ選択画面を実装する。これによりゲームの核となるフィードバックループが完成する。

### スコープ
- 経験値ジェム（ExperienceGem）のドロップと磁石吸収
- XPバーとレベル管理（GameData）
- レベルアップ検知と状態遷移（Playing → LevelUp）
- レベルアップ選択肢生成（3択）
- 選択後のアップグレード適用とゲーム再開
- 基本的なHUD更新（HPバー・XPバー）

---

## タスクリスト

### タスク 6.1: 経験値ジェムのドロップ

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-6

**説明**:
`EnemyDiedEvent` を受け取り、死亡位置に経験値ジェムをスポーンする。

**受け入れ基準**:
- [ ] 敵が死亡すると経験値ジェムが位置に出現する
- [ ] ジェムの価値（value）が敵タイプに応じて設定される
- [ ] 大量XPのドロップは複数の小さいジェムに分割される
- [ ] ジェムがプレースホルダースプライト（緑の円）で表示される

---

### タスク 6.2: XPジェムの磁石吸収システム

**優先度**: P0
**推定工数**: 1時間
**ラベル**: task, phase-6

**説明**:
プレイヤーの吸収範囲内（pickup_radius）に入ったジェムをプレイヤーに向かって引き寄せ、プレイヤー位置に到達したら吸収する。

**受け入れ基準**:
- [ ] プレイヤーから `PlayerStats.pickup_radius` 以内のジェムが引き寄せられる
- [ ] ジェムがプレイヤー位置に到達したら `GameData.current_xp` に加算されて消える
- [ ] 吸収が視覚的に分かる（ジェムがプレイヤーに向かって動く）

**実装ガイド**:
```rust
pub fn attract_gems(
    player_query: Query<(&Transform, &PlayerStats), With<Player>>,
    mut gem_query: Query<(Entity, &mut Transform, &ExperienceGem)>,
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
    time: Res<Time>,
) {
    let Ok((player_t, stats)) = player_query.get_single() else { return };
    let player_pos = player_t.translation.truncate();

    for (entity, mut gem_t, gem) in gem_query.iter_mut() {
        let gem_pos = gem_t.translation.truncate();
        let dist = gem_pos.distance(player_pos);

        if dist < stats.pickup_radius {
            let dir = (player_pos - gem_pos).normalize_or_zero();
            let speed = 300.0 + (stats.pickup_radius - dist) * 3.0;
            gem_t.translation += (dir * speed * time.delta_secs()).extend(0.0);

            if dist < 16.0 {
                game_data.current_xp += gem.value;
                commands.entity(entity).despawn();
            }
        }
    }
}
```

---

### タスク 6.3: レベルアップ検知と状態遷移

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-6

**説明**:
XPが必要量を超えたらレベルアップし、`LevelUp` 状態に遷移する。

**受け入れ基準**:
- [ ] `GameData.current_xp >= GameData.xp_to_next_level` でレベルアップする
- [ ] レベルアップ時に `LevelUpEvent` が送信される
- [ ] ゲームが `LevelUp` 状態になってポーズされる（敵・弾が停止）
- [ ] 次のレベルに必要なXPが指数的に増加する

```rust
pub fn calculate_xp_to_next_level(level: u32) -> u32 {
    (20.0 * 1.1_f32.powi(level as i32 - 1)).floor() as u32
}
```

---

### タスク 6.4: レベルアップ選択肢の生成

**優先度**: P0
**推定工数**: 1.5時間
**ラベル**: task, phase-6

**説明**:
プレイヤーの所持武器・パッシブ状況に基づいて3択の選択肢を生成し、`LevelUpChoices` リソースに保存する。

**受け入れ基準**:
- [ ] 所持武器のレベルアップが選択肢に含まれる
- [ ] 武器枠に空きがある場合は新武器が選択肢に含まれる
- [ ] パッシブアイテムのレベルアップ/新規取得が選択肢に含まれる
- [ ] 最大レベルに達したアイテムは選択肢から除外される
- [ ] 3つの選択肢がランダムに選ばれる

---

### タスク 6.5: レベルアップ選択UIの基本実装

**優先度**: P0
**推定工数**: 1時間
**ラベル**: task, phase-6, ui

**説明**:
LevelUp状態中に選択肢を3枚のカードとして表示し、クリックで選択できるようにする。

**受け入れ基準**:
- [ ] 3枚のカードが表示される（名前・効果説明）
- [ ] カードをクリックすると選択が確定する
- [ ] 選択後に `Playing` 状態に戻りゲームが再開する

---

### タスク 6.6: アップグレードの適用

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-6

**説明**:
選択されたアップグレードをプレイヤーエンティティに適用する。

**受け入れ基準**:
- [ ] 武器レベルアップが `WeaponInventory` に反映される
- [ ] 新武器が `WeaponInventory` に追加される（最大6種まで）
- [ ] パッシブアイテムの効果が `PlayerStats` に即座に反映される

---

### タスク 6.7: 基本HUDの実装（HP・XP・タイマー）

**優先度**: P1
**推定工数**: 1時間
**ラベル**: task, phase-6, ui

**説明**:
ゲームプレイ中の基本的なHUDを実装する。

**受け入れ基準**:
- [ ] HPバーが現在HP/最大HPに応じて表示される
- [ ] XPバーが現在XP進捗に応じて表示される
- [ ] タイマーがMM:SS形式で表示される
- [ ] 現在のレベルが表示される

---

## フェーズ検証

### 検証項目
- [ ] 敵を倒すとXPジェムが出る
- [ ] ジェムが磁石のようにプレイヤーに向かって引き寄せられる
- [ ] XPバーが溜まりレベルアップする
- [ ] レベルアップ時にゲームが停止して3択カードが表示される
- [ ] カード選択後にアップグレードが適用される（例: ムチがLv2になる）
- [ ] ゲームが再開する

## 次のフェーズ

Phase 6 完了 → MVPが達成された。次は **Phase 7: 武器拡充** もしくは **Phase 11: UI/HUD** に進む
