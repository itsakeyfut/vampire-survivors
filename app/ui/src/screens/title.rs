//! Title screen — the first screen the player sees when the game starts.
//!
//! Spawns a full-screen layout containing the game title, a gold display,
//! a "Start Game" button (→ CharacterSelect), and a "Gold Shop" button
//! (→ MetaShop).  All entities are tagged with
//! [`DespawnOnExit`]`(AppState::Title)` so Bevy automatically despawns them
//! when the state transitions away from `Title`.
//!
//! - Background and title text colour: [`UiStyleParams`]
//! - Heading font size / margin: [`ScreenHeadingHudParams`]
//! - Button dimensions and colors: [`MenuButtonHudParams`]

use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;
use vs_core::resources::MetaProgress;
use vs_core::states::AppState;

use crate::components::ButtonAction;
use crate::config::{
    MenuButtonHudParams, ScreenHeadingHudParams, TitleHeadingText, TitleScreenBg, UiStyleParams,
};
use crate::hud::menu_button::spawn_large_menu_button;
use crate::hud::screen_heading::ScreenHeadingHud;
use crate::styles::{DEFAULT_BG_COLOR, DEFAULT_TITLE_COLOR};

// ---------------------------------------------------------------------------
// Fallback constants
// ---------------------------------------------------------------------------

const DEFAULT_FONT_SIZE: f32 = 72.0;
const DEFAULT_MARGIN_BOTTOM: f32 = 80.0;
/// Gap between the title column's children (heading, gold label, buttons).
const DEFAULT_BUTTON_GAP: f32 = 16.0;
const DEFAULT_GOLD_FONT_SIZE: f32 = 20.0;
const DEFAULT_GOLD_TEXT_COLOR: Color = Color::srgb(1.0, 0.85, 0.20);

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

/// Marks the gold display [`Text`] node on the title screen.
///
/// [`update_title_gold`] queries this marker to update the displayed amount
/// whenever [`MetaProgress`] changes.
#[derive(Component, Debug)]
pub struct TitleGoldLabel;

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns the title screen UI when entering [`AppState::Title`].
pub fn setup_title_screen(
    mut commands: Commands,
    meta: Res<MetaProgress>,
    ui_style: UiStyleParams,
    heading_cfg: ScreenHeadingHudParams,
    btn_cfg: MenuButtonHudParams,
) {
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
        .unwrap_or(DEFAULT_FONT_SIZE);
    let heading_margin = heading_cfg
        .get()
        .map(|c| c.margin_bottom)
        .unwrap_or(DEFAULT_MARGIN_BOTTOM);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(DEFAULT_BUTTON_GAP),
                ..default()
            },
            BackgroundColor(bg_color),
            DespawnOnExit(AppState::Title),
            TitleScreenBg,
        ))
        .with_children(|parent| {
            // Game title heading.
            // Carries both the generic HUD marker (ScreenHeadingHud) for
            // hot_reload_screen_heading_hud and the screen-specific TitleHeadingText
            // marker so hot_reload_ui_style can update the title color.
            parent.spawn((
                Text::new("Vampire Survivors"),
                TextFont {
                    font_size: heading_font_size,
                    ..default()
                },
                TextColor(title_color),
                Node {
                    margin: UiRect::bottom(Val::Px(heading_margin)),
                    ..default()
                },
                ScreenHeadingHud,
                TitleHeadingText,
            ));

            // Gold display — updated when MetaProgress changes by update_title_gold.
            parent.spawn((
                Text::new(format!("Gold: {}", meta.total_gold)),
                TextFont {
                    font_size: DEFAULT_GOLD_FONT_SIZE,
                    ..default()
                },
                TextColor(DEFAULT_GOLD_TEXT_COLOR),
                TitleGoldLabel,
            ));

            // Start Game button — transitions to CharacterSelect.
            spawn_large_menu_button(
                parent,
                "Start Game",
                ButtonAction::GoToCharacterSelect,
                btn_cfg.get(),
            );

            // Gold Shop button — transitions to MetaShop.
            spawn_large_menu_button(
                parent,
                "Gold Shop",
                ButtonAction::GoToMetaShop,
                btn_cfg.get(),
            );
        });
}

/// Keeps the gold display current while the player is on the title screen.
///
/// Runs only in [`AppState::Title`] and early-returns when [`MetaProgress`]
/// has not changed (e.g. it was not modified since the last frame).
pub fn update_title_gold(
    meta: Res<MetaProgress>,
    mut label_q: Query<&mut Text, With<TitleGoldLabel>>,
) {
    if !meta.is_changed() {
        return;
    }
    let Ok(mut text) = label_q.single_mut() else {
        return;
    };
    *text = Text::new(format!("Gold: {}", meta.total_gold));
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
    use crate::hud::menu_button::LargeMenuButtonHud;

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<AppState>();
        app.insert_resource(MetaProgress::default());
        app
    }

    fn enter_title(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Playing);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Title);
        app.update();
        app.update();
    }

    #[test]
    fn setup_title_screen_spawns_nodes() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Title), setup_title_screen);
        enter_title(&mut app);

        let mut q = app.world_mut().query_filtered::<Entity, With<Node>>();
        assert!(
            q.iter(app.world()).count() > 0,
            "title screen must spawn at least one Node"
        );
    }

    #[test]
    fn setup_title_screen_has_two_buttons() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Title), setup_title_screen);
        enter_title(&mut app);

        let mut q = app.world_mut().query_filtered::<Entity, With<Button>>();
        assert_eq!(
            q.iter(app.world()).count(),
            2,
            "title screen should have exactly two buttons (Start Game + Gold Shop)"
        );
    }

    #[test]
    fn start_button_goes_to_character_select() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Title), setup_title_screen);
        enter_title(&mut app);

        let mut q = app.world_mut().query::<&MenuButton>();
        let actions: Vec<ButtonAction> = q.iter(app.world()).map(|b| b.action).collect();
        assert!(
            actions.contains(&ButtonAction::GoToCharacterSelect),
            "Start Game button must use GoToCharacterSelect"
        );
    }

    #[test]
    fn gold_shop_button_goes_to_meta_shop() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Title), setup_title_screen);
        enter_title(&mut app);

        let mut q = app.world_mut().query::<&MenuButton>();
        let actions: Vec<ButtonAction> = q.iter(app.world()).map(|b| b.action).collect();
        assert!(
            actions.contains(&ButtonAction::GoToMetaShop),
            "Gold Shop button must use GoToMetaShop"
        );
    }

    #[test]
    fn gold_label_shows_current_gold() {
        let mut app = build_app();
        app.world_mut().resource_mut::<MetaProgress>().total_gold = 42;
        app.add_systems(OnEnter(AppState::Title), setup_title_screen);
        enter_title(&mut app);

        let mut q = app
            .world_mut()
            .query_filtered::<&Text, With<TitleGoldLabel>>();
        let text = q.single(app.world()).expect("TitleGoldLabel must exist");
        assert!(
            text.0.contains("42"),
            "gold label should contain '42', got '{}'",
            text.0
        );
    }

    #[test]
    fn update_title_gold_reflects_meta_progress() {
        let mut app = build_app();
        let label = app
            .world_mut()
            .spawn((Text::new("Gold: 0"), TitleGoldLabel))
            .id();
        app.world_mut().resource_mut::<MetaProgress>().total_gold = 99;

        app.world_mut().run_system_once(update_title_gold).unwrap();

        let text = app.world().get::<Text>(label).unwrap();
        assert!(text.0.contains("99"), "expected '99' in '{}'", text.0);
    }

    #[test]
    fn title_screen_bg_has_marker_component() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Title), setup_title_screen);
        enter_title(&mut app);

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<TitleScreenBg>>();
        assert_eq!(q.iter(app.world()).count(), 1);
    }

    #[test]
    fn title_screen_has_two_button_hud_markers() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Title), setup_title_screen);
        enter_title(&mut app);

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<LargeMenuButtonHud>>();
        assert_eq!(
            q.iter(app.world()).count(),
            2,
            "both Start Game and Gold Shop should have LargeMenuButtonHud"
        );
    }

    #[test]
    fn title_screen_heading_has_hud_marker() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Title), setup_title_screen);
        enter_title(&mut app);

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<ScreenHeadingHud>>();
        assert_eq!(q.iter(app.world()).count(), 1);
    }
}
