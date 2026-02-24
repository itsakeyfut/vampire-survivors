# Phase 12: キャラクター選択画面

## フェーズ概要

**ステータス**: 🔲 未着手
**推定工数**: 3-4時間
**依存関係**: Phase 11

### 目的
キャラクター選択画面を実装する。解放済みキャラクターのみ選択可能で、未解放はロック表示。選択したキャラクターの初期ステータスでゲームを開始する。

---

## タスクリスト

### タスク 12.1: キャラクタータイプと基本ステータス定義

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-12

**説明**:
各キャラクタータイプの基本ステータスと初期武器を定義する。

**受け入れ基準**:
- [ ] `CharacterType` enumに4種（DefaultChar, Magician, Thief, Knight）が定義されている
- [ ] `CharacterBaseStats` 構造体が定義されている
- [ ] 各キャラクターのベースHPドメイン・速度・初期武器・特徴が設定されている

**実装ガイド**:
```rust
pub struct CharacterBaseStats {
    pub max_hp: f32,
    pub move_speed: f32,
    pub starting_weapon: WeaponType,
    pub damage_multiplier: f32,
    pub cooldown_reduction: f32,
    pub name: &'static str,
    pub description: &'static str,
}

pub fn get_character_stats(char_type: CharacterType) -> CharacterBaseStats {
    match char_type {
        CharacterType::DefaultChar => CharacterBaseStats {
            max_hp: 100.0,
            move_speed: 200.0,
            starting_weapon: WeaponType::Whip,
            damage_multiplier: 1.0,
            cooldown_reduction: 0.0,
            name: "デフォルト",
            description: "バランスの取れた万能型",
        },
        CharacterType::Magician => CharacterBaseStats {
            max_hp: 80.0,
            move_speed: 200.0,
            starting_weapon: WeaponType::MagicWand,
            damage_multiplier: 1.0,
            cooldown_reduction: 0.1,
            name: "マジシャン",
            description: "クールダウン-10% / 初期武器: マジックワンド",
        },
        // ... 他のキャラクター
    }
}
```

---

### タスク 12.2: キャラクター選択UI

**優先度**: P0
**推定工数**: 1.5時間
**ラベル**: task, phase-12, ui

**説明**:
設計書通りのキャラクター選択画面を実装する。カード一覧・詳細パネル・選択ボタン。

**実装ファイル**: `app/ui/src/character_select.rs`（**vs-ui** クレート）

**受け入れ基準**:
- [ ] 全キャラクターがカード形式で表示される
- [ ] 解放済みキャラクターは通常表示・選択可能
- [ ] 未解放キャラクターはロックアイコン付き・グレーアウト表示
- [ ] カード選択で詳細パネルが更新される（HP・速度・初期武器・説明）
- [ ] 「このキャラで開始」ボタンでゲーム開始
- [ ] 「戻る」でタイトルへ

---

### タスク 12.3: 選択キャラクターの適用

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-12

**説明**:
選択したキャラクターのステータスでプレイヤーをスポーンし、初期武器を設定する。

**受け入れ基準**:
- [ ] `SelectedCharacter` リソースが選択したキャラクタータイプで更新される
- [ ] ゲーム開始時に `SelectedCharacter` に基づいてプレイヤーステータスが設定される
- [ ] 初期武器が正しく `WeaponInventory` に追加される

---

### タスク 12.4: 未解放キャラクターのロック状態管理

**優先度**: P1
**推定工数**: 0.5時間
**ラベル**: task, phase-12

**説明**:
`MetaProgress.unlocked_characters` を参照してキャラクターのロック/解放状態を判定する。

**受け入れ基準**:
- [ ] デフォルトキャラクターは常に解放済み
- [ ] 他のキャラクターは `MetaProgress.unlocked_characters` に含まれる場合のみ選択可能
- [ ] 未解放のキャラクターカードに「解放には XXX G 必要」が表示される

---

## フェーズ検証

### 検証項目
- [ ] タイトルから「ゲームスタート」でキャラクター選択画面に遷移する
- [ ] デフォルトキャラクターが選択できる
- [ ] マジシャン等の未解放キャラクターがロック表示される
- [ ] キャラクター選択後、正しいステータス・初期武器でゲームが始まる

## 次のフェーズ

Phase 12 完了 → 次は **Phase 13: 宝箱システム** に進む
