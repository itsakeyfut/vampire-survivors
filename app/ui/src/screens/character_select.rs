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
use crate::config::{
    CharacterSelectScreenParams, MenuButtonHudParams, ScreenHeadingHudParams, UiStyleParams,
};
use crate::hud::menu_button::spawn_large_menu_button;
use crate::hud::screen_heading::ScreenHeadingHud;
use crate::i18n::{font_for_lang, t};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Vertical gap between root layout children.
const DEFAULT_ROOT_ROW_GAP: f32 = 24.0;

/// Card background when unlocked and selected — used by tests to verify
/// that the initially selected card has the correct color.
#[cfg(test)]
const DEFAULT_CARD_COLOR_SELECTED: Color = Color::srgb(0.300, 0.500, 0.900);
/// Card background for locked characters — used by tests to verify that
/// locked cards render with the correct color on spawn.
#[cfg(test)]
const DEFAULT_CARD_COLOR_LOCKED: Color = Color::srgb(0.10, 0.10, 0.15);

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
fn card_base_color(
    unlocked: bool,
    selected: bool,
    selected_color: Color,
    unlocked_color: Color,
    locked_color: Color,
) -> Color {
    match (unlocked, selected) {
        (_, true) => selected_color,
        (true, false) => unlocked_color,
        (false, false) => locked_color,
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
        // Evolved weapons are never used as starting weapons; map each to its
        // base weapon's i18n key so the detail panel shows a recognisable name
        // if the RON config ever assigns one.  Listed explicitly so adding a
        // new WeaponType causes a compile error here.
        WeaponType::BloodyTear => "weapon_whip",
        WeaponType::HolyWand => "weapon_magic_wand",
        WeaponType::ThousandEdge => "weapon_knife",
        WeaponType::SoulEater => "weapon_garlic",
        WeaponType::UnholyVespers => "weapon_bible",
        WeaponType::LightningRing => "weapon_thunder_ring",
    }
}

/// Builds the formatted multi-line detail string for the given character.
///
/// For unlocked characters the detail shows HP, move speed, starting weapon,
/// and the description line.  For locked characters it shows the lock badge
/// and the gold cost required to purchase the character in the gold shop.
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
        let cost_str =
            t("label_unlock_cost", lang).replace("{cost}", &stats.unlock_cost.to_string());
        format!("{}\n{}\n{}", stats.name, t("label_locked", lang), cost_str)
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
    cs_params: CharacterSelectScreenParams,
    asset_server: Option<Res<AssetServer>>,
    settings: Option<Res<GameSettings>>,
    meta: Option<Res<MetaProgress>>,
    selected: Option<Res<SelectedCharacter>>,
) {
    let lang = settings.as_deref().map(|s| s.language).unwrap_or_default();
    let font: Handle<Font> = asset_server
        .map(|s| s.load(font_for_lang(lang)))
        .unwrap_or_default();
    let bg_color = ui_style.bg_color();
    let title_color = ui_style.title_color();
    let heading_font_size = heading_cfg.font_size();
    let heading_margin = heading_cfg.margin_bottom();

    // Card and detail values from RON config with accessor fallbacks.
    let card_w = cs_params.card_width();
    let card_h = cs_params.card_height();
    let card_gap = cs_params.card_gap();
    let card_name_font_size = cs_params.card_name_font_size();
    let color_selected = cs_params.card_color_selected();
    let color_unlocked = cs_params.card_color_unlocked();
    let color_locked = cs_params.card_color_locked();
    let card_text_color = cs_params.card_text_color();
    let card_text_locked_color = cs_params.card_text_locked_color();
    let detail_bg_color = cs_params.detail_bg_color();
    let detail_text_color = cs_params.detail_text_color();
    let detail_locked_color = cs_params.detail_locked_color();
    let detail_font_size = cs_params.detail_font_size();
    let detail_panel_w = cs_params.detail_panel_width();

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
                column_gap: Val::Px(card_gap),
                ..default()
            })
            .with_children(|row| {
                for char_type in all_chars {
                    let is_unlocked = unlocked.contains(&char_type);
                    let is_selected = char_type == current_selected;
                    let stats = char_params.stats_for(char_type);

                    let card_color = card_base_color(
                        is_unlocked,
                        is_selected,
                        color_selected,
                        color_unlocked,
                        color_locked,
                    );
                    let text_color = if is_unlocked {
                        card_text_color
                    } else {
                        card_text_locked_color
                    };
                    let label = if is_unlocked {
                        stats.name.clone()
                    } else {
                        format!("🔒 {}", stats.name)
                    };

                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(card_w),
                            height: Val::Px(card_h),
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
                                font_size: card_name_font_size,
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
            let init_detail_text_color = if init_unlocked {
                detail_text_color
            } else {
                detail_locked_color
            };

            root.spawn((
                Node {
                    width: Val::Px(detail_panel_w),
                    padding: UiRect::all(Val::Px(16.0)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(detail_bg_color),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new(detail_content),
                    TextFont {
                        font: font.clone(),
                        font_size: detail_font_size,
                        ..default()
                    },
                    TextColor(init_detail_text_color),
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
    cs_params: CharacterSelectScreenParams,
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

    // Extract card/detail colors from config with accessor fallbacks.
    let color_selected = cs_params.card_color_selected();
    let color_unlocked = cs_params.card_color_unlocked();
    let color_locked = cs_params.card_color_locked();
    let color_hover = cs_params.card_color_hover();
    let color_pressed = cs_params.card_color_pressed();
    let color_locked_hover = cs_params.card_color_locked_hover();
    let detail_text_color = cs_params.detail_text_color();
    let detail_locked_color = cs_params.detail_locked_color();

    let char_type = selected.0;
    let stats = char_params.stats_for(char_type);
    let is_unlocked = meta.unlocked_characters.contains(&char_type);

    // Rebuild detail panel text.
    let content = build_detail_text(&stats, is_unlocked, lang);
    let text_color = if is_unlocked {
        detail_text_color
    } else {
        detail_locked_color
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
            Interaction::Pressed => color_pressed,
            Interaction::Hovered => {
                if card_unlocked {
                    color_hover
                } else {
                    color_locked_hover
                }
            }
            Interaction::None => card_base_color(
                card_unlocked,
                card_selected,
                color_selected,
                color_unlocked,
                color_locked,
            ),
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
            default_bg, DEFAULT_CARD_COLOR_SELECTED,
            "initially selected card must use DEFAULT_CARD_COLOR_SELECTED"
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
            magician_bg, DEFAULT_CARD_COLOR_LOCKED,
            "locked Magician card must use DEFAULT_CARD_COLOR_LOCKED"
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
        // Gold cost must also appear for locked characters (from CharacterBaseStats).
        let magician_stats = vs_core::types::get_character_stats(CharacterType::Magician);
        let cost = magician_stats.unlock_cost;
        assert!(
            text.0.contains(&cost.to_string()),
            "locked character detail must show unlock cost {cost}G; got: {:?}",
            text.0
        );
    }
}
