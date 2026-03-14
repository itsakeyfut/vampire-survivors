//! Stage select screen.
//!
//! Displays three stage cards (Mad Forest, Inlaid Library, Dairy Plant).
//! Clicking a card selects the stage; the detail panel updates to show
//! enemy types and difficulty multipliers.  Two buttons at the bottom
//! start the run (→ Playing) or return to CharacterSelect.
//!
//! ## Systems
//!
//! | System | Schedule | Purpose |
//! |--------|----------|---------|
//! | [`setup_stage_select_screen`] | `OnEnter(StageSelect)` | Spawn all UI entities |
//! | [`handle_stage_card_interaction`] | `Update` | Set [`SelectedStage`] on card press |
//! | [`update_stage_select`] | `Update` | Refresh card colors and detail panel |
//!
//! All entities carry [`DespawnOnExit`]`(AppState::StageSelect)` and are
//! cleaned up automatically when the state transitions away.

use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;
use vs_core::config::StageParams;
use vs_core::resources::{GameSettings, SelectedStage};
use vs_core::states::AppState;
use vs_core::types::{EnemyType, StageType};

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
/// Card background for the currently selected stage.
const DEFAULT_CARD_COLOR_SELECTED: Color = Color::srgb(0.300, 0.500, 0.900);
/// Card background for unselected stages.
const DEFAULT_CARD_COLOR_UNSELECTED: Color = Color::srgb(0.133, 0.200, 0.400);
/// Card background on hover.
const DEFAULT_CARD_COLOR_HOVER: Color = Color::srgb(0.200, 0.350, 0.650);
/// Card background on press.
const DEFAULT_CARD_COLOR_PRESSED: Color = Color::srgb(0.086, 0.133, 0.267);

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

/// Marks a stage-card [`Button`].
///
/// Stores the associated [`StageType`] so that interaction and update systems
/// can match this card to the currently selected stage.
#[derive(Component, Debug, Clone, Copy)]
pub struct StageCardButton(pub StageType);

/// Marks the [`Text`] entity inside the stage detail panel.
#[derive(Component, Debug)]
pub struct StageDetailText;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns a short, comma-separated string listing enemy names for a stage.
fn enemy_list_str(enemy_types: &[EnemyType]) -> String {
    enemy_types
        .iter()
        .map(|e| match e {
            EnemyType::Bat => "Bat",
            EnemyType::Skeleton => "Skeleton",
            EnemyType::Zombie => "Zombie",
            EnemyType::Ghost => "Ghost",
            EnemyType::Demon => "Demon",
            EnemyType::Medusa => "Medusa",
            EnemyType::Dragon => "Dragon",
            EnemyType::BossDeath => "Boss Death",
            EnemyType::MiniDeath => "Mini Death",
            EnemyType::MiniBoss => "Mini Boss",
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Returns the i18n key for a stage's difficulty label.
fn difficulty_key(stage: StageType) -> &'static str {
    match stage {
        StageType::MadForest => "stage_difficulty_easy",
        StageType::InlaidLibrary => "stage_difficulty_medium",
        StageType::DairyPlant => "stage_difficulty_hard",
    }
}

/// Returns the i18n key for a stage's display name.
fn stage_name_key(stage: StageType) -> &'static str {
    match stage {
        StageType::MadForest => "stage_mad_forest",
        StageType::InlaidLibrary => "stage_inlaid_library",
        StageType::DairyPlant => "stage_dairy_plant",
    }
}

/// Builds the detail panel text for the selected stage.
fn build_detail_text(
    stage: StageType,
    stage_params: &StageParams,
    lang: vs_core::resources::Language,
) -> String {
    let name = t(stage_name_key(stage), lang);
    let difficulty = t(difficulty_key(stage), lang);

    if let Some(cfg) = stage_params.get() {
        let entry = cfg.entry_for(stage);
        let enemies = enemy_list_str(&entry.enemy_types);
        format!(
            "{}\n{}\n{}: {}\n{}: ×{:.1}  |  {}: ×{:.1}",
            name,
            difficulty,
            t("stage_enemies_label", lang),
            enemies,
            t("stage_hp_label", lang),
            entry.enemy_hp_multiplier,
            t("stage_speed_label", lang),
            entry.enemy_speed_multiplier,
        )
    } else {
        // Config not yet loaded — show minimal info.
        format!("{}\n{}", name, difficulty)
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns the stage-select screen when entering [`AppState::StageSelect`].
#[allow(clippy::too_many_arguments)]
pub fn setup_stage_select_screen(
    mut commands: Commands,
    ui_style: UiStyleParams,
    heading_cfg: ScreenHeadingHudParams,
    btn_cfg: MenuButtonHudParams,
    cs_params: CharacterSelectScreenParams,
    stage_params: StageParams,
    asset_server: Option<Res<AssetServer>>,
    settings: Option<Res<GameSettings>>,
    selected: Option<Res<SelectedStage>>,
) {
    let lang = settings.as_deref().map(|s| s.language).unwrap_or_default();
    let font: Handle<Font> = asset_server
        .map(|s| s.load(font_for_lang(lang)))
        .unwrap_or_default();

    let bg_color = ui_style.bg_color();
    let title_color = ui_style.title_color();
    let heading_font_size = heading_cfg.font_size();
    let heading_margin = heading_cfg.margin_bottom();

    // Reuse character-select card dimensions for visual consistency.
    let card_w = cs_params.card_width();
    let card_h = cs_params.card_height();
    let card_gap = cs_params.card_gap();
    let card_name_font_size = cs_params.card_name_font_size();
    let detail_bg_color = cs_params.detail_bg_color();
    let detail_text_color = cs_params.detail_text_color();
    let detail_font_size = cs_params.detail_font_size();
    let detail_panel_w = cs_params.detail_panel_width();

    let current_selected = selected
        .as_deref()
        .map(|s| s.0)
        .unwrap_or(StageType::MadForest);

    let all_stages = [
        StageType::MadForest,
        StageType::InlaidLibrary,
        StageType::DairyPlant,
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
            DespawnOnExit(AppState::StageSelect),
        ))
        .with_children(|root| {
            // ── Heading ───────────────────────────────────────────────────
            root.spawn((
                Text::new(t("stage_select_title", lang)),
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

            // ── Stage card row ────────────────────────────────────────────
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(card_gap),
                ..default()
            })
            .with_children(|row| {
                for stage in all_stages {
                    let is_selected = stage == current_selected;
                    let card_color = if is_selected {
                        DEFAULT_CARD_COLOR_SELECTED
                    } else {
                        DEFAULT_CARD_COLOR_UNSELECTED
                    };
                    let name = t(stage_name_key(stage), lang);

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
                        StageCardButton(stage),
                    ))
                    .with_children(|card| {
                        card.spawn((
                            Text::new(name),
                            TextFont {
                                font: font.clone(),
                                font_size: card_name_font_size,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            TextLayout::new_with_linebreak(LineBreak::WordBoundary),
                        ));
                    });
                }
            });

            // ── Detail panel ──────────────────────────────────────────────
            let detail_content = build_detail_text(current_selected, &stage_params, lang);

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
                    TextColor(detail_text_color),
                    StageDetailText,
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
                    t("btn_select_stage", lang),
                    ButtonAction::StartGame,
                    btn_cfg.get(),
                    font.clone(),
                    None,
                );
                spawn_large_menu_button(
                    btns,
                    t("btn_back", lang),
                    ButtonAction::GoToCharacterSelect,
                    btn_cfg.get(),
                    font.clone(),
                    Some("btn_back"),
                );
            });
        });
}

/// Sets [`SelectedStage`] to the pressed card's stage type.
pub fn handle_stage_card_interaction(
    card_q: Query<(&Interaction, &StageCardButton), Changed<Interaction>>,
    selected: Option<ResMut<SelectedStage>>,
) {
    let Some(mut selected) = selected else {
        return;
    };
    for (interaction, card) in card_q.iter() {
        if *interaction == Interaction::Pressed {
            selected.0 = card.0;
        }
    }
}

/// Updates card background colors and detail panel text every frame.
pub fn update_stage_select(
    selected: Option<Res<SelectedStage>>,
    settings: Option<Res<GameSettings>>,
    stage_params: StageParams,
    mut detail_q: Query<(&mut Text, &mut TextColor), With<StageDetailText>>,
    mut card_q: Query<(&StageCardButton, &Interaction, &mut BackgroundColor)>,
) {
    let Some(selected) = selected else {
        return;
    };
    let lang = settings.as_deref().map(|s| s.language).unwrap_or_default();

    let stage = selected.0;
    let content = build_detail_text(stage, &stage_params, lang);

    if let Ok((mut text, mut color)) = detail_q.single_mut() {
        *text = Text::new(content);
        color.0 = Color::srgb(0.9, 0.9, 0.9);
    }

    for (card, interaction, mut bg) in card_q.iter_mut() {
        let card_selected = card.0 == stage;
        let color = match interaction {
            Interaction::Pressed => DEFAULT_CARD_COLOR_PRESSED,
            Interaction::Hovered => DEFAULT_CARD_COLOR_HOVER,
            Interaction::None => {
                if card_selected {
                    DEFAULT_CARD_COLOR_SELECTED
                } else {
                    DEFAULT_CARD_COLOR_UNSELECTED
                }
            }
        };
        *bg = BackgroundColor(color);
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
        app.insert_resource(SelectedStage::default());
        app.insert_resource(GameSettings::default());
        app
    }

    fn enter_stage_select(app: &mut App) {
        // Transition to CharacterSelect first (since Loading is default),
        // then to StageSelect so OnEnter fires.
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::CharacterSelect);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::StageSelect);
        app.update();
    }

    #[test]
    fn setup_spawns_heading() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::StageSelect), setup_stage_select_screen);
        enter_stage_select(&mut app);

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<ScreenHeadingHud>>();
        assert_eq!(
            q.iter(app.world()).count(),
            1,
            "exactly one heading expected"
        );
    }

    #[test]
    fn setup_spawns_three_stage_cards() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::StageSelect), setup_stage_select_screen);
        enter_stage_select(&mut app);

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<StageCardButton>>();
        assert_eq!(
            q.iter(app.world()).count(),
            3,
            "one card per StageType expected"
        );
    }

    #[test]
    fn setup_spawns_detail_text() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::StageSelect), setup_stage_select_screen);
        enter_stage_select(&mut app);

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<StageDetailText>>();
        assert_eq!(
            q.iter(app.world()).count(),
            1,
            "exactly one detail text expected"
        );
    }

    #[test]
    fn setup_spawns_start_and_back_buttons() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::StageSelect), setup_stage_select_screen);
        enter_stage_select(&mut app);

        let mut q = app.world_mut().query::<&MenuButton>();
        let actions: Vec<ButtonAction> = q.iter(app.world()).map(|b| b.action).collect();
        assert!(
            actions.contains(&ButtonAction::StartGame),
            "Start button must exist"
        );
        assert!(
            actions.contains(&ButtonAction::GoToCharacterSelect),
            "Back button must exist"
        );
    }

    #[test]
    fn default_card_is_mad_forest_highlighted() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::StageSelect), setup_stage_select_screen);
        enter_stage_select(&mut app);

        let mut q = app
            .world_mut()
            .query::<(&StageCardButton, &BackgroundColor)>();
        let mad_forest_bg = q
            .iter(app.world())
            .find(|(card, _)| card.0 == StageType::MadForest)
            .map(|(_, bg)| bg.0)
            .expect("MadForest card must exist");
        assert_eq!(
            mad_forest_bg, DEFAULT_CARD_COLOR_SELECTED,
            "initially selected card must use DEFAULT_CARD_COLOR_SELECTED"
        );
    }

    #[test]
    fn card_press_updates_selected_stage() {
        let mut app = build_app();
        let card_entity = app
            .world_mut()
            .spawn((Interaction::Pressed, StageCardButton(StageType::DairyPlant)))
            .id();

        app.world_mut()
            .run_system_once(handle_stage_card_interaction)
            .unwrap();

        let selected = app.world().resource::<SelectedStage>();
        assert_eq!(
            selected.0,
            StageType::DairyPlant,
            "pressing a card must update SelectedStage"
        );

        app.world_mut().despawn(card_entity);
    }
}
