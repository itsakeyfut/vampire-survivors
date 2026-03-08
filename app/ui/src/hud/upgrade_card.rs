//! Upgrade card HUD widget.
//!
//! Spawns individual upgrade selection cards and handles their interaction
//! colors.  Display text helpers (weapon/passive names and descriptions) also
//! live here so they can be reused from other screens in the future (e.g. the
//! meta-progression shop).

use bevy::prelude::*;
use vs_core::resources::Language;
use vs_core::types::{PassiveItemType, UpgradeChoice, WeaponType};

use crate::components::{ButtonAction, MenuButton};
use crate::config::hud::upgrade_card::{UpgradeCardHudConfig, UpgradeCardHudParams};

// ---------------------------------------------------------------------------
// Fallback constants
// ---------------------------------------------------------------------------

const DEFAULT_CARD_NORMAL: Color = Color::srgb(0.12, 0.08, 0.28);
const DEFAULT_CARD_HOVER: Color = Color::srgb(0.22, 0.14, 0.48);
const DEFAULT_CARD_PRESSED: Color = Color::srgb(0.08, 0.05, 0.18);
const DEFAULT_SUBTITLE_COLOR: Color = Color::srgb(0.85, 0.70, 0.30);
const DEFAULT_TEXT_COLOR: Color = Color::srgb(0.95, 0.90, 0.85);
const DEFAULT_CARD_WIDTH: f32 = 260.0;
const DEFAULT_CARD_HEIGHT: f32 = 320.0;
const DEFAULT_PADDING: f32 = 16.0;
const DEFAULT_INNER_GAP: f32 = 12.0;
const DEFAULT_FONT_SIZE_NAME: f32 = 28.0;
const DEFAULT_FONT_SIZE_SUBTITLE: f32 = 20.0;
const DEFAULT_FONT_SIZE_DESC: f32 = 18.0;
const DEFAULT_ICON_SIZE: f32 = 64.0;

// Icon colors per upgrade category (placeholder until sprite assets are added).
const ICON_COLOR_NEW_WEAPON: Color = Color::srgb(0.25, 0.50, 1.00);
const ICON_COLOR_WEAPON_UPGRADE: Color = Color::srgb(0.40, 0.70, 1.00);
const ICON_COLOR_NEW_PASSIVE: Color = Color::srgb(0.20, 0.75, 0.50);
const ICON_COLOR_PASSIVE_UPGRADE: Color = Color::srgb(0.40, 0.90, 0.65);

// ---------------------------------------------------------------------------
// Marker component
// ---------------------------------------------------------------------------

/// Marker component placed on every upgrade card button entity.
///
/// The `index` field corresponds to the card's position in
/// [`vs_core::resources::LevelUpChoices::choices`].
#[derive(Component)]
pub struct UpgradeCardHud {
    /// Zero-based index of this card in the choices list.
    pub index: usize,
}

// ---------------------------------------------------------------------------
// Display text helpers
// ---------------------------------------------------------------------------

/// Returns the upgrade-type subtitle (e.g. "New Weapon") in the given language.
pub fn choice_subtitle(choice: &UpgradeChoice, lang: Language) -> &'static str {
    match (choice, lang) {
        (UpgradeChoice::NewWeapon(_), Language::Japanese) => "新しい武器",
        (UpgradeChoice::NewWeapon(_), Language::English) => "New Weapon",
        (UpgradeChoice::WeaponUpgrade(_), Language::Japanese) => "武器強化",
        (UpgradeChoice::WeaponUpgrade(_), Language::English) => "Weapon Upgrade",
        (UpgradeChoice::PassiveItem(_), Language::Japanese) => "新しいパッシブ",
        (UpgradeChoice::PassiveItem(_), Language::English) => "New Passive",
        (UpgradeChoice::PassiveUpgrade(_), Language::Japanese) => "パッシブ強化",
        (UpgradeChoice::PassiveUpgrade(_), Language::English) => "Passive Upgrade",
    }
}

/// Returns the item name for a choice in the given language.
pub fn choice_name(choice: &UpgradeChoice, lang: Language) -> &'static str {
    match choice {
        UpgradeChoice::NewWeapon(wt) | UpgradeChoice::WeaponUpgrade(wt) => weapon_name(*wt, lang),
        UpgradeChoice::PassiveItem(pt) | UpgradeChoice::PassiveUpgrade(pt) => {
            passive_name(*pt, lang)
        }
    }
}

/// Returns the one-line effect description for a choice in the given language.
pub fn choice_description(choice: &UpgradeChoice, lang: Language) -> &'static str {
    match choice {
        UpgradeChoice::NewWeapon(wt) | UpgradeChoice::WeaponUpgrade(wt) => {
            weapon_description(*wt, lang)
        }
        UpgradeChoice::PassiveItem(pt) | UpgradeChoice::PassiveUpgrade(pt) => {
            passive_description(*pt, lang)
        }
    }
}

fn weapon_name(wt: WeaponType, lang: Language) -> &'static str {
    match (wt, lang) {
        (WeaponType::Whip, Language::Japanese) => "ムチ",
        (WeaponType::Whip, Language::English) => "Whip",
        (WeaponType::MagicWand, Language::Japanese) => "魔法の杖",
        (WeaponType::MagicWand, Language::English) => "Magic Wand",
        (WeaponType::Knife, Language::Japanese) => "ナイフ",
        (WeaponType::Knife, Language::English) => "Knife",
        (WeaponType::Garlic, Language::Japanese) => "ニンニク",
        (WeaponType::Garlic, Language::English) => "Garlic",
        (WeaponType::Bible, Language::Japanese) => "聖書",
        (WeaponType::Bible, Language::English) => "Bible",
        (WeaponType::ThunderRing, Language::Japanese) => "サンダーリング",
        (WeaponType::ThunderRing, Language::English) => "Thunder Ring",
        (WeaponType::Cross, Language::Japanese) => "クロス",
        (WeaponType::Cross, Language::English) => "Cross",
        (WeaponType::FireWand, Language::Japanese) => "炎の杖",
        (WeaponType::FireWand, Language::English) => "Fire Wand",
        (WeaponType::BloodyTear, Language::Japanese) => "血の涙",
        (WeaponType::BloodyTear, Language::English) => "Bloody Tear",
        (WeaponType::HolyWand, Language::Japanese) => "聖なる杖",
        (WeaponType::HolyWand, Language::English) => "Holy Wand",
        (WeaponType::ThousandEdge, Language::Japanese) => "千の刃",
        (WeaponType::ThousandEdge, Language::English) => "Thousand Edge",
        (WeaponType::SoulEater, Language::Japanese) => "魂喰い",
        (WeaponType::SoulEater, Language::English) => "Soul Eater",
        (WeaponType::UnholyVespers, Language::Japanese) => "邪悪な晩課",
        (WeaponType::UnholyVespers, Language::English) => "Unholy Vespers",
        (WeaponType::LightningRing, Language::Japanese) => "雷のリング",
        (WeaponType::LightningRing, Language::English) => "Lightning Ring",
    }
}

fn weapon_description(wt: WeaponType, lang: Language) -> &'static str {
    match (wt, lang) {
        (WeaponType::Whip, Language::Japanese) => "左右交互に扇状の斬撃。",
        (WeaponType::Whip, Language::English) => "Fan-shaped swing, alternating sides.",
        (WeaponType::MagicWand, Language::Japanese) => "最寄りの敵に向かうホーミング弾。",
        (WeaponType::MagicWand, Language::English) => "Homing projectile toward nearest enemy.",
        (WeaponType::Knife, Language::Japanese) => "移動方向へ高速貫通弾。",
        (WeaponType::Knife, Language::English) => "Fast piercing shot in movement direction.",
        (WeaponType::Garlic, Language::Japanese) => "プレイヤー周囲に継続ダメージのオーラ。",
        (WeaponType::Garlic, Language::English) => "Continuous damage aura around player.",
        (WeaponType::Bible, Language::Japanese) => "プレイヤーの周りを旋回する飛翔体。",
        (WeaponType::Bible, Language::English) => "Orbiting projectile that circles the player.",
        (WeaponType::ThunderRing, Language::Japanese) => "画面全体にランダムな落雷。",
        (WeaponType::ThunderRing, Language::English) => {
            "Random lightning strikes across the screen."
        }
        (WeaponType::Cross, Language::Japanese) => "飛んで戻ってくるブーメラン。",
        (WeaponType::Cross, Language::English) => "Boomerang that flies out and returns.",
        (WeaponType::FireWand, Language::Japanese) => "最大HPの敵を狙う火の玉。",
        (WeaponType::FireWand, Language::English) => "Fireball targeting the highest-HP enemy.",
        (WeaponType::BloodyTear, Language::Japanese) => "進化形ムチ — 広範囲の大斬撃。",
        (WeaponType::BloodyTear, Language::English) => "Evolved Whip — massive area slash.",
        (WeaponType::HolyWand, Language::Japanese) => "進化形魔法の杖 — 高速ホーミング弾。",
        (WeaponType::HolyWand, Language::English) => "Evolved Magic Wand — rapid homing bolts.",
        (WeaponType::ThousandEdge, Language::Japanese) => "進化形ナイフ — 終わりなき刃の嵐。",
        (WeaponType::ThousandEdge, Language::English) => "Evolved Knife — endless blade flurry.",
        (WeaponType::SoulEater, Language::Japanese) => "進化形ニンニク — 敵の命を吸収。",
        (WeaponType::SoulEater, Language::English) => "Evolved Garlic — drains life from enemies.",
        (WeaponType::UnholyVespers, Language::Japanese) => "進化形聖書 — 無限に旋回する刃。",
        (WeaponType::UnholyVespers, Language::English) => {
            "Evolved Bible — infinite orbiting blades."
        }
        (WeaponType::LightningRing, Language::Japanese) => "進化形サンダーリング — 雷の嵐。",
        (WeaponType::LightningRing, Language::English) => {
            "Evolved Thunder Ring — storm of lightning."
        }
    }
}

fn passive_name(pt: PassiveItemType, lang: Language) -> &'static str {
    match (pt, lang) {
        (PassiveItemType::Spinach, Language::Japanese) => "ほうれん草",
        (PassiveItemType::Spinach, Language::English) => "Spinach",
        (PassiveItemType::Wings, Language::Japanese) => "翼",
        (PassiveItemType::Wings, Language::English) => "Wings",
        (PassiveItemType::HollowHeart, Language::Japanese) => "虚ろな心",
        (PassiveItemType::HollowHeart, Language::English) => "Hollow Heart",
        (PassiveItemType::Clover, Language::Japanese) => "クローバー",
        (PassiveItemType::Clover, Language::English) => "Clover",
        (PassiveItemType::EmptyTome, Language::Japanese) => "空の魔導書",
        (PassiveItemType::EmptyTome, Language::English) => "Empty Tome",
        (PassiveItemType::Bracer, Language::Japanese) => "ブレーサー",
        (PassiveItemType::Bracer, Language::English) => "Bracer",
        (PassiveItemType::Spellbinder, Language::Japanese) => "スペルバインダー",
        (PassiveItemType::Spellbinder, Language::English) => "Spellbinder",
        (PassiveItemType::Duplicator, Language::Japanese) => "複製機",
        (PassiveItemType::Duplicator, Language::English) => "Duplicator",
        (PassiveItemType::Pummarola, Language::Japanese) => "プンマローラ",
        (PassiveItemType::Pummarola, Language::English) => "Pummarola",
    }
}

/// Returns the placeholder icon background color for a given upgrade choice.
///
/// Used until weapon/passive sprite assets are integrated in Phase 17.
/// Colors are chosen to distinguish the four upgrade categories at a glance:
/// - New weapon (blue) / Weapon upgrade (light blue)
/// - New passive (green) / Passive upgrade (light green)
fn icon_color(choice: &UpgradeChoice) -> Color {
    match choice {
        UpgradeChoice::NewWeapon(_) => ICON_COLOR_NEW_WEAPON,
        UpgradeChoice::WeaponUpgrade(_) => ICON_COLOR_WEAPON_UPGRADE,
        UpgradeChoice::PassiveItem(_) => ICON_COLOR_NEW_PASSIVE,
        UpgradeChoice::PassiveUpgrade(_) => ICON_COLOR_PASSIVE_UPGRADE,
    }
}

fn passive_description(pt: PassiveItemType, lang: Language) -> &'static str {
    match (pt, lang) {
        (PassiveItemType::Spinach, Language::Japanese) => "LVごとにダメージ+10%。",
        (PassiveItemType::Spinach, Language::English) => "+10% damage per level.",
        (PassiveItemType::Wings, Language::Japanese) => "LVごとに移動速度+10%。",
        (PassiveItemType::Wings, Language::English) => "+10% move speed per level.",
        (PassiveItemType::HollowHeart, Language::Japanese) => "LVごとに最大HP+20%。",
        (PassiveItemType::HollowHeart, Language::English) => "+20% max HP per level.",
        (PassiveItemType::Clover, Language::Japanese) => "LVごとに幸運+10%。",
        (PassiveItemType::Clover, Language::English) => "+10% luck per level.",
        (PassiveItemType::EmptyTome, Language::Japanese) => "LVごとに武器クールダウン-8%。",
        (PassiveItemType::EmptyTome, Language::English) => "-8% weapon cooldown per level.",
        (PassiveItemType::Bracer, Language::Japanese) => "LVごとに発射速度+10%。",
        (PassiveItemType::Bracer, Language::English) => "+10% projectile speed per level.",
        (PassiveItemType::Spellbinder, Language::Japanese) => "LVごとに武器持続時間+10%。",
        (PassiveItemType::Spellbinder, Language::English) => "+10% weapon duration per level.",
        (PassiveItemType::Duplicator, Language::Japanese) => "LVごとに発射数+1。",
        (PassiveItemType::Duplicator, Language::English) => "+1 projectile count per level.",
        (PassiveItemType::Pummarola, Language::Japanese) => "LVごとにHP自然回復+0.5/秒。",
        (PassiveItemType::Pummarola, Language::English) => "+0.5 HP regeneration/s per level.",
    }
}

// ---------------------------------------------------------------------------
// Spawn function
// ---------------------------------------------------------------------------

/// Spawns a single upgrade card button inside `parent`.
///
/// - `index`  — zero-based position in the choices list (stored in [`UpgradeCardHud`]).
/// - `choice` — the upgrade to display.
/// - `cfg`    — layout/color config; pass `card_params.get()`. Falls back to
///   `DEFAULT_*` constants when the asset is not yet loaded.
///
/// Card dimensions are clamped: `card_width >= 64` and `card_height >= 64` to
/// prevent zero-sized or negative-`max_width` layouts from invalid RON values.
pub fn spawn_upgrade_card(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    choice: &UpgradeChoice,
    cfg: Option<&UpgradeCardHudConfig>,
    font: Handle<Font>,
    lang: Language,
) {
    let card_width = cfg
        .map(|c| c.card_width)
        .unwrap_or(DEFAULT_CARD_WIDTH)
        .max(64.0);
    let card_height = cfg
        .map(|c| c.card_height)
        .unwrap_or(DEFAULT_CARD_HEIGHT)
        .max(64.0);
    let padding = cfg.map(|c| c.padding).unwrap_or(DEFAULT_PADDING).max(0.0);
    let inner_gap = cfg
        .map(|c| c.inner_gap)
        .unwrap_or(DEFAULT_INNER_GAP)
        .max(0.0);
    let card_normal = cfg
        .map(|c| Color::from(&c.card_normal))
        .unwrap_or(DEFAULT_CARD_NORMAL);
    let subtitle_color = cfg
        .map(|c| Color::from(&c.subtitle_color))
        .unwrap_or(DEFAULT_SUBTITLE_COLOR);
    let text_color = cfg
        .map(|c| Color::from(&c.text_color))
        .unwrap_or(DEFAULT_TEXT_COLOR);
    let font_size_name = cfg
        .map(|c| c.font_size_name)
        .unwrap_or(DEFAULT_FONT_SIZE_NAME)
        .max(1.0);
    let font_size_subtitle = cfg
        .map(|c| c.font_size_subtitle)
        .unwrap_or(DEFAULT_FONT_SIZE_SUBTITLE)
        .max(1.0);
    let font_size_desc = cfg
        .map(|c| c.font_size_desc)
        .unwrap_or(DEFAULT_FONT_SIZE_DESC)
        .max(1.0);
    let icon_size = cfg
        .map(|c| c.icon_size)
        .unwrap_or(DEFAULT_ICON_SIZE)
        .max(8.0);

    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(card_width),
                height: Val::Px(card_height),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(padding)),
                row_gap: Val::Px(inner_gap),
                ..default()
            },
            BackgroundColor(card_normal),
            MenuButton {
                action: ButtonAction::SelectUpgrade(index),
            },
            UpgradeCardHud { index },
        ))
        .with_children(|card| {
            // Icon placeholder — a solid colored square.
            // Replaced with a real sprite in Phase 17 when weapon/passive
            // art assets are integrated.
            card.spawn((
                Node {
                    width: Val::Px(icon_size),
                    height: Val::Px(icon_size),
                    ..default()
                },
                BackgroundColor(icon_color(choice)),
            ));

            // Subtitle: upgrade type label.
            card.spawn((
                Text::new(choice_subtitle(choice, lang)),
                TextFont {
                    font: font.clone(),
                    font_size: font_size_subtitle,
                    ..default()
                },
                TextColor(subtitle_color),
            ));

            // Item name.
            card.spawn((
                Text::new(choice_name(choice, lang)),
                TextFont {
                    font: font.clone(),
                    font_size: font_size_name,
                    ..default()
                },
                TextColor(text_color),
            ));

            // Effect description.
            card.spawn((
                Text::new(choice_description(choice, lang)),
                TextFont {
                    font: font.clone(),
                    font_size: font_size_desc,
                    ..default()
                },
                TextColor(text_color),
                Node {
                    max_width: Val::Px((card_width - padding * 2.0).max(0.0)),
                    ..default()
                },
            ));
        });
}

// ---------------------------------------------------------------------------
// Interaction handler
// ---------------------------------------------------------------------------

/// Query filter for changed [`UpgradeCardHud`] interactions.
type ChangedCard = (Changed<Interaction>, With<UpgradeCardHud>);

/// Overrides card button colors with the card-specific palette.
///
/// Runs every frame so hover and press feedback is always available.
/// Must run after [`crate::components::handle_button_interaction`] so card
/// colors take precedence (cards carry both [`MenuButton`] and [`UpgradeCardHud`]).
pub fn handle_card_interaction(
    mut card_q: Query<(&Interaction, &mut BackgroundColor), ChangedCard>,
    card_cfg: UpgradeCardHudParams,
) {
    let (normal, hover, pressed) = if let Some(c) = card_cfg.get() {
        (
            Color::from(&c.card_normal),
            Color::from(&c.card_hover),
            Color::from(&c.card_pressed),
        )
    } else {
        (
            DEFAULT_CARD_NORMAL,
            DEFAULT_CARD_HOVER,
            DEFAULT_CARD_PRESSED,
        )
    };

    for (interaction, mut bg) in card_q.iter_mut() {
        *bg = BackgroundColor(match interaction {
            Interaction::Pressed => pressed,
            Interaction::Hovered => hover,
            Interaction::None => normal,
        });
    }
}

// ---------------------------------------------------------------------------
// Hot-reload system
// ---------------------------------------------------------------------------

/// Logs when `config/ui/hud/upgrade_card.ron` is loaded or hot-reloaded.
///
/// # Design note — no live entity updates
///
/// Upgrade cards are spawned as children of a transient level-up overlay
/// (`DespawnOnExit(AppState::LevelUp)`).  Because the overlay is recreated
/// fresh on every `OnEnter(LevelUp)`, layout dimensions cannot be patched on
/// existing entities without a full despawn/respawn cycle.
///
/// - **Hover/press colors** are already live: [`handle_card_interaction`]
///   reads from [`UpgradeCardHudParams`] every frame.
/// - Edits to `upgrade_card.ron` take full effect the next time the level-up
///   overlay opens, which is the correct hot-reload behaviour for a transient screen.
pub fn hot_reload_upgrade_card_hud(mut events: MessageReader<AssetEvent<UpgradeCardHudConfig>>) {
    for event in events.read() {
        match event {
            AssetEvent::Added { .. } => {
                info!("✅ Upgrade card HUD config loaded");
            }
            AssetEvent::Modified { .. } => {
                info!("🔥 Upgrade card HUD config hot-reloaded");
            }
            AssetEvent::Removed { .. } => {
                warn!("⚠️ Upgrade card HUD config removed");
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weapon_name_covers_all_variants() {
        let weapons = [
            WeaponType::Whip,
            WeaponType::MagicWand,
            WeaponType::Knife,
            WeaponType::Garlic,
            WeaponType::Bible,
            WeaponType::ThunderRing,
            WeaponType::Cross,
            WeaponType::FireWand,
            WeaponType::BloodyTear,
            WeaponType::HolyWand,
            WeaponType::ThousandEdge,
            WeaponType::SoulEater,
            WeaponType::UnholyVespers,
            WeaponType::LightningRing,
        ];
        for wt in weapons {
            for lang in [Language::English, Language::Japanese] {
                assert!(
                    !weapon_name(wt, lang).is_empty(),
                    "weapon_name({wt:?}, {lang:?}) must not be empty"
                );
                assert!(
                    !weapon_description(wt, lang).is_empty(),
                    "weapon_description({wt:?}, {lang:?}) must not be empty"
                );
            }
        }
    }

    #[test]
    fn passive_name_covers_all_variants() {
        let passives = [
            PassiveItemType::Spinach,
            PassiveItemType::Wings,
            PassiveItemType::HollowHeart,
            PassiveItemType::Clover,
            PassiveItemType::EmptyTome,
            PassiveItemType::Bracer,
            PassiveItemType::Spellbinder,
            PassiveItemType::Duplicator,
            PassiveItemType::Pummarola,
        ];
        for pt in passives {
            for lang in [Language::English, Language::Japanese] {
                assert!(
                    !passive_name(pt, lang).is_empty(),
                    "passive_name({pt:?}, {lang:?}) must not be empty"
                );
                assert!(
                    !passive_description(pt, lang).is_empty(),
                    "passive_description({pt:?}, {lang:?}) must not be empty"
                );
            }
        }
    }

    #[test]
    fn choice_subtitle_returns_correct_category() {
        let en = Language::English;
        assert_eq!(
            choice_subtitle(&UpgradeChoice::NewWeapon(WeaponType::Whip), en),
            "New Weapon"
        );
        assert_eq!(
            choice_subtitle(&UpgradeChoice::WeaponUpgrade(WeaponType::Whip), en),
            "Weapon Upgrade"
        );
        assert_eq!(
            choice_subtitle(&UpgradeChoice::PassiveItem(PassiveItemType::Spinach), en),
            "New Passive"
        );
        assert_eq!(
            choice_subtitle(&UpgradeChoice::PassiveUpgrade(PassiveItemType::Spinach), en),
            "Passive Upgrade"
        );

        let jp = Language::Japanese;
        assert_eq!(
            choice_subtitle(&UpgradeChoice::NewWeapon(WeaponType::Whip), jp),
            "新しい武器"
        );
        assert_eq!(
            choice_subtitle(&UpgradeChoice::WeaponUpgrade(WeaponType::Whip), jp),
            "武器強化"
        );
    }

    #[test]
    fn spawn_upgrade_card_produces_button_with_marker() {
        use bevy::state::app::StatesPlugin;
        use vs_core::states::AppState;

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<AppState>();

        let choice = UpgradeChoice::NewWeapon(WeaponType::Whip);
        let mut cmds = app.world_mut().commands();
        cmds.spawn(Node::default()).with_children(|parent| {
            spawn_upgrade_card(
                parent,
                0,
                &choice,
                None,
                Handle::default(),
                Language::English,
            );
        });
        app.world_mut().flush();

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, (With<Button>, With<UpgradeCardHud>)>();
        assert_eq!(
            q.iter(app.world()).count(),
            1,
            "spawn_upgrade_card must produce one Button+UpgradeCardHud entity"
        );
    }

    /// Icon placeholder node is spawned with the correct size and category color.
    #[test]
    fn spawn_upgrade_card_icon_placeholder_has_correct_size_and_color() {
        use bevy::state::app::StatesPlugin;
        use vs_core::states::AppState;

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<AppState>();

        // Test each upgrade category to verify the color mapping.
        let cases: &[(&str, UpgradeChoice, Color)] = &[
            (
                "NewWeapon",
                UpgradeChoice::NewWeapon(WeaponType::Whip),
                ICON_COLOR_NEW_WEAPON,
            ),
            (
                "WeaponUpgrade",
                UpgradeChoice::WeaponUpgrade(WeaponType::Whip),
                ICON_COLOR_WEAPON_UPGRADE,
            ),
            (
                "PassiveItem",
                UpgradeChoice::PassiveItem(PassiveItemType::Spinach),
                ICON_COLOR_NEW_PASSIVE,
            ),
            (
                "PassiveUpgrade",
                UpgradeChoice::PassiveUpgrade(PassiveItemType::Spinach),
                ICON_COLOR_PASSIVE_UPGRADE,
            ),
        ];

        for (label, choice, expected_color) in cases {
            // Spawn a fresh card for each case.
            let mut cmds = app.world_mut().commands();
            cmds.spawn(Node::default()).with_children(|parent| {
                spawn_upgrade_card(
                    parent,
                    0,
                    choice,
                    None,
                    Handle::default(),
                    Language::English,
                );
            });
            app.world_mut().flush();

            // The icon placeholder is a Node with a BackgroundColor whose size
            // equals DEFAULT_ICON_SIZE.  Find all background-colored nodes and
            // verify at least one matches the expected icon dimensions and color.
            let mut q = app.world_mut().query::<(&Node, &BackgroundColor)>();
            let icon_found = q.iter(app.world()).any(|(node, bg)| {
                node.width == Val::Px(DEFAULT_ICON_SIZE.max(8.0))
                    && node.height == Val::Px(DEFAULT_ICON_SIZE.max(8.0))
                    && bg.0 == *expected_color
            });
            assert!(
                icon_found,
                "{label}: icon node with size {}px and expected color not found",
                DEFAULT_ICON_SIZE
            );

            // Clean up spawned entities before the next iteration.
            let entities: Vec<Entity> = app.world_mut().iter_entities().map(|e| e.id()).collect();
            for e in entities {
                app.world_mut().despawn(e);
            }
            app.world_mut().flush();
        }
    }
}
