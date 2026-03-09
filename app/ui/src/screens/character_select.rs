//! Character select screen.
//!
//! Displays four character cards in a row.  Unlocked characters can be
//! selected by clicking their card; locked characters are greyed out with a
//! lock badge and show unlock instructions in the detail panel.  Below the
//! cards a detail panel displays the selected character's stats.  Two buttons
//! at the bottom confirm the selection (→ Playing) or return to the Title.
//!
//! ## Systems
//!
//! | System | Schedule | Purpose |
//! |--------|----------|---------|
//! | [`setup_character_select_screen`] | `OnEnter(CharacterSelect)` | Spawn all UI entities |
//! | [`handle_character_card_interaction`] | `Update` | Set [`SelectedCharacter`] on card press |
//! | [`update_character_select`] | `Update` | Refresh card colors and detail panel |
//!
//! All entities carry [`DespawnOnExit`]`(AppState::CharacterSelect)` and are
//! cleaned up automatically when the state transitions away.

use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;
use vs_core::config::CharacterParams;
use vs_core::resources::{GameSettings, MetaProgress, SelectedCharacter};
use vs_core::states::AppState;
use vs_core::types::{CharacterType, WeaponType};

use crate::components::ButtonAction;
use crate::config::{MenuButtonHudParams, ScreenHeadingHudParams, UiStyleParams};
use crate::hud::menu_button::spawn_large_menu_button;
use crate::hud::screen_heading::ScreenHeadingHud;
use crate::i18n::{font_for_lang, t};

// ---------------------------------------------------------------------------
// Fallback constants
// ---------------------------------------------------------------------------

/// Dark-purple background (#1a0a2e) per docs/04_ui_ux.md.
const DEFAULT_BG_COLOR: Color = Color::srgb(0.102, 0.039, 0.180);
/// Gold title color (#ffd700) per docs/04_ui_ux.md.
const DEFAULT_TITLE_COLOR: Color = Color::srgb(1.0, 0.843, 0.0);
const DEFAULT_HEADING_FONT_SIZE: f32 = 48.0;
const DEFAULT_HEADING_MARGIN: f32 = 32.0;
const DEFAULT_ROOT_ROW_GAP: f32 = 24.0;

/// Width of each character card in pixels.
const CARD_W: f32 = 160.0;
/// Height of each character card in pixels.
const CARD_H: f32 = 140.0;
/// Horizontal gap between cards.
const CARD_GAP: f32 = 16.0;
/// Font size for the character name inside each card.
const CARD_NAME_FONT_SIZE: f32 = 18.0;

/// Card background when unlocked and not selected.
const CARD_COLOR_UNLOCKED: Color = Color::srgb(0.133, 0.200, 0.400);
/// Card background when unlocked and selected.
pub(crate) const CARD_COLOR_SELECTED: Color = Color::srgb(0.300, 0.500, 0.900);
/// Card background when hovered (unlocked).
const CARD_COLOR_HOVER: Color = Color::srgb(0.200, 0.350, 0.650);
/// Card background when pressed.
const CARD_COLOR_PRESSED: Color = Color::srgb(0.086, 0.133, 0.267);
/// Card background for locked characters.
pub(crate) const CARD_COLOR_LOCKED: Color = Color::srgb(0.10, 0.10, 0.15);
/// Hover color for locked cards.
const CARD_COLOR_LOCKED_HOVER: Color = Color::srgb(0.13, 0.13, 0.18);

/// Card name text color when unlocked.
const CARD_TEXT_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);
/// Card name text color when locked.
const CARD_TEXT_LOCKED_COLOR: Color = Color::srgb(0.5, 0.5, 0.55);

/// Detail panel background color.
const DETAIL_BG_COLOR: Color = Color::srgb(0.08, 0.04, 0.16);
/// Detail panel text color for unlocked characters.
const DETAIL_TEXT_COLOR: Color = Color::srgb(0.90, 0.90, 0.90);
/// Detail panel text color when a locked character is selected.
const DETAIL_LOCKED_COLOR: Color = Color::srgb(0.50, 0.50, 0.55);
/// Font size for the detail panel text.
const DETAIL_FONT_SIZE: f32 = 20.0;
/// Width of the detail panel.
const DETAIL_PANEL_W: f32 = 580.0;

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

/// Marks a character-card [`Button`].
///
/// Stores the associated [`CharacterType`] so that
/// [`handle_character_card_interaction`] and [`update_character_select`] can
/// match this card to the currently selected character.
#[derive(Component, Debug, Clone, Copy)]
pub struct CharacterCardButton(pub CharacterType);

/// Marks the [`Text`] entity inside the character detail panel.
///
/// [`update_character_select`] queries this marker to refresh the displayed
/// stats whenever [`SelectedCharacter`] changes.
#[derive(Component, Debug)]
pub struct CharacterDetailText;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns the base background color for a card given its locked/selected state.
fn card_base_color(unlocked: bool, selected: bool) -> Color {
    match (unlocked, selected) {
        (_, true) => CARD_COLOR_SELECTED,
        (true, false) => CARD_COLOR_UNLOCKED,
        (false, false) => CARD_COLOR_LOCKED,
    }
}

/// Returns the i18n key for a given [`WeaponType`] name.
fn weapon_name_key(wt: WeaponType) -> &'static str {
    match wt {
        WeaponType::Whip => "weapon_whip",
        WeaponType::MagicWand => "weapon_magic_wand",
        WeaponType::Knife => "weapon_knife",
        WeaponType::Garlic => "weapon_garlic",
        WeaponType::Bible => "weapon_bible",
        WeaponType::ThunderRing => "weapon_thunder_ring",
        WeaponType::Cross => "weapon_cross",
        WeaponType::FireWand => "weapon_fire_wand",
        // Evolved weapons are never used as starting weapons.
        // Listed explicitly so adding a new WeaponType causes a compile error here.
        WeaponType::BloodyTear
        | WeaponType::HolyWand
        | WeaponType::ThousandEdge
        | WeaponType::SoulEater
        | WeaponType::UnholyVespers
        | WeaponType::LightningRing => "weapon_whip",
    }
}

/// Builds the formatted multi-line detail string for the given character.
fn build_detail_text(
    stats: &vs_core::types::CharacterBaseStats,
    is_unlocked: bool,
    lang: vs_core::resources::Language,
) -> String {
    if is_unlocked {
        format!(
            "{}\n{}: {}  |  {}: {}\n{}: {}\n{}",
            stats.name,
            t("label_hp", lang),
            stats.max_hp as u32,
            t("label_speed", lang),
            stats.move_speed as u32,
            t("label_weapon", lang),
            t(weapon_name_key(stats.starting_weapon), lang),
            stats.description,
        )
    } else {
        format!("{}\n{}", stats.name, t("label_locked", lang))
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns the character-select screen when entering [`AppState::CharacterSelect`].
///
/// Reads [`MetaProgress`] to determine which characters are unlocked and
/// [`SelectedCharacter`] to highlight the initially selected card.
/// [`CharacterParams`] provides RON-backed stats with hardcoded fallbacks.
#[allow(clippy::too_many_arguments)]
pub fn setup_character_select_screen(
    mut commands: Commands,
    ui_style: UiStyleParams,
    heading_cfg: ScreenHeadingHudParams,
    btn_cfg: MenuButtonHudParams,
    char_params: CharacterParams,
    asset_server: Option<Res<AssetServer>>,
    settings: Option<Res<GameSettings>>,
    meta: Option<Res<MetaProgress>>,
    selected: Option<Res<SelectedCharacter>>,
) {
    let lang = settings.as_deref().map(|s| s.language).unwrap_or_default();
    let font: Handle<Font> = asset_server
        .map(|s| s.load(font_for_lang(lang)))
        .unwrap_or_default();
    let bg_color = ui_style
        .get()
        .map(|c| Color::from(&c.bg_color))
        .unwrap_or(DEFAULT_BG_COLOR);
    let title_color = ui_style
        .get()
        .map(|c| Color::from(&c.title_color))
        .unwrap_or(DEFAULT_TITLE_COLOR);
    let heading_font_size = heading_cfg
        .get()
        .map(|c| c.font_size)
        .unwrap_or(DEFAULT_HEADING_FONT_SIZE);
    let heading_margin = heading_cfg
        .get()
        .map(|c| c.margin_bottom)
        .unwrap_or(DEFAULT_HEADING_MARGIN);

    let current_selected = selected
        .as_deref()
        .map(|s| s.0)
        .unwrap_or(CharacterType::DefaultCharacter);
    let unlocked: Vec<CharacterType> = meta
        .as_deref()
        .map(|m| m.unlocked_characters.clone())
        .unwrap_or_else(|| vec![CharacterType::DefaultCharacter]);

    let all_chars = [
        CharacterType::DefaultCharacter,
        CharacterType::Magician,
        CharacterType::Thief,
        CharacterType::Knight,
    ];

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(DEFAULT_ROOT_ROW_GAP),
                ..default()
            },
            BackgroundColor(bg_color),
            DespawnOnExit(AppState::CharacterSelect),
        ))
        .with_children(|root| {
            // ── Heading ───────────────────────────────────────────────────
            root.spawn((
                Text::new(t("character_select_title", lang)),
                TextFont {
                    font: font.clone(),
                    font_size: heading_font_size,
                    ..default()
                },
                TextColor(title_color),
                Node {
                    margin: UiRect::bottom(Val::Px(heading_margin)),
                    ..default()
                },
                ScreenHeadingHud,
            ));

            // ── Character card row ────────────────────────────────────────
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(CARD_GAP),
                ..default()
            })
            .with_children(|row| {
                for char_type in all_chars {
                    let is_unlocked = unlocked.contains(&char_type);
                    let is_selected = char_type == current_selected;
                    let stats = char_params.stats_for(char_type);

                    let card_color = card_base_color(is_unlocked, is_selected);
                    let text_color = if is_unlocked {
                        CARD_TEXT_COLOR
                    } else {
                        CARD_TEXT_LOCKED_COLOR
                    };
                    let label = if is_unlocked {
                        stats.name.clone()
                    } else {
                        format!("🔒 {}", stats.name)
                    };

                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(CARD_W),
                            height: Val::Px(CARD_H),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        BackgroundColor(card_color),
                        CharacterCardButton(char_type),
                    ))
                    .with_children(|card| {
                        card.spawn((
                            Text::new(label),
                            TextFont {
                                font: font.clone(),
                                font_size: CARD_NAME_FONT_SIZE,
                                ..default()
                            },
                            TextColor(text_color),
                            TextLayout::new_with_linebreak(LineBreak::WordBoundary),
                        ));
                    });
                }
            });

            // ── Detail panel ──────────────────────────────────────────────
            let init_stats = char_params.stats_for(current_selected);
            let init_unlocked = unlocked.contains(&current_selected);
            let detail_content = build_detail_text(&init_stats, init_unlocked, lang);
            let detail_text_color = if init_unlocked {
                DETAIL_TEXT_COLOR
            } else {
                DETAIL_LOCKED_COLOR
            };

            root.spawn((
                Node {
                    width: Val::Px(DETAIL_PANEL_W),
                    padding: UiRect::all(Val::Px(16.0)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(DETAIL_BG_COLOR),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new(detail_content),
                    TextFont {
                        font: font.clone(),
                        font_size: DETAIL_FONT_SIZE,
                        ..default()
                    },
                    TextColor(detail_text_color),
                    CharacterDetailText,
                ));
            });

            // ── Action buttons ────────────────────────────────────────────
            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(12.0),
                margin: UiRect::top(Val::Px(8.0)),
                ..default()
            })
            .with_children(|btns| {
                spawn_large_menu_button(
                    btns,
                    t("btn_start_with_char", lang),
                    ButtonAction::StartGame,
                    btn_cfg.get(),
                    font.clone(),
                    Some("btn_start_with_char"),
                );
                spawn_large_menu_button(
                    btns,
                    t("btn_back", lang),
                    ButtonAction::GoToTitle,
                    btn_cfg.get(),
                    font.clone(),
                    Some("btn_back"),
                );
            });
        });
}

/// Updates the detail panel text and card background colors every frame.
///
/// Reads [`SelectedCharacter`] and [`MetaProgress`] to rebuild the detail
/// string and recompute each card's background.  Running unconditionally keeps
/// card hover state consistent with the latest selection at minimal cost (only
/// four card entities per screen).
pub fn update_character_select(
    selected: Option<Res<SelectedCharacter>>,
    meta: Option<Res<MetaProgress>>,
    settings: Option<Res<GameSettings>>,
    char_params: CharacterParams,
    mut detail_q: Query<(&mut Text, &mut TextColor), With<CharacterDetailText>>,
    mut card_q: Query<(&CharacterCardButton, &Interaction, &mut BackgroundColor)>,
) {
    let Some(selected) = selected else {
        return;
    };
    let Some(meta) = meta else {
        return;
    };
    let lang = settings.as_deref().map(|s| s.language).unwrap_or_default();

    let char_type = selected.0;
    let stats = char_params.stats_for(char_type);
    let is_unlocked = meta.unlocked_characters.contains(&char_type);

    // Rebuild detail panel text.
    let content = build_detail_text(&stats, is_unlocked, lang);
    let text_color = if is_unlocked {
        DETAIL_TEXT_COLOR
    } else {
        DETAIL_LOCKED_COLOR
    };
    if let Ok((mut text, mut color)) = detail_q.single_mut() {
        *text = Text::new(content);
        color.0 = text_color;
    }

    // Update card background colors (respects current hover/pressed state).
    for (card, interaction, mut bg) in card_q.iter_mut() {
        let card_unlocked = meta.unlocked_characters.contains(&card.0);
        let card_selected = card.0 == char_type;
        let color = match interaction {
            Interaction::Pressed => CARD_COLOR_PRESSED,
            Interaction::Hovered => {
                if card_unlocked {
                    CARD_COLOR_HOVER
                } else {
                    CARD_COLOR_LOCKED_HOVER
                }
            }
            Interaction::None => card_base_color(card_unlocked, card_selected),
        };
        *bg = BackgroundColor(color);
    }
}

/// Sets [`SelectedCharacter`] to the pressed card's character type.
///
/// Uses `Changed<Interaction>` to only run on frames where a card is
/// actually interacted with.
pub fn handle_character_card_interaction(
    card_q: Query<(&Interaction, &CharacterCardButton), Changed<Interaction>>,
    selected: Option<ResMut<SelectedCharacter>>,
    meta: Option<Res<MetaProgress>>,
) {
    let Some(mut selected) = selected else {
        return;
    };
    for (interaction, card) in card_q.iter() {
        if *interaction == Interaction::Pressed {
            // Only allow selecting characters that have been unlocked.
            let is_unlocked = meta
                .as_deref()
                .map(|m| m.unlocked_characters.contains(&card.0))
                .unwrap_or(false);
            if is_unlocked {
                selected.0 = card.0;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce as _;
    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::components::MenuButton;

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<AppState>();
        app.insert_resource(SelectedCharacter::default());
        app.insert_resource(MetaProgress::default());
        app.insert_resource(GameSettings::default());
        app
    }

    fn enter_character_select(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::CharacterSelect);
        app.update();
    }

    // ── setup ────────────────────────────────────────────────────────────────

    #[test]
    fn setup_spawns_heading() {
        let mut app = build_app();
        app.add_systems(
            OnEnter(AppState::CharacterSelect),
            setup_character_select_screen,
        );
        enter_character_select(&mut app);

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<ScreenHeadingHud>>();
        assert_eq!(
            q.iter(app.world()).count(),
            1,
            "exactly one ScreenHeadingHud expected"
        );
    }

    #[test]
    fn setup_spawns_four_character_cards() {
        let mut app = build_app();
        app.add_systems(
            OnEnter(AppState::CharacterSelect),
            setup_character_select_screen,
        );
        enter_character_select(&mut app);

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<CharacterCardButton>>();
        assert_eq!(
            q.iter(app.world()).count(),
            4,
            "one card per CharacterType expected"
        );
    }

    #[test]
    fn setup_spawns_detail_text() {
        let mut app = build_app();
        app.add_systems(
            OnEnter(AppState::CharacterSelect),
            setup_character_select_screen,
        );
        enter_character_select(&mut app);

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<CharacterDetailText>>();
        assert_eq!(
            q.iter(app.world()).count(),
            1,
            "exactly one CharacterDetailText expected"
        );
    }

    #[test]
    fn setup_spawns_start_and_back_buttons() {
        let mut app = build_app();
        app.add_systems(
            OnEnter(AppState::CharacterSelect),
            setup_character_select_screen,
        );
        enter_character_select(&mut app);

        let mut q = app.world_mut().query::<&MenuButton>();
        let actions: Vec<ButtonAction> = q.iter(app.world()).map(|b| b.action).collect();
        assert!(
            actions.contains(&ButtonAction::StartGame),
            "Start button must exist"
        );
        assert!(
            actions.contains(&ButtonAction::GoToTitle),
            "Back button must exist"
        );
    }

    #[test]
    fn default_card_is_highlighted_on_spawn() {
        let mut app = build_app();
        app.add_systems(
            OnEnter(AppState::CharacterSelect),
            setup_character_select_screen,
        );
        enter_character_select(&mut app);

        let mut q = app
            .world_mut()
            .query::<(&CharacterCardButton, &BackgroundColor)>();
        let default_bg = q
            .iter(app.world())
            .find(|(card, _)| card.0 == CharacterType::DefaultCharacter)
            .map(|(_, bg)| bg.0)
            .expect("DefaultCharacter card must exist");
        assert_eq!(
            default_bg, CARD_COLOR_SELECTED,
            "initially selected card must use CARD_COLOR_SELECTED"
        );
    }

    #[test]
    fn locked_card_uses_locked_color_on_spawn() {
        let mut app = build_app();
        app.add_systems(
            OnEnter(AppState::CharacterSelect),
            setup_character_select_screen,
        );
        enter_character_select(&mut app);

        let mut q = app
            .world_mut()
            .query::<(&CharacterCardButton, &BackgroundColor)>();
        let magician_bg = q
            .iter(app.world())
            .find(|(card, _)| card.0 == CharacterType::Magician)
            .map(|(_, bg)| bg.0)
            .expect("Magician card must exist");
        assert_eq!(
            magician_bg, CARD_COLOR_LOCKED,
            "locked Magician card must use CARD_COLOR_LOCKED"
        );
    }

    // ── handle_character_card_interaction ───────────────────────────────────

    #[test]
    fn card_press_updates_selected_character_when_unlocked() {
        let mut app = build_app();
        // DefaultCharacter is unlocked by default (MetaProgress::default).
        let card_entity = app
            .world_mut()
            .spawn((
                Interaction::Pressed,
                CharacterCardButton(CharacterType::DefaultCharacter),
            ))
            .id();

        app.world_mut()
            .run_system_once(handle_character_card_interaction)
            .unwrap();

        let selected = app.world().resource::<SelectedCharacter>();
        assert_eq!(
            selected.0,
            CharacterType::DefaultCharacter,
            "pressing an unlocked card must update SelectedCharacter"
        );

        app.world_mut().despawn(card_entity);
    }

    #[test]
    fn card_press_does_not_select_locked_character() {
        let mut app = build_app();
        // Thief is locked in default MetaProgress.
        let card_entity = app
            .world_mut()
            .spawn((
                Interaction::Pressed,
                CharacterCardButton(CharacterType::Thief),
            ))
            .id();

        app.world_mut()
            .run_system_once(handle_character_card_interaction)
            .unwrap();

        let selected = app.world().resource::<SelectedCharacter>();
        assert_eq!(
            selected.0,
            CharacterType::DefaultCharacter,
            "pressing a locked card must NOT change SelectedCharacter"
        );

        app.world_mut().despawn(card_entity);
    }

    #[test]
    fn hover_does_not_change_selected_character() {
        let mut app = build_app();

        let card_entity = app
            .world_mut()
            .spawn((
                Interaction::Hovered,
                CharacterCardButton(CharacterType::Knight),
            ))
            .id();

        app.world_mut()
            .run_system_once(handle_character_card_interaction)
            .unwrap();

        let selected = app.world().resource::<SelectedCharacter>();
        assert_eq!(
            selected.0,
            CharacterType::DefaultCharacter,
            "hovering must not change SelectedCharacter"
        );

        app.world_mut().despawn(card_entity);
    }

    // ── update_character_select ─────────────────────────────────────────────

    #[test]
    fn update_shows_knight_stats_when_knight_selected() {
        let mut app = build_app();
        // Unlock Knight so detail shows stats (not locked message).
        app.world_mut()
            .resource_mut::<MetaProgress>()
            .unlocked_characters
            .push(CharacterType::Knight);
        app.world_mut().resource_mut::<SelectedCharacter>().0 = CharacterType::Knight;

        app.world_mut()
            .spawn((Text::new(""), TextColor::default(), CharacterDetailText));
        app.world_mut().spawn((
            Interaction::None,
            CharacterCardButton(CharacterType::DefaultCharacter),
            BackgroundColor::default(),
        ));

        app.world_mut()
            .run_system_once(update_character_select)
            .unwrap();

        let mut q = app
            .world_mut()
            .query_filtered::<&Text, With<CharacterDetailText>>();
        let text = q.single(app.world()).expect("detail text must exist");
        assert!(
            text.0.contains("150"),
            "Knight HP 150 must appear in detail; got: {:?}",
            text.0
        );
    }

    #[test]
    fn update_shows_lock_badge_for_locked_character() {
        let mut app = build_app();
        // Magician is locked by default.
        app.world_mut().resource_mut::<SelectedCharacter>().0 = CharacterType::Magician;

        app.world_mut()
            .spawn((Text::new(""), TextColor::default(), CharacterDetailText));
        app.world_mut().spawn((
            Interaction::None,
            CharacterCardButton(CharacterType::Magician),
            BackgroundColor::default(),
        ));

        app.world_mut()
            .run_system_once(update_character_select)
            .unwrap();

        let mut q = app
            .world_mut()
            .query_filtered::<&Text, With<CharacterDetailText>>();
        let text = q.single(app.world()).expect("detail text must exist");
        assert!(
            text.0.contains('🔒'),
            "locked character detail must contain lock badge; got: {:?}",
            text.0
        );
    }
}
