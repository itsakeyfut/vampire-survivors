# Phase 8: パッシブアイテム・武器進化

## フェーズ概要

**ステータス**: 🔲 未着手
**推定工数**: 5-7時間
**依存関係**: Phase 7

### 目的
全9種のパッシブアイテムを実装し、武器進化システムを構築する。ゲームの深みとなる強化システムが完成する。

---

## タスクリスト

### タスク 8.1: パッシブアイテムの定義と効果計算

**優先度**: P0
**推定工数**: 1.5時間
**ラベル**: task, phase-8

**説明**:
`PassiveItemType` enumと `PassiveInventory` コンポーネント、パッシブ効果の計算システムを実装する。

**受け入れ基準**:
- [ ] 全9種のパッシブアイテムが `PassiveItemType` enumに定義されている
- [ ] `apply_passives()` 関数がパッシブ効果を `PlayerStats` に正しく適用する
- [ ] パッシブが変化するたびに `PlayerStats` が再計算される

**実装ガイド**:
```rust
pub fn apply_passives(
    base_stats: &CharacterBaseStats,
    passive_inventory: &PassiveInventory,
) -> PlayerStats {
    let mut stats = PlayerStats::from_base(base_stats);

    for passive in &passive_inventory.items {
        let lv = passive.level as f32;
        match passive.item_type {
            PassiveItemType::Spinach     => stats.damage_multiplier *= 1.0 + 0.1 * lv,
            PassiveItemType::Wings       => stats.move_speed *= 1.0 + 0.1 * lv,
            PassiveItemType::HollowHeart => stats.max_hp *= 1.0 + 0.2 * lv,
            PassiveItemType::Clover      => stats.luck *= 1.0 + 0.1 * lv,
            PassiveItemType::EmptyTome   => stats.cooldown_reduction += 0.08 * lv,
            PassiveItemType::Bracer      => stats.projectile_speed_mult *= 1.0 + 0.1 * lv,
            PassiveItemType::Spellbinder => stats.duration_multiplier *= 1.0 + 0.1 * lv,
            PassiveItemType::Duplicator  => stats.extra_projectiles += passive.level as u32,
            PassiveItemType::Pummarola   => stats.hp_regen += 0.5 * lv,
        }
    }

    stats.cooldown_reduction = stats.cooldown_reduction.min(0.9);
    stats
}
```

---

### タスク 8.2: パッシブアイテムをレベルアップ選択肢に追加

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-8

**説明**:
Phase 6で作成したレベルアップ選択肢生成に、パッシブアイテムの取得・レベルアップを追加する。

**受け入れ基準**:
- [ ] 未取得のパッシブが「新規取得」として選択肢に出る
- [ ] 取得済みのパッシブがLv5未満の場合「レベルアップ」として出る
- [ ] Lv5のパッシブは選択肢から除外される

---

### タスク 8.3: HP再生システム

**優先度**: P1
**推定工数**: 0.5時間
**ラベル**: task, phase-8

**説明**:
`PlayerStats.hp_regen` に基づいて毎秒HPを回復するシステムを実装する（ポマローラ対応）。

**受け入れ基準**:
- [ ] `hp_regen > 0.0` の場合、毎秒 `hp_regen` ポイント回復する
- [ ] 最大HPを超えて回復しない

---

### タスク 8.4: 武器進化条件チェック

**優先度**: P0
**推定工数**: 1時間
**ラベル**: task, phase-8

**説明**:
武器進化の条件（Lv8 + 対応パッシブ所持）をチェックする関数を実装する。

**受け入れ基準**:
- [ ] `can_evolve_weapon(weapon_state, passive_inventory)` が進化可能かチェックする
- [ ] 対応するパッシブの対応テーブルが設計書通りに実装されている
- [ ] 進化可能な武器が存在する場合、宝箱開封時に優先的に進化する

**実装ガイド**:
```rust
pub fn get_evolution_requirement(weapon: WeaponType) -> Option<PassiveItemType> {
    match weapon {
        WeaponType::Whip        => Some(PassiveItemType::HollowHeart),
        WeaponType::MagicWand   => Some(PassiveItemType::EmptyTome),
        WeaponType::Knife       => Some(PassiveItemType::Bracer),
        WeaponType::Garlic      => Some(PassiveItemType::Pummarola),
        WeaponType::Bible       => Some(PassiveItemType::Spellbinder),
        WeaponType::ThunderRing => Some(PassiveItemType::Duplicator),
        _                       => None,
    }
}
```

---

### タスク 8.5: 進化後武器の実装

**優先度**: P0
**推定工数**: 2時間
**ラベル**: task, phase-8

**説明**:
6種の進化後武器を実装する。進化後はベース武器の動作を大幅に強化したバリエーション。

**受け入れ基準**:
- [ ] ブラッディティア（Whip進化）: 範囲2倍・HP吸収効果
- [ ] ホーリーワンド（MagicWand進化）: 全方向発射・無限貫通
- [ ] サウザンドエッジ（Knife進化）: 弾数大幅増
- [ ] ソウルイーター（Garlic進化）: ダメージ3倍・HP回復
- [ ] アンホーリーベスパーズ（Bible進化）: 周回速度2倍・範囲拡大
- [ ] ライトニングリング（ThunderRing進化）: 全敵同時・連鎖

**実装ガイド**:
進化後武器は新しい `WeaponType` variant として定義。`evolved: true` フラグと組み合わせて管理。

---

### タスク 8.6: 進化発動の通知

**優先度**: P1
**推定工数**: 0.5時間
**ラベル**: task, phase-8

**説明**:
武器進化発動時に画面中央で「WEAPON EVOLVED!」等のテキストを短時間表示する。

**受け入れ基準**:
- [ ] 宝箱開封時に武器進化が発動すると画面中央に通知が表示される
- [ ] 数秒後に自動的に消える

---

## フェーズ検証

### 検証項目
- [ ] パッシブアイテムが選択肢に出現する
- [ ] パッシブ取得後にプレイヤーのステータスが変化する（移動速度・攻撃力等）
- [ ] ムチLv8 + ホローハート所持 → 宝箱で進化できる
- [ ] 進化後の武器が強化された挙動をする

## 次のフェーズ

Phase 8 完了 → 次は **Phase 13: 宝箱システム** または **Phase 9: 敵バリエーション** に進む
