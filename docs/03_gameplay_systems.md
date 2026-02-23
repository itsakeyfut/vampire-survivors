# Vampire Survivors クローン - ゲームプレイシステム設計書

## 1. 武器システム詳細

### 1.1 武器の基本データ構造

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeaponType {
    Whip,        // ムチ
    MagicWand,   // マジックワンド
    Knife,       // ナイフ
    Garlic,      // ガーリック
    Bible,       // 聖書
    ThunderRing, // 稲妻の指輪
    Cross,       // クロス
    FireWand,    // ファイアウォンド
}

/// 武器のベースパラメータ（レベル1時の値）
pub struct WeaponBaseParams {
    pub base_damage: f32,
    pub base_cooldown: f32,    // 秒
    pub base_count: u32,       // 発射数/周回数
    pub base_range: f32,       // 射程/範囲
    pub base_speed: f32,       // 弾速
    pub base_duration: f32,    // 弾の存在時間
    pub base_piercing: u32,    // 貫通数
}
```

### 1.2 各武器の詳細仕様

#### ムチ (Whip)
**攻撃タイプ**: 扇形スイング（瞬間判定）

```
動作:
- 1秒ごとに左右交互に扇形攻撃
- 範囲: プレイヤーから横方向90度
- 即時ダメージ判定（弾なし）

レベル別パラメータ:
Lv1: ダメージ10、範囲100px、クールダウン1.0s
Lv2: ダメージ10、範囲110px、クールダウン1.0s
Lv3: ダメージ15、範囲110px、クールダウン1.0s
Lv4: ダメージ15、範囲120px、クールダウン0.8s
Lv5: ダメージ20、範囲120px、クールダウン0.8s、弾数+1（両側同時）
Lv6: ダメージ20、範囲140px、クールダウン0.7s
Lv7: ダメージ25、範囲140px、クールダウン0.7s
Lv8: ダメージ25、範囲160px、クールダウン0.6s、弾数+1
```

**実装:**
```rust
fn fire_whip(
    player_pos: Vec2,
    player_stats: &PlayerStats,
    weapon_state: &WeaponState,
    whip_side: &mut WhipSide,
    nearby_enemies: &[Entity],
    enemy_transforms: &Query<&Transform, With<Enemy>>,
    damage_events: &mut EventWriter<DamageEnemyEvent>,
) {
    let effective_range = WHIP_BASE_RANGE
        * weapon_state.range_mult()
        * player_stats.area_multiplier;

    let direction = if *whip_side == WhipSide::Left { -1.0 } else { 1.0 };

    for &enemy_entity in nearby_enemies {
        let Ok(transform) = enemy_transforms.get(enemy_entity) else { continue };
        let enemy_pos = transform.translation.truncate();
        let relative = enemy_pos - player_pos;

        // 扇形判定: 横方向（X）が前面、指定距離以内
        if relative.x * direction > 0.0
            && relative.length() < effective_range
            && relative.y.abs() < effective_range * 0.6
        {
            damage_events.send(DamageEnemyEvent {
                entity: enemy_entity,
                damage: weapon_state.damage(player_stats),
                weapon_type: WeaponType::Whip,
            });
        }
    }

    // 次回は反対側を攻撃
    *whip_side = whip_side.flip();
}
```

---

#### マジックワンド (Magic Wand)
**攻撃タイプ**: 追尾弾

```
動作:
- 最も近い敵に向かって弾を発射
- 弾は直進（ホーミングなし、発射時の方向に固定）
- 貫通なし（デフォルト）

レベル別パラメータ:
Lv1: ダメージ20、弾数1、クールダウン0.5s、弾速300
Lv2: ダメージ20、弾数2、クールダウン0.5s、弾速300
Lv3: ダメージ30、弾数2、クールダウン0.5s、弾速300
Lv4: ダメージ30、弾数2、クールダウン0.4s、弾速350
Lv5: ダメージ40、弾数3、クールダウン0.4s、弾速350
Lv6: ダメージ40、弾数3、クールダウン0.35s、弾速350
Lv7: ダメージ50、弾数3、クールダウン0.35s、弾速400、貫通+1
Lv8: ダメージ50、弾数4、クールダウン0.3s、弾速400、貫通+1
```

**実装:**
```rust
fn fire_magic_wand(
    commands: &mut Commands,
    player_pos: Vec2,
    player_stats: &PlayerStats,
    weapon_state: &WeaponState,
    enemies: &[(Entity, Vec2)],  // (entity, position)のリスト
) {
    // 最も近い敵を弾数分探す
    let count = (weapon_state.base_count + player_stats.extra_projectiles) as usize;
    let targets = find_nearest_enemies(player_pos, enemies, count);

    for (_, target_pos) in targets {
        let direction = (target_pos - player_pos).normalize_or_zero();
        let speed = weapon_state.speed(player_stats);

        commands.spawn((
            Projectile {
                damage: weapon_state.damage(player_stats),
                piercing: weapon_state.piercing(),
                hit_enemies: vec![],
                lifetime: 5.0,  // 最大5秒で消える
                weapon_type: WeaponType::MagicWand,
            },
            ProjectileVelocity(direction * speed),
            CircleCollider { radius: 6.0 },
            SpriteBundle {
                // 水色の弾スプライト
                ..default()
            },
            Transform::from_translation(player_pos.extend(1.0)),
        ));
    }
}
```

---

#### ナイフ (Knife)
**攻撃タイプ**: 貫通弾（移動方向に発射）

```
動作:
- プレイヤーの移動方向に高速で弾を発射
- 移動していない場合は最後の移動方向を使用（デフォルト: 右）
- 高い貫通数（デフォルトで無限貫通）

レベル別パラメータ:
Lv1: ダメージ15、弾数1、クールダウン0.3s、弾速600
Lv2: ダメージ15、弾数1、クールダウン0.25s、弾速700
Lv3: ダメージ20、弾数2、クールダウン0.25s、弾速700
Lv4: ダメージ20、弾数2、クールダウン0.2s、弾速800
Lv5: ダメージ25、弾数3、クールダウン0.2s、弾速800
Lv6: ダメージ25、弾数3、クールダウン0.18s、弾速900
Lv7: ダメージ30、弾数4、クールダウン0.18s、弾速900
Lv8: ダメージ30、弾数5、クールダウン0.15s、弾速1000
```

---

#### ガーリック (Garlic)
**攻撃タイプ**: 継続オーラ（プレイヤー中心の円形範囲）

```
動作:
- プレイヤーの周囲にオーラを常時展開
- 範囲内の敵に0.5秒ごとにダメージ
- 視覚的にはプレイヤー周囲に半透明の円

レベル別パラメータ:
Lv1: ダメージ5/tick、範囲80px、tickInterval0.5s
Lv2: ダメージ5/tick、範囲90px、tickInterval0.5s
Lv3: ダメージ8/tick、範囲90px、tickInterval0.5s
Lv4: ダメージ8/tick、範囲100px、tickInterval0.45s
Lv5: ダメージ10/tick、範囲110px、tickInterval0.4s
Lv6: ダメージ12/tick、範囲120px、tickInterval0.4s
Lv7: ダメージ15/tick、範囲130px、tickInterval0.35s
Lv8: ダメージ20/tick、範囲150px、tickInterval0.3s
```

---

#### 聖書 (Bible)
**攻撃タイプ**: 周回体（プレイヤーの周りを周回）

```
動作:
- プレイヤーの周りを一定半径で周回
- 接触した敵に継続ダメージ
- 同じ敵への連続ヒット防止のためクールダウン管理

レベル別パラメータ:
Lv1: ダメージ25、周回数1、半径80px、速度2.5rad/s
Lv2: ダメージ25、周回数2、半径80px、速度2.5rad/s
Lv3: ダメージ35、周回数2、半径90px、速度3.0rad/s
Lv4: ダメージ35、周回数2、半径90px、速度3.5rad/s
Lv5: ダメージ45、周回数3、半径100px、速度3.5rad/s
Lv6: ダメージ45、周回数3、半径110px、速度4.0rad/s
Lv7: ダメージ55、周回数3、半径120px、速度4.5rad/s
Lv8: ダメージ55、周回数4、半径130px、速度5.0rad/s
```

---

#### 稲妻の指輪 (Thunder Ring)
**攻撃タイプ**: 雷撃（画面内ランダム敵に雷を落とす）

```
動作:
- 画面内のランダムな敵に雷を落とす
- 雷は対象頭上から瞬時に発生（視覚エフェクト付き）
- 範囲ダメージなし（直撃ダメージのみ）

レベル別パラメータ:
Lv1: ダメージ40、雷撃数1、クールダウン2.0s
Lv2: ダメージ40、雷撃数1、クールダウン1.7s
Lv3: ダメージ60、雷撃数2、クールダウン1.7s
Lv4: ダメージ60、雷撃数2、クールダウン1.5s
Lv5: ダメージ80、雷撃数3、クールダウン1.5s
Lv6: ダメージ80、雷撃数3、クールダウン1.3s
Lv7: ダメージ100、雷撃数4、クールダウン1.3s
Lv8: ダメージ100、雷撃数5、クールダウン1.0s
```

---

#### クロス (Cross)
**攻撃タイプ**: ブーメラン（往復する投擲物）

```
動作:
- プレイヤーの前方に投擲し、一定距離後に戻ってくる
- 往路・復路両方でダメージ判定
- 往復中に貫通

レベル別パラメータ:
Lv1: ダメージ30、弾数1、射程200px、クールダウン1.5s
Lv2: ダメージ30、弾数1、射程220px、クールダウン1.3s
Lv3: ダメージ40、弾数2、射程220px、クールダウン1.3s
Lv4: ダメージ40、弾数2、射程240px、クールダウン1.2s
Lv5: ダメージ50、弾数2、射程260px、クールダウン1.1s
Lv6: ダメージ50、弾数3、射程280px、クールダウン1.0s
Lv7: ダメージ60、弾数3、射程300px、クールダウン0.9s
Lv8: ダメージ60、弾数4、射程320px、クールダウン0.8s
```

---

#### ファイアウォンド (Fire Wand)
**攻撃タイプ**: 火球（最大HPの敵に大型弾を発射）

```
動作:
- 現在画面内で最大HPを持つ敵を狙って大型火球を発射
- 着弾時に小範囲の爆発（周囲の敵にもダメージ）
- クールダウンが長い高ダメージ武器

レベル別パラメータ:
Lv1: ダメージ50、爆発範囲50px、クールダウン3.0s、弾速200
Lv2: ダメージ70、爆発範囲60px、クールダウン2.7s、弾速220
Lv3: ダメージ90、爆発範囲70px、クールダウン2.5s、弾速240
Lv4: ダメージ110、爆発範囲70px、クールダウン2.3s、弾速260
Lv5: ダメージ130、爆発範囲80px、クールダウン2.1s、弾速280
Lv6: ダメージ150、爆発範囲90px、クールダウン2.0s、弾速300
Lv7: ダメージ170、爆発範囲100px、クールダウン1.8s、弾速320
Lv8: ダメージ200、爆発範囲120px、クールダウン1.5s、弾速350
```

---

### 1.3 武器パラメータ計算

プレイヤーのステータスは武器のパラメータに乗算される：

```rust
impl WeaponState {
    /// 実効ダメージ計算
    pub fn damage(&self, stats: &PlayerStats) -> f32 {
        self.base_damage() * stats.damage_multiplier
    }

    /// 実効クールダウン計算
    pub fn effective_cooldown(&self, stats: &PlayerStats) -> f32 {
        let reduction = (1.0 - stats.cooldown_reduction).max(0.1);
        self.base_cooldown() * reduction
    }

    /// 実効弾速計算
    pub fn speed(&self, stats: &PlayerStats) -> f32 {
        self.base_speed() * stats.projectile_speed_mult
    }

    /// 実効範囲計算
    pub fn range(&self, stats: &PlayerStats) -> f32 {
        self.base_range() * stats.area_multiplier
    }
}
```

---

## 2. 敵システム詳細

### 2.1 敵スポーンシステム

```rust
/// スポーンテーブル（時間帯別の出現敵タイプと確率）
pub fn get_spawn_table(elapsed_minutes: f32) -> Vec<(EnemyType, f32)> {
    let mut table = vec![];

    // コウモリは常時出現
    table.push((EnemyType::Bat, 1.0));

    // スケルトンは常時出現
    table.push((EnemyType::Skeleton, 1.0));

    if elapsed_minutes >= 5.0 {
        table.push((EnemyType::Zombie, 0.8));
    }
    if elapsed_minutes >= 10.0 {
        table.push((EnemyType::Ghost, 0.6));
    }
    if elapsed_minutes >= 15.0 {
        table.push((EnemyType::Demon, 0.5));
    }
    if elapsed_minutes >= 20.0 {
        table.push((EnemyType::Medusa, 0.4));
    }
    if elapsed_minutes >= 25.0 {
        table.push((EnemyType::Dragon, 0.3));
    }

    table
}

/// 時間に応じた難易度倍率
pub fn get_difficulty_multiplier(elapsed_minutes: f32) -> f32 {
    let base = 1.0_f32;
    let growth = 0.1 * elapsed_minutes;  // 1分あたり10%増加
    (base + growth).min(4.0)              // 最大4倍まで
}
```

### 2.2 スポーン位置の決定

```rust
/// カメラ外の画面外にスポーン
pub fn get_spawn_position(
    camera_pos: Vec2,
    window_size: Vec2,
    rng: &mut impl Rng,
) -> Vec2 {
    let margin = 100.0;  // 画面端からの余白
    let half_w = window_size.x / 2.0 + margin;
    let half_h = window_size.y / 2.0 + margin;

    // 画面の4辺のいずれかにランダムにスポーン
    match rng.gen_range(0..4) {
        0 => Vec2::new(rng.gen_range(-half_w..half_w), half_h) + camera_pos,   // 上
        1 => Vec2::new(rng.gen_range(-half_w..half_w), -half_h) + camera_pos,  // 下
        2 => Vec2::new(-half_w, rng.gen_range(-half_h..half_h)) + camera_pos,  // 左
        _ => Vec2::new(half_w, rng.gen_range(-half_h..half_h)) + camera_pos,   // 右
    }
}
```

### 2.3 ボス（デス）の詳細設計

ボスは30分経過後に出現。通常の敵スポーンは停止する。

**フェーズ1（HP: 100%〜60%）:**
- 低速でプレイヤーを追跡
- 接触でプレイヤーに50ダメージ

**フェーズ2（HP: 60%〜30%）:**
- 移動速度1.5倍に増加
- 周囲に小さな分身（Mini Death）を3体スポーン

**フェーズ3（HP: 30%〜0%）:**
- 移動速度2倍
- 分身が5体に増加
- 特殊攻撃: 鎌を遠距離に投擲

```rust
/// ボスのフェーズ判定
pub fn get_boss_phase(hp_ratio: f32) -> BossPhase {
    if hp_ratio > 0.6 {
        BossPhase::Phase1
    } else if hp_ratio > 0.3 {
        BossPhase::Phase2
    } else {
        BossPhase::Phase3
    }
}
```

---

## 3. 経験値・レベルアップシステム詳細

### 3.1 XPジェムのドロップ

```rust
/// 敵死亡時のXPジェムスポーン
pub fn spawn_xp_gems(
    commands: &mut Commands,
    position: Vec2,
    xp_value: u32,
) {
    // 大量XPは複数のジェムに分割してスポーン
    let (gem_count, gem_value) = if xp_value >= 20 {
        let count = (xp_value / 10).min(5);  // 最大5個
        (count, xp_value / count)
    } else {
        (1, xp_value)
    };

    for _ in 0..gem_count {
        let offset = Vec2::new(
            rand::random::<f32>() * 20.0 - 10.0,
            rand::random::<f32>() * 20.0 - 10.0,
        );
        commands.spawn((
            ExperienceGem { value: gem_value },
            CircleCollider { radius: 6.0 },
            Transform::from_translation((position + offset).extend(0.5)),
            // 緑色のジェムスプライト
        ));
    }
}
```

### 3.2 XP吸収メカニクス

```rust
/// XPジェムの磁石効果（吸引範囲内に入ったら自動吸収）
pub fn attract_gems_to_player(
    player_query: Query<(&Transform, &PlayerStats), With<Player>>,
    mut gem_query: Query<(Entity, &mut Transform, &ExperienceGem), Without<Player>>,
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
    time: Res<Time>,
) {
    let Ok((player_transform, player_stats)) = player_query.get_single() else { return };
    let player_pos = player_transform.translation.truncate();
    let pickup_radius = player_stats.pickup_radius;

    for (entity, mut transform, gem) in gem_query.iter_mut() {
        let gem_pos = transform.translation.truncate();
        let dist = gem_pos.distance(player_pos);

        if dist < pickup_radius {
            // 磁石範囲内: プレイヤーに向かって高速移動
            let dir = (player_pos - gem_pos).normalize_or_zero();
            let attract_speed = 400.0 + (pickup_radius - dist) * 5.0;
            transform.translation += (dir * attract_speed * time.delta_secs()).extend(0.0);

            // プレイヤー位置に到達したら吸収
            if dist < 16.0 {
                game_data.current_xp += gem.value;
                commands.entity(entity).despawn();
            }
        }
    }
}
```

### 3.3 レベルアップ処理

```rust
/// XP量を確認してレベルアップ処理
pub fn check_level_up(
    mut game_data: ResMut<GameData>,
    mut level_up_events: EventWriter<LevelUpEvent>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    while game_data.current_xp >= game_data.xp_to_next_level {
        game_data.current_xp -= game_data.xp_to_next_level;
        game_data.current_level += 1;
        game_data.xp_to_next_level = calculate_xp_requirement(game_data.current_level);

        level_up_events.send(LevelUpEvent {
            new_level: game_data.current_level,
        });

        // レベルアップ選択画面に遷移（ゲームを一時停止）
        next_state.set(AppState::LevelUp);
    }
}

/// 次のレベルに必要なXP計算
pub fn calculate_xp_requirement(level: u32) -> u32 {
    // 指数的増加: floor(20 * 1.1^(level-1))
    (20.0 * 1.1_f32.powi(level as i32 - 1)).floor() as u32
}
```

### 3.4 レベルアップ選択肢生成

```rust
/// レベルアップ選択肢をランダムに生成
pub fn generate_level_up_choices(
    weapon_inventory: &WeaponInventory,
    passive_inventory: &PassiveInventory,
    player_stats: &PlayerStats,
    rng: &mut impl Rng,
) -> Vec<UpgradeChoice> {
    let mut candidates: Vec<UpgradeChoice> = vec![];

    // アップグレード可能な武器を候補に追加
    for weapon_state in &weapon_inventory.weapons {
        if weapon_state.level < 8 && !weapon_state.evolved {
            candidates.push(UpgradeChoice::WeaponUpgrade(weapon_state.weapon_type));
        }
    }

    // 武器枠に空きがあれば、未所持の新武器を候補に追加
    if weapon_inventory.weapons.len() < 6 {
        for weapon_type in WeaponType::all() {
            if !weapon_inventory.has_weapon(weapon_type) {
                candidates.push(UpgradeChoice::NewWeapon(weapon_type));
            }
        }
    }

    // アップグレード可能なパッシブを候補に追加
    for passive_state in &passive_inventory.items {
        if passive_state.level < 5 {
            candidates.push(UpgradeChoice::PassiveUpgrade(passive_state.item_type));
        }
    }

    // 未所持のパッシブを候補に追加
    for item_type in PassiveItemType::all() {
        if !passive_inventory.has_item(item_type) {
            candidates.push(UpgradeChoice::PassiveItem(item_type));
        }
    }

    // ラックに応じたシャッフル後、最大3〜4つ選択
    candidates.shuffle(rng);
    let count = if player_stats.luck > 1.5 { 4 } else { 3 };
    candidates.into_iter().take(count).collect()
}
```

---

## 4. 武器進化システム詳細

### 4.1 進化条件チェック

```rust
/// 武器進化が可能かチェック
pub fn can_evolve_weapon(
    weapon_state: &WeaponState,
    passive_inventory: &PassiveInventory,
) -> Option<WeaponType> {
    if weapon_state.level < 8 || weapon_state.evolved {
        return None;
    }

    let required_passive = get_evolution_requirement(weapon_state.weapon_type)?;

    if passive_inventory.has_item(required_passive) {
        Some(get_evolved_weapon(weapon_state.weapon_type))
    } else {
        None
    }
}

/// 武器進化に必要なパッシブ
pub fn get_evolution_requirement(weapon: WeaponType) -> Option<PassiveItemType> {
    match weapon {
        WeaponType::Whip        => Some(PassiveItemType::HollowHeart),
        WeaponType::MagicWand   => Some(PassiveItemType::EmptyTome),
        WeaponType::Knife       => Some(PassiveItemType::Bracer),
        WeaponType::Garlic      => Some(PassiveItemType::Pummarola),
        WeaponType::Bible       => Some(PassiveItemType::Spellbinder),
        WeaponType::ThunderRing => Some(PassiveItemType::Duplicator),
        _                       => None,  // クロス・ファイアウォンドは進化なし（MVP）
    }
}
```

### 4.2 宝箱での進化発動

```rust
/// 宝箱を開けた時の処理
pub fn open_treasure(
    weapon_inventory: &mut WeaponInventory,
    passive_inventory: &PassiveInventory,
    rng: &mut impl Rng,
) -> TreasureContent {
    // 進化可能な武器がある場合は優先的に進化
    for weapon_state in weapon_inventory.weapons.iter_mut() {
        if let Some(evolved_type) = can_evolve_weapon(weapon_state, passive_inventory) {
            weapon_state.weapon_type = evolved_type;
            weapon_state.evolved = true;
            return TreasureContent::WeaponEvolution(evolved_type);
        }
    }

    // 進化がない場合はランダムな内容
    match rng.gen_range(0..4) {
        0 => TreasureContent::RandomUpgrade,  // ランダムアップグレード
        1 => TreasureContent::HpRestore(30.0), // HP30%回復
        2 => TreasureContent::Gold(rng.gen_range(50..200)),
        _ => TreasureContent::RandomUpgrade,
    }
}
```

---

## 5. パッシブアイテムシステム詳細

### 5.1 パッシブ効果の適用

```rust
/// パッシブアイテムをプレイヤーステータスに適用
pub fn apply_passives(
    base_stats: &CharacterBaseStats,
    passive_inventory: &PassiveInventory,
) -> PlayerStats {
    let mut stats = PlayerStats::from_base(base_stats);

    for passive in &passive_inventory.items {
        let level = passive.level as f32;

        match passive.item_type {
            PassiveItemType::Spinach => {
                stats.damage_multiplier *= 1.0 + 0.1 * level;  // +10%/Lv
            }
            PassiveItemType::Wings => {
                stats.move_speed *= 1.0 + 0.1 * level;          // +10%/Lv
            }
            PassiveItemType::HollowHeart => {
                stats.max_hp *= 1.0 + 0.2 * level;              // +20%/Lv
            }
            PassiveItemType::Clover => {
                stats.luck *= 1.0 + 0.1 * level;                // +10%/Lv
            }
            PassiveItemType::EmptyTome => {
                stats.cooldown_reduction += 0.08 * level;        // -8%/Lv
            }
            PassiveItemType::Bracer => {
                stats.projectile_speed_mult *= 1.0 + 0.1 * level; // +10%/Lv
            }
            PassiveItemType::Spellbinder => {
                stats.duration_multiplier *= 1.0 + 0.1 * level; // +10%/Lv
            }
            PassiveItemType::Duplicator => {
                stats.extra_projectiles += passive.level as u32; // +1/Lv
            }
            PassiveItemType::Pummarola => {
                stats.hp_regen += 0.5 * level;                   // +0.5/s per Lv
            }
        }

        // クールダウン削減は最大90%まで（最低10%のクールダウンを保証）
        stats.cooldown_reduction = stats.cooldown_reduction.min(0.9);
    }

    stats
}
```

---

## 6. ダメージシステム

### 6.1 敵へのダメージ適用

```rust
/// 敵へのダメージイベント
#[derive(Event)]
pub struct DamageEnemyEvent {
    pub entity: Entity,
    pub damage: f32,
    pub weapon_type: WeaponType,
}

/// ダメージを敵エンティティに適用
pub fn apply_damage_to_enemies(
    mut damage_events: EventReader<DamageEnemyEvent>,
    mut enemy_query: Query<(&mut Enemy, &Transform)>,
    mut death_events: EventWriter<EnemyDiedEvent>,
    mut commands: Commands,
) {
    for event in damage_events.read() {
        let Ok((mut enemy, transform)) = enemy_query.get_mut(event.entity) else { continue };

        enemy.current_hp -= event.damage;

        // ダメージ数値表示のスポーン（フローティングテキスト）
        // spawn_damage_text(&mut commands, transform.translation, event.damage);

        if enemy.current_hp <= 0.0 {
            death_events.send(EnemyDiedEvent {
                entity: event.entity,
                enemy_type: enemy.enemy_type,
                position: transform.translation.truncate(),
                xp_value: enemy.xp_value,
                gold_chance: enemy.gold_chance,
            });
        }
    }
}
```

### 6.2 プレイヤーへのダメージ適用

```rust
/// プレイヤーへのダメージ適用（無敵時間を考慮）
pub fn apply_damage_to_player(
    mut player_query: Query<(&mut PlayerStats, Option<&InvincibilityTimer>), With<Player>>,
    mut damage_events: EventReader<PlayerDamagedEvent>,
    mut game_over_events: EventWriter<GameOverEvent>,
    mut commands: Commands,
    player_entity: Query<Entity, With<Player>>,
    game_data: Res<GameData>,
) {
    let Ok((mut stats, invincibility)) = player_query.get_single_mut() else { return };

    // 無敵時間中はダメージを受けない
    if invincibility.is_some() {
        return;
    }

    for event in damage_events.read() {
        stats.current_hp -= event.damage;

        // 無敵時間を付与（0.5秒）
        let Ok(entity) = player_entity.get_single() else { continue };
        commands.entity(entity).insert(InvincibilityTimer { remaining: 0.5 });

        if stats.current_hp <= 0.0 {
            game_over_events.send(GameOverEvent {
                survived_time: game_data.elapsed_time,
                kill_count: game_data.kill_count,
                gold_earned: game_data.gold_earned,
            });
            return;
        }
    }
}
```

---

## 7. 宝箱スポーンシステム

```rust
#[derive(Resource)]
pub struct TreasureSpawner {
    pub timer: f32,
    pub interval: f32,   // デフォルト: 180秒（3分）
}

pub fn spawn_treasure_on_timer(
    mut spawner: ResMut<TreasureSpawner>,
    time: Res<Time>,
    camera_query: Query<&Transform, With<Camera>>,
    mut commands: Commands,
    rng: Local<rand::rngs::SmallRng>,
) {
    spawner.timer += time.delta_secs();

    if spawner.timer >= spawner.interval {
        spawner.timer = 0.0;

        let Ok(camera_transform) = camera_query.get_single() else { return };
        let camera_pos = camera_transform.translation.truncate();

        // 画面内のランダムな位置にスポーン（プレイヤーから少し離れた位置）
        let offset = Vec2::new(
            (rng.gen::<f32>() - 0.5) * 400.0,
            (rng.gen::<f32>() - 0.5) * 300.0,
        );

        commands.spawn((
            Treasure,
            CircleCollider { radius: 20.0 },
            Transform::from_translation((camera_pos + offset).extend(0.2)),
            // 宝箱スプライト
        ));
    }
}
```

---

## 8. ゴールドシステム

```rust
/// 敵死亡時のゴールドドロップ
pub fn handle_gold_drop(
    mut death_events: EventReader<EnemyDiedEvent>,
    mut commands: Commands,
    mut rng: Local<rand::rngs::SmallRng>,
) {
    for event in death_events.read() {
        if rng.gen::<f32>() < event.gold_chance {
            let gold_value = rng.gen_range(1..=5);
            commands.spawn((
                GoldCoin { value: gold_value },
                CircleCollider { radius: 6.0 },
                Transform::from_translation(event.position.extend(0.4)),
                // 金色のコインスプライト
            ));
        }
    }
}

/// プレイヤーがゴールドを拾う
pub fn pickup_gold(
    player_query: Query<&Transform, With<Player>>,
    coin_query: Query<(Entity, &Transform, &GoldCoin), Without<Player>>,
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
    mut meta_progress: ResMut<MetaProgress>,
) {
    let Ok(player_transform) = player_query.get_single() else { return };
    let player_pos = player_transform.translation.truncate();

    for (entity, transform, coin) in coin_query.iter() {
        let coin_pos = transform.translation.truncate();
        if player_pos.distance(coin_pos) < 24.0 {  // 自動拾い範囲
            game_data.gold_earned += coin.value;
            meta_progress.total_gold += coin.value;
            commands.entity(entity).despawn();
        }
    }
}
```
