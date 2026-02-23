# Phase 15: オーディオ統合

## フェーズ概要

**ステータス**: 🔲 未着手
**推定工数**: 4-6時間
**依存関係**: Phase 11

### 目的
bevy_kira_audio を使ってBGMとSFXをゲームに統合する。各シーン・ゲームイベントで適切なサウンドが再生されるようにする。

---

## タスクリスト

### タスク 15.1: bevy_kira_audio のセットアップ

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-15

**説明**:
`vs-audio` クレートに `bevy_kira_audio` を追加し、基本的なオーディオプラグインを設定する。

**受け入れ基準**:
- [ ] `app/audio/Cargo.toml` に `bevy_kira_audio` が追加されている
- [ ] `GameAudioPlugin` が `AudioPlugin` を追加している
- [ ] アプリが正常にビルド・起動できる

---

### タスク 15.2: オーディオアセットのロード

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-15

**説明**:
BGMとSFXのアセットをロードし、`AudioHandles` リソースとして管理する。ファイルが存在しない場合もクラッシュしないようにする。

**受け入れ基準**:
- [ ] `AudioHandles` リソースが `Startup` でロードされる
- [ ] アセットファイルが存在しない場合、ゲームはサウンドなしで動作する
- [ ] 全BGM・SFXのハンドルが管理されている

---

### タスク 15.3: BGM管理システム

**優先度**: P0
**推定工数**: 1時間
**ラベル**: task, phase-15

**説明**:
`AppState` の変化に応じてBGMを切り替えるシステムを実装する。

**受け入れ基準**:
- [ ] `AppState::Title` で `bgm_title` が再生される
- [ ] `AppState::Playing` で `bgm_gameplay_early` が再生される
- [ ] 15分経過後に `bgm_gameplay_late` に切り替わる
- [ ] ボス出現後に `bgm_boss` に切り替わる
- [ ] `AppState::GameOver` で `bgm_gameover` が再生される
- [ ] `AppState::Victory` で `bgm_victory` が再生される
- [ ] 同じBGMが再生中の場合は切り替えない（重複再生防止）
- [ ] BGMはループ再生される

**実装ガイド**:
```rust
pub fn manage_bgm(
    state: Res<State<AppState>>,
    game_data: Res<GameData>,
    audio: Res<Audio>,
    handles: Res<AudioHandles>,
    mut current: Local<Option<String>>,
) {
    let target_key = match state.get() {
        AppState::Title => "title",
        AppState::Playing => {
            if game_data.boss_spawned { "boss" }
            else if game_data.elapsed_time >= 15.0 * 60.0 { "late" }
            else { "early" }
        }
        AppState::GameOver => "gameover",
        AppState::Victory => "victory",
        _ => return,
    };

    if *current == Some(target_key.to_string()) { return; }
    *current = Some(target_key.to_string());

    // 適切なBGMを再生
    let handle = match target_key {
        "title"    => handles.bgm_title.clone(),
        "early"    => handles.bgm_gameplay_early.clone(),
        "late"     => handles.bgm_gameplay_late.clone(),
        "boss"     => handles.bgm_boss.clone(),
        "gameover" => handles.bgm_gameover.clone(),
        "victory"  => handles.bgm_victory.clone(),
        _          => return,
    };
    audio.play(handle).looped();
}
```

---

### タスク 15.4: 武器SFXシステム

**優先度**: P0
**推定工数**: 0.5時間
**ラベル**: task, phase-15

**説明**:
`WeaponFiredEvent` に基づいて武器攻撃音を再生する。

**受け入れ基準**:
- [ ] 各武器タイプに対応したSFXが再生される
- [ ] SFXの音量が適切（武器音は控えめに0.5）

---

### タスク 15.5: 敵死亡・ゲームイベントSFX

**優先度**: P0
**推定工数**: 1時間
**ラベル**: task, phase-15

**説明**:
敵死亡・レベルアップ・宝箱開封・ボス出現等の重要イベントのSFXを実装する。

**受け入れ基準**:
- [ ] 敵死亡音が再生される（敵タイプに応じて大小）
- [ ] レベルアップ音が再生される（明確に）
- [ ] 宝箱開封音が再生される
- [ ] 武器進化時に特別なSFXが再生される
- [ ] ボス出現時に重厚なSFXが再生される
- [ ] プレイヤー被ダメージ音が再生される

---

### タスク 15.6: XPジェム吸収SFX（制限付き）

**優先度**: P1
**推定工数**: 0.5時間
**ラベル**: task, phase-15

**説明**:
XPジェム吸収音は連続で大量発生するため、一定間隔（50ms）でのみ再生する制限を設ける。

**受け入れ基準**:
- [ ] XPジェム吸収音が再生される
- [ ] 50ms間隔でのみ再生される（連続再生が煩くならない）

---

### タスク 15.7: UI操作SFX

**優先度**: P2
**推定工数**: 0.5時間
**ラベル**: task, phase-15

**説明**:
ボタンクリック・画面遷移・ショップ購入等のUISFXを実装する。

**受け入れ基準**:
- [ ] ボタンクリック時に `sfx_ui_click` が再生される
- [ ] 画面遷移時に `sfx_ui_transition` が再生される
- [ ] ゴールドショップ購入時に `sfx_shop_buy` が再生される

---

## フェーズ検証

### 検証項目
- [ ] 各シーンで適切なBGMが再生される
- [ ] 武器攻撃・敵死亡・レベルアップのSFXが鳴る
- [ ] XPジェム吸収音が大量発生しても煩くない
- [ ] 音量バランスが設計書通り（BGM 0.6、武器SFX 0.5等）
- [ ] アセットファイルが存在しない場合もゲームが動作する

## 次のフェーズ

Phase 15 完了 → 次は **Phase 16: エフェクト・ポリッシュ** に進む
