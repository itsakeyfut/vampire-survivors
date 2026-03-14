//! Settings screen.
//!
//! Displays configurable options; currently only language selection.
//! Returns to the Title screen via the "Back" button.
//!
//! All entities are tagged with [`DespawnOnExit`]`(AppState::Settings)` so
//! Bevy cleans them up automatically on state exit.
//!
//! # Layout
//!
//! ```text
//!         Settings / 設定
//!
//!  Language / 言語:  [ Japanese / 日本語 ↔ English ]
//!
//!         [ Back / もどる ]
//! ```
//!
//! The language button cycles between Japanese and English via
//! [`ButtonAction::ToggleLanguage`].  [`update_settings_display`] refreshes
//! the button label whenever [`GameSettings`] changes, and
//! [`crate::i18n::update_translatable_texts`] refreshes all other labeled
//! nodes at the same time.

use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;
use vs_core::resources::GameSettings;
use vs_core::states::AppState;

use crate::components::ButtonAction;
use crate::config::{MenuButtonHudParams, ScreenHeadingHudParams, UiStyleParams};
use crate::hud::menu_button::spawn_large_menu_button;
use crate::hud::screen_heading::ScreenHeadingHud;
use crate::i18n::{TranslatableText, font_for_lang, t};

// ---------------------------------------------------------------------------
// Fallback constants
// ---------------------------------------------------------------------------

const DEFAULT_ROW_GAP: f32 = 24.0;
const DEFAULT_LABEL_FONT_SIZE: f32 = 20.0;
const DEFAULT_LABEL_COLOR: Color = Color::srgb(0.95, 0.90, 0.85);
const DEFAULT_ROW_COLUMN_GAP: f32 = 16.0;

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

/// Marks the language toggle button label.
///
/// [`update_settings_display`] queries this to refresh the language name
/// text whenever [`GameSettings`] changes.
#[derive(Component, Debug)]
pub struct LanguageButtonLabel;

/// Marks the "Language:" row label.
///
/// [`update_settings_display`] also refreshes this text so the colon
/// is preserved (e.g. "Language:" / "言語:") after a language toggle.
#[derive(Component, Debug)]
pub struct LanguageLabelText;

// ---------------------------------------------------------------------------
// System: spawn
// ---------------------------------------------------------------------------

/// Spawns the settings screen UI when entering [`AppState::Settings`].
pub fn setup_settings_screen(
    mut commands: Commands,
    settings: Res<GameSettings>,
    ui_style: UiStyleParams,
    heading_cfg: ScreenHeadingHudParams,
    btn_cfg: MenuButtonHudParams,
    asset_server: Option<Res<AssetServer>>,
) {
    let lang = settings.language;
    let font: Handle<Font> = asset_server
        .map(|s| s.load(font_for_lang(lang)))
        .unwrap_or_default();

    let bg_color = ui_style.bg_color();
    let title_color = ui_style.title_color();
    let heading_font_size = heading_cfg.font_size();
    let heading_margin = heading_cfg.margin_bottom();

    let btn_width = btn_cfg.width();
    let btn_height = btn_cfg.height();
    let btn_font_size = btn_cfg.font_size();
    let btn_normal = btn_cfg.color_normal();
    let btn_text_color = btn_cfg.text_color();

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
            DespawnOnExit(AppState::Settings),
        ))
        .with_children(|parent| {
            // Heading — tagged for live language updates.
            parent.spawn((
                Text::new(t("settings_title", lang)),
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
                TranslatableText("settings_title"),
            ));

            // Language row: label + toggle button.
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(DEFAULT_ROW_COLUMN_GAP),
                    ..default()
                })
                .with_children(|row| {
                    // "Language:" label.
                    row.spawn((
                        Text::new(format!("{}:", t("label_language", lang))),
                        TextFont {
                            font: font.clone(),
                            font_size: DEFAULT_LABEL_FONT_SIZE,
                            ..default()
                        },
                        TextColor(DEFAULT_LABEL_COLOR),
                        LanguageLabelText,
                    ));

                    // Language toggle button — pressing it cycles JP ↔ EN.
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(btn_width),
                            height: Val::Px(btn_height * 0.75),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(btn_normal),
                        crate::components::MenuButton {
                            action: ButtonAction::ToggleLanguage,
                        },
                        crate::hud::menu_button::LargeMenuButtonHud,
                    ))
                    .with_children(|btn| {
                        let lang_key = match lang {
                            vs_core::resources::Language::Japanese => "lang_japanese",
                            vs_core::resources::Language::English => "lang_english",
                        };
                        btn.spawn((
                            Text::new(t(lang_key, lang)),
                            TextFont {
                                font: font.clone(),
                                font_size: btn_font_size * 0.6,
                                ..default()
                            },
                            TextColor(btn_text_color),
                            TextLayout::new_with_linebreak(LineBreak::NoWrap),
                            crate::hud::menu_button::LargeMenuButtonLabelHud,
                            LanguageButtonLabel,
                        ));
                    });
                });

            // Back button — TranslatableText enables live language updates.
            spawn_large_menu_button(
                parent,
                t("btn_back", lang),
                ButtonAction::GoToTitle,
                btn_cfg.get(),
                font.clone(),
                Some("btn_back"),
            );
        });
}

// ---------------------------------------------------------------------------
// System: update display
// ---------------------------------------------------------------------------

/// Updates the language button label whenever [`GameSettings`] changes.
///
/// Runs only in [`AppState::Settings`]; early-returns when `GameSettings`
/// was not modified this frame.  Updates both the text string and the
/// [`TextFont`] handle so the correct language-appropriate font is applied.
type RowLabelQuery<'w, 's> = Query<
    'w,
    's,
    (&'static mut Text, &'static mut TextFont),
    (With<LanguageLabelText>, Without<LanguageButtonLabel>),
>;

pub fn update_settings_display(
    settings: Res<GameSettings>,
    asset_server: Option<Res<AssetServer>>,
    mut button_label_q: Query<(&mut Text, &mut TextFont), With<LanguageButtonLabel>>,
    mut row_label_q: RowLabelQuery,
) {
    if !settings.is_changed() {
        return;
    }
    let lang = settings.language;
    let lang_key = match lang {
        vs_core::resources::Language::Japanese => "lang_japanese",
        vs_core::resources::Language::English => "lang_english",
    };
    let new_font: Handle<Font> = asset_server
        .map(|s| s.load(font_for_lang(lang)))
        .unwrap_or_default();
    for (mut text, mut text_font) in button_label_q.iter_mut() {
        *text = Text::new(t(lang_key, lang));
        text_font.font = new_font.clone();
    }
    for (mut text, mut text_font) in row_label_q.iter_mut() {
        *text = Text::new(format!("{}:", t("label_language", lang)));
        text_font.font = new_font.clone();
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
        app.insert_resource(GameSettings::default());
        app
    }

    fn enter_settings(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Settings);
        app.update();
    }

    #[test]
    fn setup_spawns_heading() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Settings), setup_settings_screen);
        enter_settings(&mut app);

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<ScreenHeadingHud>>();
        assert_eq!(q.iter(app.world()).count(), 1);
    }

    #[test]
    fn has_toggle_language_button() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Settings), setup_settings_screen);
        enter_settings(&mut app);

        let mut q = app.world_mut().query::<&MenuButton>();
        let actions: Vec<ButtonAction> = q.iter(app.world()).map(|b| b.action).collect();
        assert!(
            actions.contains(&ButtonAction::ToggleLanguage),
            "settings must have a ToggleLanguage button"
        );
    }

    #[test]
    fn has_back_button() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Settings), setup_settings_screen);
        enter_settings(&mut app);

        let mut q = app.world_mut().query::<&MenuButton>();
        let actions: Vec<ButtonAction> = q.iter(app.world()).map(|b| b.action).collect();
        assert!(
            actions.contains(&ButtonAction::GoToTitle),
            "settings must have a GoToTitle (Back) button"
        );
    }

    #[test]
    fn language_label_shows_japanese_by_default() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::Settings), setup_settings_screen);
        enter_settings(&mut app);

        let mut q = app
            .world_mut()
            .query_filtered::<&Text, With<LanguageButtonLabel>>();
        let text = q
            .single(app.world())
            .expect("LanguageButtonLabel must exist");
        assert_eq!(text.0, "日本語", "default language label should be 日本語");
    }

    #[test]
    fn update_settings_display_refreshes_label_on_language_change() {
        let mut app = build_app();
        let label = app
            .world_mut()
            .spawn((
                Text::new("日本語"),
                TextFont::default(),
                LanguageButtonLabel,
            ))
            .id();

        // Switch to English.
        app.world_mut().resource_mut::<GameSettings>().language =
            vs_core::resources::Language::English;

        app.world_mut()
            .run_system_once(update_settings_display)
            .unwrap();

        let text = app.world().get::<Text>(label).unwrap();
        assert_eq!(text.0, "English");
    }

    #[test]
    fn update_settings_display_preserves_colon_on_row_label() {
        let mut app = build_app();
        let row_label = app
            .world_mut()
            .spawn((Text::new("言語:"), TextFont::default(), LanguageLabelText))
            .id();

        // Switch to English.
        app.world_mut().resource_mut::<GameSettings>().language =
            vs_core::resources::Language::English;

        app.world_mut()
            .run_system_once(update_settings_display)
            .unwrap();

        let text = app.world().get::<Text>(row_label).unwrap();
        assert_eq!(text.0, "Language:", "colon must be preserved after toggle");
    }
}
