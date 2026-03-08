//! Meta shop screen — minimal stub.
//!
//! Displays a heading and a "Back" button that returns to the Title screen.
//! A full gold shop UI (with purchasable upgrades and character unlocks) will be
//! implemented in a later phase; this stub ensures `AppState::MetaShop` has an
//! entry handler so the Gold Shop button on the title screen does not lead to
//! an uninitialized state.
//!
//! All entities are tagged with [`DespawnOnExit`]`(AppState::MetaShop)` so
//! they are cleaned up automatically on any state transition.

use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;
use vs_core::states::AppState;

use vs_core::resources::GameSettings;

use crate::components::ButtonAction;
use crate::config::{MenuButtonHudParams, ScreenHeadingHudParams, UiStyleParams};
use crate::hud::menu_button::spawn_large_menu_button;
use crate::hud::screen_heading::ScreenHeadingHud;
use crate::i18n::{font_for_lang, t};
use crate::styles::{DEFAULT_BG_COLOR, DEFAULT_TITLE_COLOR};

// ---------------------------------------------------------------------------
// Fallback constants
// ---------------------------------------------------------------------------

const DEFAULT_FONT_SIZE: f32 = 48.0;
const DEFAULT_MARGIN_BOTTOM: f32 = 60.0;
const DEFAULT_ROW_GAP: f32 = 16.0;

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Spawns the meta shop screen when entering [`AppState::MetaShop`].
pub fn setup_meta_shop_screen(
    mut commands: Commands,
    ui_style: UiStyleParams,
    heading_cfg: ScreenHeadingHudParams,
    btn_cfg: MenuButtonHudParams,
    asset_server: Option<Res<AssetServer>>,
    settings: Option<Res<GameSettings>>,
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
                row_gap: Val::Px(DEFAULT_ROW_GAP),
                ..default()
            },
            BackgroundColor(bg_color),
            DespawnOnExit(AppState::MetaShop),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(t("meta_shop_title", lang)),
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

            spawn_large_menu_button(
                parent,
                t("btn_back", lang),
                ButtonAction::GoToTitle,
                btn_cfg.get(),
                font.clone(),
                None,
            );
        });
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
}
