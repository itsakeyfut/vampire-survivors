# Phase 10: ボスシステム

## フェーズ概要

**ステータス**: 🔲 未着手
**推定工数**: 4-6時間
**依存関係**: Phase 9

### 目的
30分後にボス「デス（Death）」が出現し、3フェーズで戦えるシステムを実装する。ボス撃破で勝利画面へ遷移する。

---

## タスクリスト

### タスク 10.1: ボス出現タイミングシステム

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-10

**説明**:
30分経過後にボスを出現させ、通常の敵スポーンを停止する。

**受け入れ基準**:
- [ ] `GameData.elapsed_time >= 30 * 60` でボス出現フラグが立つ
- [ ] `EnemySpawner.active = false` で通常の敵スポーンが停止する
- [ ] `BossSpawnedEvent` が送信される
- [ ] プレイヤーの画面に「BOSS APPROACHING」等の警告テキストが表示される

---

### タスク 10.2: ボスエンティティのスポーン

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-10

**説明**:
ボスデス（Death）エンティティをスポーンする。

**受け入れ基準**:
- [ ] ボスが画面外（上方）からスポーンする
- [ ] 大型のスプライト（プレースホルダー: 大きな赤い円）
- [ ] HP: 5000（難易度倍率なし）、速度: 30
- [ ] `BossPhase::Phase1` 状態でスポーン

---

### タスク 10.3: ボスAI - フェーズ1（HP 100%〜60%）

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-10

**説明**:
フェーズ1のボスAIを実装する。低速でプレイヤーを追跡。

**受け入れ基準**:
- [ ] HP 100%〜60%の間、速度30でプレイヤーを追跡
- [ ] 接触でプレイヤーに50ダメージ
- [ ] ボスHPバーがHUDに表示される

---

### タスク 10.4: ボスAI - フェーズ2（HP 60%〜30%）

**優先度**: P0
**推定工数**: 1時間
**ラベル**: task, phase-10

**説明**:
HP60%でフェーズ2に移行。移動速度増加と分身スポーン。

**受け入れ基準**:
- [ ] HP 60%以下で移動速度が1.5倍（45px/s）になる
- [ ] フェーズ移行時にミニデス（Mini Death）を3体スポーンする
- [ ] ミニデスはHP低め・速度普通のミニボス相当

**実装ガイド**:
```rust
pub fn update_boss_phase(
    mut boss_query: Query<(&mut Enemy, &mut EnemyAI), With<BossDeath>>,
    mut commands: Commands,
    boss_pos: Query<&Transform, With<BossDeath>>,
) {
    let Ok((mut enemy, mut ai)) = boss_query.get_single_mut() else { return };
    let hp_ratio = enemy.current_hp / enemy.max_hp;
    let new_phase = get_boss_phase(hp_ratio);

    // フェーズ変化を検出して処理
    if new_phase != ai.current_phase {
        match new_phase {
            BossPhase::Phase2 => {
                enemy.move_speed = 45.0;
                spawn_mini_deaths(&mut commands, /* boss_pos */ 3);
            }
            BossPhase::Phase3 => {
                enemy.move_speed = 60.0;
                spawn_mini_deaths(&mut commands, /* boss_pos */ 5);
            }
            _ => {}
        }
        ai.current_phase = new_phase;
    }
}
```

---

### タスク 10.5: ボスAI - フェーズ3（HP 30%〜0%）

**優先度**: P0
**推定工数**: 1時間
**ラベル**: task, phase-10

**説明**:
HP30%でフェーズ3に移行。さらに速度増加と鎌の遠距離攻撃を追加。

**受け入れ基準**:
- [ ] HP 30%以下で移動速度が2倍（60px/s）になる
- [ ] 分身が5体に増加する
- [ ] 3秒ごとに鎌（投射体）をプレイヤー方向に発射する

---

### タスク 10.6: ボス撃破と勝利画面遷移

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-10

**説明**:
ボスのHPが0になったら `VictoryEvent` を送信し、勝利画面に遷移する。

**受け入れ基準**:
- [ ] ボス死亡時に `VictoryEvent` が送信される
- [ ] `Victory` 状態に遷移する
- [ ] 勝利画面に「YOU WIN!」とクリアタイム・統計が表示される

---

### タスク 10.7: ボスHPバーのHUD

**優先度**: P1
**推定工数**: 0.5時間
**ラベル**: task, phase-10

**説明**:
ボス出現後、画面上部にボスHPバーを追加表示する。

**受け入れ基準**:
- [ ] ボス出現後、上部HUDにボスHPバーが表示される
- [ ] ボスHPが減るに従いバーが短くなる
- [ ] ボスの名前「DEATH」が表示される

---

## フェーズ検証

### 検証項目
- [ ] 30分後にボスが出現する（通常の敵スポーンが停止）
- [ ] HP60%でボスが速くなり分身が出る
- [ ] HP30%でさらに速くなり分身が増え鎌を投げてくる
- [ ] ボスを倒すと勝利画面が表示される
- [ ] ボスHPバーがHUDに正しく表示される

## 次のフェーズ

Phase 10 完了 → 次は **Phase 11: UI/HUD実装** に進む
