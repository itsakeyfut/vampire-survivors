//! Internationalisation: in-code string table.
//!
//! All user-visible strings should pass through [`t`] so that switching
//! [`Language`] at runtime immediately changes the UI text on the next
//! screen rebuild (or the same frame for [`TranslatableText`] nodes).
//!
//! # Usage
//!
//! ```ignore
//! use crate::i18n::{t, TranslatableText};
//! use vs_core::resources::Language;
//!
//! // At spawn time:
//! Text::new(t("btn_start_game", lang))
//!
//! // For live updates (e.g. settings screen language toggle):
//! entity.insert(TranslatableText("btn_start_game"));
//! ```

use bevy::prelude::*;
use vs_core::resources::Language;

// ---------------------------------------------------------------------------
// Marker component
// ---------------------------------------------------------------------------

/// Tags a [`Text`] node whose content should be refreshed via [`t`] whenever
/// [`vs_core::resources::GameSettings::language`] changes.
///
/// The stored key must be a `&'static str` matching an entry in [`t`].
/// [`update_translatable_texts`] queries all entities with this component and
/// re-sets their text to the localised string for the current language.
#[derive(Component, Debug)]
pub struct TranslatableText(pub &'static str);

// ---------------------------------------------------------------------------
// Translation table
// ---------------------------------------------------------------------------

/// Returns the localised string for the given key and language.
///
/// Falls back to `key` itself when a translation is missing so new keys
/// remain visible during development rather than silently showing empty text.
/// All call sites must pass a string literal so the returned `&'static str`
/// reference is valid for the lifetime of the program.
pub fn t(key: &'static str, lang: Language) -> &'static str {
    match (key, lang) {
        // ── Title screen ──────────────────────────────────────────────────
        ("game_title", Language::Japanese) => "Vampire Survivors",
        ("game_title", Language::English) => "Vampire Survivors",
        ("btn_start_game", Language::Japanese) => "スタート",
        ("btn_start_game", Language::English) => "Start Game",
        ("btn_gold_shop", Language::Japanese) => "ゴールドショップ",
        ("btn_gold_shop", Language::English) => "Gold Shop",
        ("btn_settings", Language::Japanese) => "設定",
        ("btn_settings", Language::English) => "Settings",
        ("gold_display", Language::Japanese) => "ゴールド",
        ("gold_display", Language::English) => "Gold",

        // ── Settings screen ───────────────────────────────────────────────
        ("settings_title", Language::Japanese) => "設定",
        ("settings_title", Language::English) => "Settings",
        ("label_language", Language::Japanese) => "言語",
        ("label_language", Language::English) => "Language",
        ("lang_japanese", Language::Japanese) => "日本語",
        ("lang_japanese", Language::English) => "Japanese",
        ("lang_english", Language::Japanese) => "English",
        ("lang_english", Language::English) => "English",
        ("btn_back", Language::Japanese) => "もどる",
        ("btn_back", Language::English) => "Back",

        // ── Character select screen ───────────────────────────────────────
        ("character_select_title", Language::Japanese) => "キャラクター選択",
        ("character_select_title", Language::English) => "Choose Your Character",
        ("btn_play", Language::Japanese) => "プレイ",
        ("btn_play", Language::English) => "Play",

        // ── Meta shop screen ──────────────────────────────────────────────
        ("meta_shop_title", Language::Japanese) => "ゴールドショップ",
        ("meta_shop_title", Language::English) => "Gold Shop",

        // ── Pause screen ──────────────────────────────────────────────────
        ("pause_title", Language::Japanese) => "ポーズ",
        ("pause_title", Language::English) => "PAUSED",
        ("btn_resume", Language::Japanese) => "再開",
        ("btn_resume", Language::English) => "Resume",
        ("btn_to_title", Language::Japanese) => "タイトルへ",
        ("btn_to_title", Language::English) => "To Title",

        // ── Game over screen ──────────────────────────────────────────────
        ("game_over_title", Language::Japanese) => "ゲームオーバー",
        ("game_over_title", Language::English) => "GAME OVER",
        ("btn_retry", Language::Japanese) => "もう一度",
        ("btn_retry", Language::English) => "Retry",

        // ── Victory screen ────────────────────────────────────────────────
        ("victory_title", Language::Japanese) => "勝利！",
        ("victory_title", Language::English) => "VICTORY!",

        // ── Fallback ──────────────────────────────────────────────────────
        _ => key,
    }
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Updates all [`TranslatableText`] nodes whenever [`GameSettings`] changes.
///
/// Queries every text entity tagged with [`TranslatableText`] and re-sets its
/// text to the localised string for the current language.  Only performs work
/// on frames where [`GameSettings`] was actually modified.
pub fn update_translatable_texts(
    settings: Res<vs_core::resources::GameSettings>,
    mut query: Query<(&mut Text, &TranslatableText)>,
) {
    if !settings.is_changed() {
        return;
    }
    let lang = settings.language;
    for (mut text, key) in query.iter_mut() {
        *text = Text::new(t(key.0, lang));
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_key_japanese() {
        assert_eq!(t("btn_start_game", Language::Japanese), "スタート");
    }

    #[test]
    fn known_key_english() {
        assert_eq!(t("btn_start_game", Language::English), "Start Game");
    }

    #[test]
    fn unknown_key_returns_key_itself() {
        assert_eq!(t("nonexistent_key", Language::Japanese), "nonexistent_key");
        assert_eq!(t("nonexistent_key", Language::English), "nonexistent_key");
    }

    #[test]
    fn all_keys_non_empty() {
        let keys = [
            "game_title",
            "btn_start_game",
            "btn_gold_shop",
            "btn_settings",
            "settings_title",
            "label_language",
            "lang_japanese",
            "lang_english",
            "btn_back",
            "character_select_title",
            "btn_play",
            "meta_shop_title",
            "pause_title",
            "btn_resume",
            "game_over_title",
            "victory_title",
        ];
        for key in &keys {
            assert!(!t(key, Language::Japanese).is_empty(), "JP: {key}");
            assert!(!t(key, Language::English).is_empty(), "EN: {key}");
        }
    }

    #[test]
    fn languages_differ_for_distinguishable_keys() {
        assert_ne!(
            t("btn_start_game", Language::Japanese),
            t("btn_start_game", Language::English)
        );
        assert_ne!(
            t("btn_settings", Language::Japanese),
            t("btn_settings", Language::English)
        );
    }
}
