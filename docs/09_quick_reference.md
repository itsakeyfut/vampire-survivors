# Vampire Survivors クローン - クイックリファレンス

## クレート構成

### ディレクトリとクレート名の対応

| ディレクトリ | クレート名 | 説明 |
|------------|-----------|------|
| `app/core/` | `vs-core` | コアゲームロジック・ECS・UI |
| `app/audio/` | `vs-audio` | オーディオ（BGM/SFX）管理 |
| `app/assets/` | `vs-assets` | スプライト・フォントアセット管理 |
| `app/vampire-survivors/` | `vs` | メインバイナリ（プラグイン統合） |

## 依存関係グラフ

```
vs (main binary)
    │
    ├─→ vs-core        (コアロジック、他クレートに依存しない)
    ├─→ vs-audio ────→ vs-core
    └─→ vs-assets       (独立、他クレートに依存しない)
```

---

## よく使うコマンド

### ビルド
```bash
# ワークスペース全体をビルド
cargo build

# リリースビルド
cargo build --release

# 特定のクレートのみビルド
cargo build -p vs-core
cargo build -p vs
```

### 実行
```bash
# ゲームを実行
cargo run -p vs

# リリースモードで実行（パフォーマンス確認用）
cargo run -p vs --release
```

### Just コマンド（justfile定義後）
```bash
just run      # ゲーム実行
just dev      # デバッグモードで実行（bevy_inspector等）
just check    # フォーマット + Clippy
just test     # テスト実行
just build    # リリースビルド
```

### テスト
```bash
# 全テスト実行
cargo test

# 特定クレートのテスト
cargo test -p vs-core

# 特定テスト実行
cargo test -p vs-core -- collision::tests
```

---

## クレート間のインポート

### vs (main) から他のクレートを使う
```rust
use vs_core::GameCorePlugin;
use vs_audio::GameAudioPlugin;
use vs_assets::GameAssetsPlugin;
```

### vs-audio から vs-core を使う
```rust
use vs_core::states::AppState;
use vs_core::resources::GameData;
use vs_core::events::{EnemyDiedEvent, LevelUpEvent};
```

---

## 主要コンポーネント早見表

| コンポーネント | クレート | 用途 |
|-------------|---------|------|
| `Player` | vs-core | プレイヤーマーカー |
| `PlayerStats` | vs-core | HP・速度・攻撃力等のステータス |
| `WeaponInventory` | vs-core | 所持武器一覧（最大6種） |
| `PassiveInventory` | vs-core | 所持パッシブ一覧 |
| `Enemy` | vs-core | 敵の基本データ（HP・速度・ダメージ） |
| `EnemyAI` | vs-core | 敵AIタイプと攻撃タイマー |
| `Projectile` | vs-core | 投射体（ダメージ・貫通・寿命） |
| `OrbitWeapon` | vs-core | 周回武器（聖書等） |
| `AuraWeapon` | vs-core | オーラ武器（ガーリック等） |
| `ExperienceGem` | vs-core | XPジェム |
| `GoldCoin` | vs-core | ゴールドコイン |
| `Treasure` | vs-core | 宝箱 |
| `CircleCollider` | vs-core | 円形衝突判定用半径 |
| `InvincibilityTimer` | vs-core | 被ダメージ後の無敵時間 |

## 主要リソース早見表

| リソース | クレート | 用途 |
|---------|---------|------|
| `GameData` | vs-core | 経過時間・レベル・XP・撃破数等 |
| `EnemySpawner` | vs-core | 敵スポーンタイマー・難易度倍率 |
| `TreasureSpawner` | vs-core | 宝箱スポーンタイマー |
| `LevelUpChoices` | vs-core | レベルアップ時の選択肢 |
| `MetaProgress` | vs-core | メタ進行データ（セーブ対象） |
| `SelectedCharacter` | vs-core | 選択中のキャラクタータイプ |
| `SpatialGrid` | vs-core | 衝突判定用空間グリッド |
| `SpriteAssets` | vs-assets | スプライトハンドル集 |
| `FontAssets` | vs-assets | フォントハンドル集 |
| `AudioHandles` | vs-audio | BGM/SFXハンドル集 |

---

## ゲーム状態（AppState）

```
Title → CharacterSelect → Playing ⇄ LevelUp（3択選択でPlayingに戻る）
                            ↕ ESC
                          Paused → Playing（再開）
                                 → Title（タイトルへ）
Playing → GameOver（HP=0）
Playing → Victory（ボス撃破）
Title → MetaShop → Title
```

---

## 武器タイプ一覧

| `WeaponType` | 日本語名 | 攻撃タイプ |
|-------------|---------|---------|
| `Whip` | ムチ | 扇形スイング |
| `MagicWand` | マジックワンド | 追尾弾 |
| `Knife` | ナイフ | 貫通弾 |
| `Garlic` | ガーリック | 継続オーラ |
| `Bible` | 聖書 | 周回体 |
| `ThunderRing` | 稲妻の指輪 | 雷撃 |
| `Cross` | クロス | ブーメラン |
| `FireWand` | ファイアウォンド | 火球 |

---

## 武器進化チャート

| 基本武器 | + パッシブ | → 進化後 |
|---------|---------|---------|
| Whip (Lv8) | HollowHeart | BloodyTear |
| MagicWand (Lv8) | EmptyTome | HolyWand |
| Knife (Lv8) | Bracer | ThousandEdge |
| Garlic (Lv8) | Pummarola | SoulEater |
| Bible (Lv8) | Spellbinder | UnholyVespers |
| ThunderRing (Lv8) | Duplicator | LightningRing |

---

## パッシブアイテム一覧

| `PassiveItemType` | 日本語名 | 主な効果 | 武器進化用途 |
|-----------------|---------|---------|-----------|
| `Spinach` | スピナッチ | 攻撃力+10%/Lv | - |
| `Wings` | ウィング | 移動速度+10%/Lv | - |
| `HollowHeart` | ホローハート | 最大HP+20%/Lv | Whip進化 |
| `Clover` | クローバー | ラック+10%/Lv | - |
| `EmptyTome` | エンプティトーム | CD-8%/Lv | MagicWand進化 |
| `Bracer` | ブレイサー | 弾速+10%/Lv | Knife進化 |
| `Spellbinder` | スペルバインダー | 持続+10%/Lv | Bible進化 |
| `Duplicator` | デュプリケーター | 発射数+1/Lv | ThunderRing進化 |
| `Pummarola` | ポマローラ | HP再生+0.5/s/Lv | Garlic進化 |

---

## 敵一覧

| `EnemyType` | 日本語名 | 出現開始 |
|------------|---------|---------|
| `Bat` | コウモリ | 0分〜 |
| `Skeleton` | スケルトン | 0分〜 |
| `Zombie` | ゾンビ | 5分〜 |
| `Ghost` | ゴースト | 10分〜 |
| `Demon` | デーモン | 15分〜 |
| `Medusa` | メデューサ | 20分〜 |
| `Dragon` | ドラゴン | 25分〜 |
| `BossDeath` | ボスデス | 30分（ボス） |

---

## 衝突半径早見表

| エンティティ | 半径 |
|------------|------|
| プレイヤー | 12px |
| 弾（小） | 5px |
| 弾（大） | 10px |
| XP吸収範囲 | 80px（強化可能） |
| XPジェム | 6px |
| 宝箱 | 20px |

---

## トラブルシューティング

### ビルドエラー: クレートが見つからない
```bash
rm Cargo.lock
cargo build
```

### 依存関係のエラー
```bash
cargo clean
cargo build
```

### クレート名変更後のエラー
- `Cargo.toml` の `[workspace.members]` を確認
- 各クレートの `Cargo.toml` の `name` を確認
- `main.rs` の `use` 文を確認

### パフォーマンスが悪い（開発中）
```toml
# Cargo.toml に追加済みのはず
[profile.dev.package."*"]
opt-level = 3  # 依存クレートを最適化
```

---

## ファイル構成チェックリスト

- [ ] `Cargo.toml` (workspace): members と workspace.dependencies が正しい
- [ ] `app/core/Cargo.toml`: name = "vs-core"
- [ ] `app/audio/Cargo.toml`: name = "vs-audio", vs-core への依存がある
- [ ] `app/assets/Cargo.toml`: name = "vs-assets"
- [ ] `app/vampire-survivors/Cargo.toml`: name = "vs", 全クレートへの依存がある
- [ ] `app/vampire-survivors/src/main.rs`: 全プラグインを追加している
