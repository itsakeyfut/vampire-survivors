# Phase 14: メタ進行（ゴールドショップ）

## フェーズ概要

**ステータス**: 🔲 未着手
**推定工数**: 5-7時間
**依存関係**: Phase 13

### 目的
ゲーム間をまたぐメタ進行システムを実装する。ゴールドを永続的に保存し、ゴールドショップでキャラクター解放やパーマネントアップグレードを購入できるようにする。

---

## タスクリスト

### タスク 14.1: MetaProgressのセーブ/ロードシステム

**優先度**: P0
**推定工数**: 1時間
**ラベル**: task, phase-14

**説明**:
`MetaProgress` をJSONファイルに保存・ロードするシステムを実装する。

**受け入れ基準**:
- [ ] `MetaProgress` が `serde::Serialize/Deserialize` を実装している
- [ ] `save/meta.json` に保存できる
- [ ] アプリ起動時に `save/meta.json` から読み込める
- [ ] ファイルが存在しない場合はデフォルト値でスタートする
- [ ] ゲームオーバー・勝利・ショップ購入後に自動保存される

**実装ガイド**:
```rust
#[derive(Resource, Serialize, Deserialize, Default)]
pub struct MetaProgress {
    pub total_gold: u32,
    pub unlocked_characters: Vec<CharacterType>,
    pub purchased_upgrades: Vec<MetaUpgradeType>,
}

impl MetaProgress {
    pub fn load() -> Self {
        std::fs::read_to_string("save/meta.json")
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write("save/meta.json", json);
        }
    }
}
```

---

### タスク 14.2: ゴールドのゲーム間引き継ぎ

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-14

**説明**:
ゲームオーバー・勝利時に獲得ゴールドを `MetaProgress.total_gold` に加算して保存する。

**受け入れ基準**:
- [ ] `GameOverEvent` 受信時に `GameData.gold_earned` を `MetaProgress.total_gold` に加算する
- [ ] `VictoryEvent` 受信時も同様に加算する
- [ ] 加算後に `MetaProgress.save()` が呼ばれる

---

### タスク 14.3: ゴールドショップUIの実装

**優先度**: P0
**推定工数**: 2時間
**ラベル**: task, phase-14, ui

**説明**:
設計書通りのゴールドショップ画面を実装する。タイトル画面から遷移可能。

**受け入れ基準**:
- [ ] タイトルの「ゴールドショップ」ボタンで `MetaShop` 状態に遷移する
- [ ] 所持ゴールドが画面右上に表示される
- [ ] キャラクター解放セクション（マジシャン・シーフ・ナイト）が表示される
- [ ] パーマネントアップグレードセクション（HP・XP・ゴールドボーナス等）が表示される
- [ ] 購入済みアイテムはグレーアウト表示
- [ ] ゴールド不足のアイテムは購入ボタンを押せない状態
- [ ] 「閉じる」でタイトルに戻る

---

### タスク 14.4: キャラクター解放の購入ロジック

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-14

**説明**:
ゴールドショップでキャラクター解放ボタンを押した時の購入処理を実装する。

**受け入れ基準**:
- [ ] 購入ボタンクリックで `MetaProgress.total_gold` からゴールドを消費する
- [ ] `MetaProgress.unlocked_characters` にキャラクタータイプを追加する
- [ ] 保存が実行される
- [ ] UIが即座に更新される（購入済み表示に変わる）

---

### タスク 14.5: パーマネントアップグレードの実装

**優先度**: P0
**推定工数**: 1時間
**ラベル**: task, phase-14

**説明**:
パーマネントアップグレード（HP・速度・XP・ゴールドボーナス等）の購入処理と、ゲームへの反映を実装する。

**受け入れ基準**:
- [ ] 各アップグレードの購入処理が実装されている
- [ ] 購入済みアップグレードが `MetaProgress.purchased_upgrades` に記録される
- [ ] ゲーム開始時に購入済みアップグレードがプレイヤーステータスに適用される

**実装ガイド**:
```rust
pub fn apply_meta_upgrades(
    meta: &MetaProgress,
    base_stats: &mut CharacterBaseStats,
) {
    for upgrade in &meta.purchased_upgrades {
        match upgrade {
            MetaUpgradeType::BonusHp         => base_stats.max_hp += 10.0,
            MetaUpgradeType::BonusSpeed       => base_stats.move_speed *= 1.05,
            MetaUpgradeType::XpBonus         => { /* XP multiplier に適用 */ }
            MetaUpgradeType::GoldBonus       => { /* gold drop chance 増加 */ }
            MetaUpgradeType::ExtraChoice     => { /* 選択肢4択に */ }
        }
    }
}
```

---

### タスク 14.6: ゴールド表示のHUD追加

**優先度**: P1
**推定工数**: 0.5時間
**ラベル**: task, phase-14, ui

**説明**:
ゲームプレイ中の獲得ゴールドをHUDに表示する。

**受け入れ基準**:
- [ ] 現在セッションで獲得したゴールドがHUDに表示される
- [ ] ゴールド獲得時にHUDが更新される

---

## フェーズ検証

### 検証項目
- [ ] ゲームオーバー後にゴールドが保存されて次回起動時も残っている
- [ ] ゴールドショップでマジシャンを解放できる
- [ ] 解放後にキャラクター選択画面でマジシャンが選べる
- [ ] パーマネントアップグレード購入後にゲーム内で効果が出る
- [ ] セーブファイルが `save/meta.json` に作成される

## 次のフェーズ

Phase 14 完了 → 次は **Phase 15: オーディオ** に進む
