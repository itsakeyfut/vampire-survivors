//! Gold shop screen.
//!
//! Displays the player's total gold balance, a list of purchasable character
//! unlocks, and a list of permanent upgrades.  Items that have already been
//! purchased are grayed out.  The "Back" button returns to the Title screen.
//!
//! # Layout
//!
//! ```text
//! ┌────────────────────────────────────────────────────────────┐
//! │  ゴールドショップ                        Gold: 1500G        │  ← header row
//! │                                                            │
//! │  ── Unlock Characters ─────────────────────────────────── │
//! │  [Magician 500G]  [Thief 500G]  [Knight 1000G]           │
//! │                                                            │
//! │  ── Permanent Upgrades ────────────────────────────────── │
//! │  [+Max HP 300G]  [+Speed 300G]  [+Damage 300G]           │
//! │  [+XP Gain 300G]  [+Starting Weapon 500G]                │
//! │                                                            │
//! │                       [Back]                              │
//! └────────────────────────────────────────────────────────────┘
//! ```
//!
//! All entities are tagged with [`DespawnOnExit`]`(AppState::MetaShop)` for
//! automatic cleanup on any state transition.

use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;
use vs_core::config::{CharacterParams, GameParams};
use vs_core::resources::{GameSettings, MetaProgress};
use vs_core::states::AppState;
use vs_core::types::{CharacterType, MetaUpgradeType};

use crate::components::ButtonAction;
use crate::config::{MenuButtonHudParams, ScreenHeadingHudParams, UiStyleParams};
use crate::hud::menu_button::{
    LargeMenuButtonHud, LargeMenuButtonLabelHud, spawn_large_menu_button,
};
use crate::hud::screen_heading::ScreenHeadingHud;
use crate::i18n::{font_for_lang, t};

// ---------------------------------------------------------------------------
// Fallback constants
// ---------------------------------------------------------------------------

/// Gold display text color (warm yellow).
const DEFAULT_GOLD_TEXT_COLOR: Color = Color::srgb(1.0, 0.85, 0.20);
/// Section heading color (light gray).
const DEFAULT_SECTION_COLOR: Color = Color::srgb(0.75, 0.75, 0.75);
/// Already-purchased / unlocked button color (dark gray).
const DEFAULT_PURCHASED_COLOR: Color = Color::srgb(0.25, 0.25, 0.25);
/// Unaffordable button color (dark red-brown).
const DEFAULT_UNAFFORDABLE_COLOR: Color = Color::srgb(0.20, 0.08, 0.08);
/// Brightness multiplier applied to the base color on hover (slightly darker).
const HOVER_DARKEN: f32 = 0.72;
/// Brightness multiplier applied to the base color on press (darker still).
const PRESS_DARKEN: f32 = 0.50;

const DEFAULT_GOLD_FONT_SIZE: f32 = 22.0;
const DEFAULT_SECTION_FONT_SIZE: f32 = 20.0;
const DEFAULT_ITEM_FONT_SIZE: f32 = 20.0;
const DEFAULT_ITEM_BTN_WIDTH: f32 = 220.0;
const DEFAULT_ITEM_BTN_HEIGHT: f32 = 52.0;
const DEFAULT_ROW_GAP: f32 = 24.0;
const DEFAULT_BTN_GAP: f32 = 12.0;
const DEFAULT_PADDING: f32 = 40.0;

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

/// Marks the gold balance [`Text`] node.
///
/// [`update_meta_shop_screen`] queries this to keep the displayed amount
/// current after purchases.
#[derive(Component, Debug)]
pub struct MetaShopGoldLabel;

/// Marks a shop item button so [`update_meta_shop_screen`] can set its color
/// based on whether the item has been purchased and whether it is affordable.
#[derive(Component, Debug, Clone, Copy)]
pub enum MetaShopItemButton {
    /// A character unlock button.
    Character(CharacterType),
    /// A permanent upgrade purchase button.
    Upgrade(MetaUpgradeType),
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns the base background color for a shop item button.
///
/// `normal_color` is the theme color for an affordable, unpurchased item;
/// it is read from [`MenuButtonHudParams`] via `color_normal()`.
fn item_base_color(purchased: bool, affordable: bool, normal_color: Color) -> Color {
    if purchased {
        DEFAULT_PURCHASED_COLOR
    } else if affordable {
        normal_color
    } else {
        DEFAULT_UNAFFORDABLE_COLOR
    }
}

/// Returns a slightly darkened version of `color` for hover/press feedback.
fn darken(color: Color, factor: f32) -> Color {
    let s = color.to_srgba();
    Color::srgba(s.red * factor, s.green * factor, s.blue * factor, s.alpha)
}

/// Spawns a shop item button (character unlock or upgrade) as a child of
/// `parent`.  The button carries both a [`MenuButton`] for the interaction
/// handler and a [`MetaShopItemButton`] marker so the per-frame update system
/// can adjust its color.
fn spawn_item_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    action: ButtonAction,
    item_marker: MetaShopItemButton,
    initial_color: Color,
    font: Handle<Font>,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Auto,
                min_width: Val::Px(DEFAULT_ITEM_BTN_WIDTH),
                height: Val::Px(DEFAULT_ITEM_BTN_HEIGHT),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(16.0)),
                ..default()
            },
            BackgroundColor(initial_color),
            crate::components::MenuButton { action },
            LargeMenuButtonHud,
            item_marker,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font,
                    font_size: DEFAULT_ITEM_FONT_SIZE,
                    ..default()
                },
                TextColor(Color::WHITE),
                LargeMenuButtonLabelHud,
            ));
        });
}

// ---------------------------------------------------------------------------
// Setup system
// ---------------------------------------------------------------------------

/// Spawns the gold shop screen when entering [`AppState::MetaShop`].
#[allow(clippy::too_many_arguments)]
pub fn setup_meta_shop_screen(
    mut commands: Commands,
    meta: Res<MetaProgress>,
    settings: Res<GameSettings>,
    ui_style: UiStyleParams,
    heading_cfg: ScreenHeadingHudParams,
    btn_cfg: MenuButtonHudParams,
    asset_server: Option<Res<AssetServer>>,
    char_params: CharacterParams,
    game_params: GameParams,
) {
    let lang = settings.language;
    let font: Handle<Font> = asset_server
        .as_ref()
        .map(|s| s.load(font_for_lang(lang)))
        .unwrap_or_default();
    let bg_color = ui_style.bg_color();
    let title_color = ui_style.title_color();
    let heading_font_size = heading_cfg.font_size();
    let heading_margin = heading_cfg.margin_bottom();
    let color_normal = btn_cfg.color_normal();

    // Character unlock affordability
    let char_cost = |ct: CharacterType| char_params.stats_for(ct).unlock_cost;
    let is_unlocked = |ct: CharacterType| meta.unlocked_characters.contains(&ct);
    let char_affordable = |ct: CharacterType| meta.total_gold >= char_cost(ct);

    // Upgrade affordability
    let upgrade_cost = |ut: MetaUpgradeType| game_params.upgrade_cost(ut);
    let is_purchased = |ut: MetaUpgradeType| meta.purchased_upgrades.contains(&ut);
    let upgrade_affordable = |ut: MetaUpgradeType| meta.total_gold >= upgrade_cost(ut);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(DEFAULT_PADDING)),
                row_gap: Val::Px(DEFAULT_ROW_GAP),
                ..default()
            },
            BackgroundColor(bg_color),
            DespawnOnExit(AppState::MetaShop),
        ))
        .with_children(|root| {
            // ── Header row: title (left) + gold balance (right) ─────────
            root.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                margin: UiRect::bottom(Val::Px(heading_margin)),
                ..default()
            })
            .with_children(|header| {
                // Shop title
                header.spawn((
                    Text::new(t("meta_shop_title", lang)),
                    TextFont {
                        font: font.clone(),
                        font_size: heading_font_size,
                        ..default()
                    },
                    TextColor(title_color),
                    ScreenHeadingHud,
                ));

                // Gold balance (top-right)
                header.spawn((
                    Text::new(format!("{}: {}G", t("gold_display", lang), meta.total_gold)),
                    TextFont {
                        font: font.clone(),
                        font_size: DEFAULT_GOLD_FONT_SIZE,
                        ..default()
                    },
                    TextColor(DEFAULT_GOLD_TEXT_COLOR),
                    MetaShopGoldLabel,
                ));
            });

            // ── Character unlock section ─────────────────────────────────
            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: Val::Px(10.0),
                ..default()
            })
            .with_children(|section| {
                // Section heading
                section.spawn((
                    Text::new(t("shop_section_characters", lang)),
                    TextFont {
                        font: font.clone(),
                        font_size: DEFAULT_SECTION_FONT_SIZE,
                        ..default()
                    },
                    TextColor(DEFAULT_SECTION_COLOR),
                ));

                // Character buttons row
                section
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(DEFAULT_BTN_GAP),
                        ..default()
                    })
                    .with_children(|row| {
                        let chars = [
                            CharacterType::Magician,
                            CharacterType::Thief,
                            CharacterType::Knight,
                        ];
                        let keys = ["shop_char_magician", "shop_char_thief", "shop_char_knight"];
                        for (ct, key) in chars.into_iter().zip(keys) {
                            let cost = char_cost(ct);
                            let label = format!("{} ({}G)", t(key, lang), cost);
                            let color =
                                item_base_color(is_unlocked(ct), char_affordable(ct), color_normal);
                            spawn_item_button(
                                row,
                                &label,
                                ButtonAction::UnlockCharacter(ct),
                                MetaShopItemButton::Character(ct),
                                color,
                                font.clone(),
                            );
                        }
                    });
            });

            // ── Permanent upgrade section ────────────────────────────────
            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: Val::Px(10.0),
                ..default()
            })
            .with_children(|section| {
                // Section heading
                section.spawn((
                    Text::new(t("shop_section_upgrades", lang)),
                    TextFont {
                        font: font.clone(),
                        font_size: DEFAULT_SECTION_FONT_SIZE,
                        ..default()
                    },
                    TextColor(DEFAULT_SECTION_COLOR),
                ));

                // Upgrade buttons row
                section
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: Val::Px(DEFAULT_BTN_GAP),
                        row_gap: Val::Px(DEFAULT_BTN_GAP),
                        ..default()
                    })
                    .with_children(|row| {
                        let upgrades = [
                            (MetaUpgradeType::BonusHp, "shop_upgrade_hp"),
                            (MetaUpgradeType::BonusSpeed, "shop_upgrade_speed"),
                            (MetaUpgradeType::BonusDamage, "shop_upgrade_damage"),
                            (MetaUpgradeType::BonusXp, "shop_upgrade_xp"),
                            (MetaUpgradeType::StartingWeapon, "shop_upgrade_weapon"),
                        ];
                        for (ut, key) in upgrades {
                            let cost = upgrade_cost(ut);
                            let label = format!("{} ({}G)", t(key, lang), cost);
                            let color = item_base_color(
                                is_purchased(ut),
                                upgrade_affordable(ut),
                                color_normal,
                            );
                            spawn_item_button(
                                row,
                                &label,
                                ButtonAction::PurchaseUpgrade(ut),
                                MetaShopItemButton::Upgrade(ut),
                                color,
                                font.clone(),
                            );
                        }
                    });
            });

            // ── Spacer ───────────────────────────────────────────────────
            root.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });

            // ── Back button ──────────────────────────────────────────────
            spawn_large_menu_button(
                root,
                t("btn_back", lang),
                ButtonAction::GoToTitle,
                btn_cfg.get(),
                font.clone(),
                Some("btn_back"),
            );
        });
}

// ---------------------------------------------------------------------------
// Update system
// ---------------------------------------------------------------------------

/// Keeps the gold balance display and item button colors current.
///
/// Runs every frame while in [`AppState::MetaShop`], **after**
/// [`crate::components::handle_button_interaction`].
///
/// Each shop item button gets a color derived from its state and the current
/// [`Interaction`]:
///
/// | Item state   | None (idle)              | Hovered         | Pressed         |
/// |---|---|---|---|
/// | Purchased    | gray                     | darker gray     | darkest gray    |
/// | Affordable   | blue                     | darker blue     | darkest blue    |
/// | Unaffordable | dark-red                 | darker dark-red | darkest dark-red|
///
/// Running every frame after `handle_button_interaction` means this system
/// always has the final say on shop-item colors.
pub fn update_meta_shop_screen(
    meta: Res<MetaProgress>,
    settings: Res<GameSettings>,
    btn_cfg: MenuButtonHudParams,
    char_params: CharacterParams,
    game_params: GameParams,
    mut gold_q: Query<&mut Text, With<MetaShopGoldLabel>>,
    mut item_q: Query<(&MetaShopItemButton, &mut BackgroundColor, &Interaction)>,
) {
    let color_normal = btn_cfg.color_normal();

    // Update gold balance label only when something changed.
    if meta.is_changed() || settings.is_changed() {
        let lang = settings.language;
        if let Ok(mut text) = gold_q.single_mut() {
            *text = Text::new(format!("{}: {}G", t("gold_display", lang), meta.total_gold));
        }
    }

    // Every frame: set each item button color based on purchase state + hover.
    for (item, mut bg, interaction) in item_q.iter_mut() {
        let (purchased, affordable) = match item {
            MetaShopItemButton::Character(ct) => (
                meta.unlocked_characters.contains(ct),
                meta.total_gold >= char_params.stats_for(*ct).unlock_cost,
            ),
            MetaShopItemButton::Upgrade(ut) => (
                meta.purchased_upgrades.contains(ut),
                meta.total_gold >= game_params.upgrade_cost(*ut),
            ),
        };
        let base = item_base_color(purchased, affordable, color_normal);
        let color = match interaction {
            Interaction::Pressed => darken(base, PRESS_DARKEN),
            Interaction::Hovered => darken(base, HOVER_DARKEN),
            Interaction::None => base,
        };
        *bg = BackgroundColor(color);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::components::MenuButton;

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<AppState>();
        app.insert_resource(MetaProgress::default());
        app.insert_resource(GameSettings::default());
        app
    }

    fn enter_meta_shop(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Playing);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::MetaShop);
        app.update();
        app.update();
    }

    /// Screen heading is spawned exactly once.
    #[test]
    fn setup_spawns_heading() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::MetaShop), setup_meta_shop_screen);
        enter_meta_shop(&mut app);

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<ScreenHeadingHud>>();
        assert_eq!(
            q.iter(app.world()).count(),
            1,
            "meta shop must have exactly one ScreenHeadingHud"
        );
    }

    /// Back button navigates to Title.
    #[test]
    fn back_button_goes_to_title() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::MetaShop), setup_meta_shop_screen);
        enter_meta_shop(&mut app);

        let mut q = app.world_mut().query::<&MenuButton>();
        let actions: Vec<ButtonAction> = q.iter(app.world()).map(|b| b.action).collect();
        assert!(
            actions.contains(&ButtonAction::GoToTitle),
            "meta shop back button must use GoToTitle"
        );
    }

    /// Three character unlock buttons are spawned.
    #[test]
    fn setup_spawns_three_character_buttons() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::MetaShop), setup_meta_shop_screen);
        enter_meta_shop(&mut app);

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<MetaShopItemButton>>();
        let item_count = q.iter(app.world()).count();
        // 3 characters + 5 upgrades = 8 shop item buttons
        assert_eq!(
            item_count, 8,
            "must have 8 shop item buttons (3 chars + 5 upgrades)"
        );
    }

    /// Unlock-character actions are present for all three locked characters.
    #[test]
    fn character_buttons_have_unlock_actions() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::MetaShop), setup_meta_shop_screen);
        enter_meta_shop(&mut app);

        let mut q = app.world_mut().query::<&MenuButton>();
        let actions: Vec<ButtonAction> = q.iter(app.world()).map(|b| b.action).collect();

        assert!(
            actions.contains(&ButtonAction::UnlockCharacter(CharacterType::Magician)),
            "Magician unlock button missing"
        );
        assert!(
            actions.contains(&ButtonAction::UnlockCharacter(CharacterType::Thief)),
            "Thief unlock button missing"
        );
        assert!(
            actions.contains(&ButtonAction::UnlockCharacter(CharacterType::Knight)),
            "Knight unlock button missing"
        );
    }

    /// All five permanent upgrade buttons are present.
    #[test]
    fn upgrade_buttons_have_purchase_actions() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::MetaShop), setup_meta_shop_screen);
        enter_meta_shop(&mut app);

        let mut q = app.world_mut().query::<&MenuButton>();
        let actions: Vec<ButtonAction> = q.iter(app.world()).map(|b| b.action).collect();

        for ut in [
            MetaUpgradeType::BonusHp,
            MetaUpgradeType::BonusSpeed,
            MetaUpgradeType::BonusDamage,
            MetaUpgradeType::BonusXp,
            MetaUpgradeType::StartingWeapon,
        ] {
            assert!(
                actions.contains(&ButtonAction::PurchaseUpgrade(ut)),
                "{ut:?} purchase button missing"
            );
        }
    }

    /// A gold balance label is spawned.
    #[test]
    fn setup_spawns_gold_label() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::MetaShop), setup_meta_shop_screen);
        enter_meta_shop(&mut app);

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<MetaShopGoldLabel>>();
        assert_eq!(
            q.iter(app.world()).count(),
            1,
            "must have exactly one MetaShopGoldLabel"
        );
    }

    /// Purchased items have the gray color.
    #[test]
    fn purchased_item_has_gray_color() {
        let mut app = build_app();
        // Pre-purchase BonusHp
        app.world_mut()
            .resource_mut::<MetaProgress>()
            .purchased_upgrades
            .push(MetaUpgradeType::BonusHp);
        app.add_systems(OnEnter(AppState::MetaShop), setup_meta_shop_screen);
        enter_meta_shop(&mut app);

        let mut q = app
            .world_mut()
            .query::<(&MetaShopItemButton, &BackgroundColor)>();
        let hp_btn = q.iter(app.world()).find(|(item, _)| {
            matches!(item, MetaShopItemButton::Upgrade(MetaUpgradeType::BonusHp))
        });
        let (_, bg) = hp_btn.expect("BonusHp button must exist");
        assert_eq!(
            bg.0, DEFAULT_PURCHASED_COLOR,
            "purchased item must use gray color"
        );
    }
}
