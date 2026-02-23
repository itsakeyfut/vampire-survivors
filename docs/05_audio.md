# Vampire Survivors クローン - オーディオ設計書

## 1. オーディオシステム概要

### 1.1 使用ライブラリ
- **bevy_kira_audio**: Bevy 0.17.3対応のオーディオクレート
  - BGMのループ再生・クロスフェード
  - 音量・ピッチのリアルタイム制御
  - 複数チャンネルの同時再生（BGM + SFX）

### 1.2 オーディオチャンネル設計
```rust
#[derive(Resource)]
pub struct AudioChannels {
    pub bgm: AudioChannel<BgmChannel>,
    pub sfx: AudioChannel<SfxChannel>,
    pub ui: AudioChannel<UiChannel>,
}

// チャンネルマーカー型
struct BgmChannel;
struct SfxChannel;
struct UiChannel;
```

---

## 2. BGM（バックグラウンドミュージック）

### 2.1 BGM一覧

| シーン | ファイル名 | ループ | 説明 |
|--------|-----------|--------|------|
| タイトル | `bgm_title.ogg` | ✓ | 不気味で引き込まれるダークファンタジー曲 |
| ゲームプレイ（序盤） | `bgm_gameplay_early.ogg` | ✓ | 緊迫感のある中速の曲 |
| ゲームプレイ（後半） | `bgm_gameplay_late.ogg` | ✓ | 激しくなるアップテンポ曲（15分〜） |
| ボス戦 | `bgm_boss.ogg` | ✓ | 重厚で緊張感のある曲 |
| ゲームオーバー | `bgm_gameover.ogg` | ✗ | 短い悲しいメロディ（数秒） |
| 勝利 | `bgm_victory.ogg` | ✗ | 達成感のあるファンファーレ |

### 2.2 BGM遷移ルール

```rust
/// ゲーム状態に応じてBGMを切り替える
pub fn manage_bgm(
    state: Res<State<AppState>>,
    game_data: Res<GameData>,
    audio: Res<Audio>,
    audio_handles: Res<AudioHandles>,
    mut current_bgm: Local<Option<Handle<AudioSource>>>,
) {
    let target_bgm = match state.get() {
        AppState::Title         => Some(&audio_handles.bgm_title),
        AppState::Playing       => {
            // 30分ボス戦ならボスBGM
            if game_data.boss_spawned {
                Some(&audio_handles.bgm_boss)
            } else if game_data.elapsed_time >= 15.0 * 60.0 {
                Some(&audio_handles.bgm_gameplay_late)
            } else {
                Some(&audio_handles.bgm_gameplay_early)
            }
        }
        AppState::GameOver      => Some(&audio_handles.bgm_gameover),
        AppState::Victory       => Some(&audio_handles.bgm_victory),
        _                       => None,
    };

    // 同じBGMなら切り替えない
    if let Some(bgm) = target_bgm {
        if current_bgm.as_ref() != Some(bgm) {
            audio.play(bgm.clone()).looped();
            *current_bgm = Some(bgm.clone());
        }
    }
}
```

### 2.3 BGMの音量設定

| BGM | 推奨音量 |
|-----|---------|
| タイトル | 0.7 |
| ゲームプレイ | 0.6 |
| ボス戦 | 0.7 |
| ゲームオーバー | 0.5 |
| 勝利 | 0.8 |

---

## 3. SFX（効果音）

### 3.1 SFX一覧

#### 武器攻撃音

| SFX名 | ファイル名 | 説明 |
|--------|-----------|------|
| ムチ攻撃 | `sfx_whip.ogg` | シュッと空気を切る音 |
| 弾発射（小） | `sfx_projectile_small.ogg` | マジックワンド・ナイフ用 |
| 弾発射（大） | `sfx_projectile_large.ogg` | ファイアウォンド・クロス用 |
| 着弾・爆発 | `sfx_explosion.ogg` | ファイアウォンド着弾時 |
| 雷撃 | `sfx_thunder.ogg` | 稲妻の指輪攻撃時 |
| ガーリックオーラ | `sfx_aura_tick.ogg` | ガーリックのダメージtick音 |
| 聖書周回 | （なし、または効果音なし） | - |

#### 敵関連音

| SFX名 | ファイル名 | 説明 |
|--------|-----------|------|
| 敵被ダメージ | `sfx_enemy_hit.ogg` | 敵がダメージを受けた時 |
| 敵死亡（小） | `sfx_enemy_die_small.ogg` | 弱い敵が倒れる音 |
| 敵死亡（大） | `sfx_enemy_die_large.ogg` | 強い敵が倒れる音 |
| ボス出現 | `sfx_boss_spawn.ogg` | 重厚なボス登場SE |
| ボス被ダメージ | `sfx_boss_hit.ogg` | ボスへのダメージ音（重い） |
| ボス死亡 | `sfx_boss_die.ogg` | ボス撃破音 |

#### プレイヤー関連音

| SFX名 | ファイル名 | 説明 |
|--------|-----------|------|
| プレイヤー被ダメージ | `sfx_player_hit.ogg` | プレイヤーがダメージを受けた時 |
| プレイヤー死亡 | `sfx_player_die.ogg` | HP0になった時 |
| HP回復 | `sfx_heal.ogg` | HP回復時 |

#### ゲームイベント音

| SFX名 | ファイル名 | 説明 |
|--------|-----------|------|
| レベルアップ | `sfx_level_up.ogg` | キラキラした上昇音 |
| 武器選択 | `sfx_upgrade_select.ogg` | レベルアップ選択時 |
| 武器進化 | `sfx_weapon_evolve.ogg` | 特別な進化SE（派手な音） |
| XPジェム吸収 | `sfx_gem_pickup.ogg` | ジェムを拾う音（連続再生対応） |
| ゴールド獲得 | `sfx_gold_pickup.ogg` | コインを拾う音 |
| 宝箱開封 | `sfx_treasure_open.ogg` | 宝箱を開ける音 |

#### UI操作音

| SFX名 | ファイル名 | 説明 |
|--------|-----------|------|
| ボタンクリック | `sfx_ui_click.ogg` | 汎用UIクリック音 |
| メニュー遷移 | `sfx_ui_transition.ogg` | 画面切り替え音 |
| ショップ購入 | `sfx_shop_buy.ogg` | ゴールドショップ購入音 |

### 3.2 SFX再生システム

```rust
/// 効果音を再生するシステム
pub fn play_sfx_on_events(
    audio: Res<Audio>,
    audio_handles: Res<AudioHandles>,
    mut weapon_events: EventReader<WeaponFiredEvent>,
    mut enemy_death_events: EventReader<EnemyDiedEvent>,
    mut level_up_events: EventReader<LevelUpEvent>,
    mut treasure_events: EventReader<TreasureOpenedEvent>,
    mut boss_events: EventReader<BossSpawnedEvent>,
    mut player_damage_events: EventReader<PlayerDamagedEvent>,
) {
    // 武器発射音
    for event in weapon_events.read() {
        let handle = match event.weapon_type {
            WeaponType::Whip        => &audio_handles.sfx_whip,
            WeaponType::MagicWand |
            WeaponType::Knife       => &audio_handles.sfx_projectile_small,
            WeaponType::FireWand |
            WeaponType::Cross       => &audio_handles.sfx_projectile_large,
            WeaponType::ThunderRing => &audio_handles.sfx_thunder,
            WeaponType::Garlic      => &audio_handles.sfx_aura_tick,
            _                       => continue,
        };
        audio.play(handle.clone()).with_volume(0.5);
    }

    // 敵死亡音
    for event in enemy_death_events.read() {
        let (handle, volume) = if event.enemy_type == EnemyType::BossDeath {
            (&audio_handles.sfx_boss_die, 0.9)
        } else if matches!(event.enemy_type, EnemyType::Dragon | EnemyType::Demon) {
            (&audio_handles.sfx_enemy_die_large, 0.6)
        } else {
            (&audio_handles.sfx_enemy_die_small, 0.4)
        };
        audio.play(handle.clone()).with_volume(volume);
    }

    // レベルアップ音
    for _ in level_up_events.read() {
        audio.play(audio_handles.sfx_level_up.clone()).with_volume(0.8);
    }

    // 宝箱開封音
    for event in treasure_events.read() {
        let handle = if matches!(event.content, TreasureContent::WeaponEvolution(_)) {
            &audio_handles.sfx_weapon_evolve
        } else {
            &audio_handles.sfx_treasure_open
        };
        audio.play(handle.clone()).with_volume(0.7);
    }

    // ボス出現音
    for _ in boss_events.read() {
        audio.play(audio_handles.sfx_boss_spawn.clone()).with_volume(1.0);
    }

    // プレイヤー被ダメージ音
    for _ in player_damage_events.read() {
        audio.play(audio_handles.sfx_player_hit.clone()).with_volume(0.6);
    }
}
```

### 3.3 SFXの同時再生制限

XPジェム吸収音は大量に同時発生するため制限が必要：

```rust
/// ジェム吸収音の制限（1フレームに1回のみ）
#[derive(Resource, Default)]
pub struct GemPickupSfxCooldown(pub f32);

pub fn play_gem_pickup_sfx(
    mut cooldown: ResMut<GemPickupSfxCooldown>,
    time: Res<Time>,
    audio: Res<Audio>,
    audio_handles: Res<AudioHandles>,
    gem_pickups: EventReader<GemPickedUpEvent>,
) {
    cooldown.0 = (cooldown.0 - time.delta_secs()).max(0.0);

    if !gem_pickups.is_empty() && cooldown.0 <= 0.0 {
        audio.play(audio_handles.sfx_gem_pickup.clone()).with_volume(0.3);
        cooldown.0 = 0.05;  // 50ms間隔でのみ再生
    }
}
```

---

## 4. アセット要件（プレースホルダー対応）

### 4.1 MVP段階のアセット方針
- 初期実装では無音で動作するように設計
- アセットが存在しない場合のフォールバックを実装

```rust
pub fn load_audio_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // アセットが存在しない場合、ロードは失敗するが
    // ゲーム自体は動作し続ける（サウンドなし）
    let handles = AudioHandles {
        bgm_title: asset_server.load("sounds/bgm/bgm_title.ogg"),
        bgm_gameplay_early: asset_server.load("sounds/bgm/bgm_gameplay_early.ogg"),
        // ...
    };
    commands.insert_resource(handles);
}
```

### 4.2 アセットファイル構成

```
assets/
└── sounds/
    ├── bgm/
    │   ├── bgm_title.ogg
    │   ├── bgm_gameplay_early.ogg
    │   ├── bgm_gameplay_late.ogg
    │   ├── bgm_boss.ogg
    │   ├── bgm_gameover.ogg
    │   └── bgm_victory.ogg
    └── sfx/
        ├── weapons/
        │   ├── sfx_whip.ogg
        │   ├── sfx_projectile_small.ogg
        │   ├── sfx_projectile_large.ogg
        │   ├── sfx_explosion.ogg
        │   ├── sfx_thunder.ogg
        │   └── sfx_aura_tick.ogg
        ├── enemies/
        │   ├── sfx_enemy_hit.ogg
        │   ├── sfx_enemy_die_small.ogg
        │   ├── sfx_enemy_die_large.ogg
        │   ├── sfx_boss_spawn.ogg
        │   ├── sfx_boss_hit.ogg
        │   └── sfx_boss_die.ogg
        ├── player/
        │   ├── sfx_player_hit.ogg
        │   ├── sfx_player_die.ogg
        │   └── sfx_heal.ogg
        ├── events/
        │   ├── sfx_level_up.ogg
        │   ├── sfx_upgrade_select.ogg
        │   ├── sfx_weapon_evolve.ogg
        │   ├── sfx_gem_pickup.ogg
        │   ├── sfx_gold_pickup.ogg
        │   └── sfx_treasure_open.ogg
        └── ui/
            ├── sfx_ui_click.ogg
            ├── sfx_ui_transition.ogg
            └── sfx_shop_buy.ogg
```

---

## 5. 音量バランス設計

| カテゴリ | マスター音量 | 備考 |
|---------|------------|------|
| BGM | 0.6 | ゲームプレイの邪魔にならない音量 |
| SFX（武器） | 0.5 | 頻繁に鳴るため控えめに |
| SFX（敵死亡） | 0.4 | 大量の敵が倒れても煩くならない |
| SFX（イベント） | 0.8 | レベルアップ・進化など重要イベントは明確に |
| SFX（UI） | 0.6 | 操作フィードバック用 |
