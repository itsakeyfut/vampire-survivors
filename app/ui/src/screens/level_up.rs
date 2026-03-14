//! Level-up upgrade card selection screen.
//!
//! Shown when the game enters [`AppState::LevelUp`].  Displays up to three
//! upgrade cards read from [`LevelUpChoices`], composed from HUD widget functions.
//! Clicking a card resumes the run by transitioning back to [`AppState::Playing`].
//!
//! All entities carry [`DespawnOnExit`]`(`[`AppState::LevelUp`]`)` so Bevy
//! cleans them up automatically when the state leaves.

use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;
use vs_core::resources::{GameData, GameSettings, LevelUpChoices};
use vs_core::states::AppState;

use crate::config::{LevelUpScreenParams, ScreenHeadingHudParams, UpgradeCardHudParams};
use crate::hud::screen_heading::spawn_screen_heading;
use crate::hud::upgrade_card::spawn_upgrade_card;
use crate::i18n::{font_for_lang, t};

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

/// Marks the root overlay node of the level-up screen.
#[derive(Component)]
pub struct LevelUpScreenBg;

/// Marks the row container that holds all upgrade cards.
#[derive(Component)]
pub struct LevelUpCardRow;

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Spawns the level-up card selection overlay when entering [`AppState::LevelUp`].
///
/// Reads [`LevelUpChoices`] and spawns one interactive card per choice via
/// [`spawn_upgrade_card`].  Each card is tagged with
/// [`DespawnOnExit`]`(`[`AppState::LevelUp`]`)` via the root overlay node.
#[allow(clippy::too_many_arguments)]
pub fn setup_level_up_screen(
    mut commands: Commands,
    choices: Res<LevelUpChoices>,
    game_data: Res<GameData>,
    mut next_state: ResMut<NextState<AppState>>,
    screen_cfg: LevelUpScreenParams,
    heading_cfg: ScreenHeadingHudParams,
    card_cfg: UpgradeCardHudParams,
    asset_server: Option<Res<AssetServer>>,
    settings: Option<Res<GameSettings>>,
) {
    // Guard against an empty pool (all items maxed, or player query failed).
    // Without cards there is no way to dismiss the overlay, which would
    // soft-lock the game.  Return to Playing immediately in that case.
    if choices.choices.is_empty() {
        next_state.set(AppState::Playing);
        return;
    }

    let lang = settings.as_deref().map(|s| s.language).unwrap_or_default();
    let font: Handle<Font> = asset_server
        .map(|s| s.load(font_for_lang(lang)))
        .unwrap_or_default();

    let overlay_color = screen_cfg.overlay_color();
    let heading_color = screen_cfg.heading_color();
    let card_gap = card_cfg.card_gap().max(0.0);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(40.0),
                ..default()
            },
            BackgroundColor(overlay_color),
            DespawnOnExit(AppState::LevelUp),
            LevelUpScreenBg,
        ))
        .with_children(|root| {
            // "LEVEL UP! Lv.X" heading — gold; color is screen-specific.
            // The level number is formatted at spawn time from GameData.
            let heading_text = format!(
                "{} Lv.{}",
                t("level_up_title", lang),
                game_data.current_level
            );
            spawn_screen_heading(
                root,
                &heading_text,
                heading_color,
                heading_cfg.get(),
                font.clone(),
            );

            // Row of upgrade cards.
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(card_gap),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Stretch,
                    ..default()
                },
                LevelUpCardRow,
            ))
            .with_children(|row| {
                for (i, choice) in choices.choices.iter().enumerate() {
                    spawn_upgrade_card(row, i, choice, card_cfg.get(), font.clone(), lang);
                }
            });
        });
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;
    use vs_core::types::{PassiveItemType, UpgradeChoice, WeaponType};

    use super::*;
    use crate::components::{ButtonAction, MenuButton};
    use crate::hud::upgrade_card::UpgradeCardHud;

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<AppState>();
        app.insert_resource(LevelUpChoices::default());
        app.insert_resource(GameData::default());
        app
    }

    fn enter_level_up(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::LevelUp);
        app.update();
        app.update();
    }

    fn populate_choices(app: &mut App, choices: Vec<UpgradeChoice>) {
        app.world_mut().resource_mut::<LevelUpChoices>().choices = choices;
    }

    /// No choices — soft-lock guard returns to Playing without spawning the overlay.
    #[test]
    fn setup_level_up_screen_with_no_choices_returns_to_playing() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::LevelUp), setup_level_up_screen);

        enter_level_up(&mut app);

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<LevelUpScreenBg>>();
        assert_eq!(
            q.iter(app.world()).count(),
            0,
            "no LevelUpScreenBg should exist when choices are empty"
        );
        assert_eq!(
            *app.world().resource::<State<AppState>>(),
            AppState::Playing,
            "state should return to Playing when there are no choices"
        );
    }

    /// Three choices produce exactly three UpgradeCardHud entities.
    #[test]
    fn setup_level_up_screen_spawns_one_card_per_choice() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::LevelUp), setup_level_up_screen);

        populate_choices(
            &mut app,
            vec![
                UpgradeChoice::NewWeapon(WeaponType::Whip),
                UpgradeChoice::PassiveItem(PassiveItemType::Spinach),
                UpgradeChoice::WeaponUpgrade(WeaponType::MagicWand),
            ],
        );

        enter_level_up(&mut app);

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<UpgradeCardHud>>();
        assert_eq!(
            q.iter(app.world()).count(),
            3,
            "exactly 3 UpgradeCardHud entities expected for 3 choices"
        );
    }

    /// Cards are interactive buttons.
    #[test]
    fn level_up_cards_are_buttons() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::LevelUp), setup_level_up_screen);

        populate_choices(
            &mut app,
            vec![
                UpgradeChoice::NewWeapon(WeaponType::Whip),
                UpgradeChoice::PassiveItem(PassiveItemType::Wings),
            ],
        );

        enter_level_up(&mut app);

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, (With<Button>, With<UpgradeCardHud>)>();
        assert_eq!(
            q.iter(app.world()).count(),
            2,
            "each card must be a Bevy Button"
        );
    }

    /// Cards carry SelectUpgrade actions with the correct index.
    #[test]
    fn cards_have_select_upgrade_actions_with_correct_indices() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::LevelUp), setup_level_up_screen);

        populate_choices(
            &mut app,
            vec![
                UpgradeChoice::NewWeapon(WeaponType::Knife),
                UpgradeChoice::PassiveItem(PassiveItemType::Clover),
            ],
        );

        enter_level_up(&mut app);

        let mut q = app.world_mut().query::<(&UpgradeCardHud, &MenuButton)>();
        let mut pairs: Vec<(usize, ButtonAction)> = q
            .iter(app.world())
            .map(|(card, btn)| (card.index, btn.action))
            .collect();
        pairs.sort_by_key(|(i, _)| *i);

        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0], (0, ButtonAction::SelectUpgrade(0)));
        assert_eq!(pairs[1], (1, ButtonAction::SelectUpgrade(1)));
    }

    /// Screen despawns when leaving LevelUp state.
    #[test]
    fn level_up_screen_despawns_on_exit() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::LevelUp), setup_level_up_screen);

        populate_choices(&mut app, vec![UpgradeChoice::NewWeapon(WeaponType::Whip)]);

        enter_level_up(&mut app);

        {
            let mut q = app
                .world_mut()
                .query_filtered::<Entity, With<LevelUpScreenBg>>();
            assert_eq!(q.iter(app.world()).count(), 1);
        }

        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Playing);
        app.update();
        app.update();

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<LevelUpScreenBg>>();
        assert_eq!(
            q.iter(app.world()).count(),
            0,
            "level-up overlay must despawn after leaving LevelUp state"
        );
    }

    /// Heading text includes the current level number from GameData.
    #[test]
    fn heading_includes_current_level() {
        let mut app = build_app();
        app.world_mut().resource_mut::<GameData>().current_level = 7;
        app.add_systems(OnEnter(AppState::LevelUp), setup_level_up_screen);
        populate_choices(&mut app, vec![UpgradeChoice::NewWeapon(WeaponType::Whip)]);
        enter_level_up(&mut app);

        let mut q = app.world_mut().query::<&Text>();
        let texts: Vec<String> = q.iter(app.world()).map(|t| t.0.clone()).collect();
        assert!(
            texts.iter().any(|t| t.contains("Lv.7")),
            "heading must contain 'Lv.7'; got: {texts:?}"
        );
    }
}
