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

use crate::styles::{FONT_TEXT_EN, FONT_TEXT_JP};

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
        ("btn_start_with_char", Language::Japanese) => "このキャラで開始",
        ("btn_start_with_char", Language::English) => "Start",
        ("label_locked", Language::Japanese) => "🔒 ゴールドショップで解放できます",
        ("label_locked", Language::English) => "🔒 Unlock in the Gold Shop",
        ("label_unlock_cost", Language::Japanese) => "解放には {cost} G 必要",
        ("label_unlock_cost", Language::English) => "Requires {cost} G to unlock",
        ("label_hp", Language::Japanese) => "HP",
        ("label_hp", Language::English) => "HP",
        ("label_speed", Language::Japanese) => "速度",
        ("label_speed", Language::English) => "Speed",
        ("label_weapon", Language::Japanese) => "初期武器",
        ("label_weapon", Language::English) => "Starting Weapon",
        // ── Weapon names (for character-select detail panel) ─────────────
        ("weapon_whip", Language::Japanese) => "ムチ",
        ("weapon_whip", Language::English) => "Whip",
        ("weapon_magic_wand", Language::Japanese) => "マジックワンド",
        ("weapon_magic_wand", Language::English) => "Magic Wand",
        ("weapon_knife", Language::Japanese) => "ナイフ",
        ("weapon_knife", Language::English) => "Knife",
        ("weapon_garlic", Language::Japanese) => "ガーリック",
        ("weapon_garlic", Language::English) => "Garlic",
        ("weapon_bible", Language::Japanese) => "聖書",
        ("weapon_bible", Language::English) => "Bible",
        ("weapon_thunder_ring", Language::Japanese) => "サンダーリング",
        ("weapon_thunder_ring", Language::English) => "Thunder Ring",
        ("weapon_cross", Language::Japanese) => "クロス",
        ("weapon_cross", Language::English) => "Cross",
        ("weapon_fire_wand", Language::Japanese) => "ファイアワンド",
        ("weapon_fire_wand", Language::English) => "Fire Wand",

        // ── Level-up screen ───────────────────────────────────────────────
        ("level_up_title", Language::Japanese) => "レベルアップ！",
        ("level_up_title", Language::English) => "LEVEL UP!",

        // ── Meta shop screen ──────────────────────────────────────────────
        ("meta_shop_title", Language::Japanese) => "ゴールドショップ",
        ("meta_shop_title", Language::English) => "Gold Shop",
        ("shop_section_characters", Language::Japanese) => "キャラクター解放",
        ("shop_section_characters", Language::English) => "Unlock Characters",
        ("shop_section_upgrades", Language::Japanese) => "パーマネントアップグレード",
        ("shop_section_upgrades", Language::English) => "Permanent Upgrades",
        ("shop_char_magician", Language::Japanese) => "マジシャン",
        ("shop_char_magician", Language::English) => "Magician",
        ("shop_char_thief", Language::Japanese) => "シーフ",
        ("shop_char_thief", Language::English) => "Thief",
        ("shop_char_knight", Language::Japanese) => "ナイト",
        ("shop_char_knight", Language::English) => "Knight",
        ("shop_upgrade_hp", Language::Japanese) => "+最大HP",
        ("shop_upgrade_hp", Language::English) => "+Max HP",
        ("shop_upgrade_speed", Language::Japanese) => "+移動速度",
        ("shop_upgrade_speed", Language::English) => "+Speed",
        ("shop_upgrade_damage", Language::Japanese) => "+攻撃力",
        ("shop_upgrade_damage", Language::English) => "+Damage",
        ("shop_upgrade_xp", Language::Japanese) => "+XP取得",
        ("shop_upgrade_xp", Language::English) => "+XP Gain",
        ("shop_upgrade_weapon", Language::Japanese) => "+開始武器",
        ("shop_upgrade_weapon", Language::English) => "+Starting Weapon",

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
        ("stat_clear_time", Language::Japanese) => "クリアタイム:",
        ("stat_clear_time", Language::English) => "Clear Time:",
        ("stat_level_reached", Language::Japanese) => "到達レベル:",
        ("stat_level_reached", Language::English) => "Level Reached:",
        ("stat_enemies_defeated", Language::Japanese) => "撃破数:",
        ("stat_enemies_defeated", Language::English) => "Enemies Defeated:",
        ("stat_gold_earned", Language::Japanese) => "獲得ゴールド:",
        ("stat_gold_earned", Language::English) => "Gold Earned:",

        // ── Fallback ──────────────────────────────────────────────────────
        _ => key,
    }
}

// ---------------------------------------------------------------------------
// Font selection
// ---------------------------------------------------------------------------

/// Returns the font asset path for the given language.
///
/// Each language group maps to a dedicated font constant so that swapping a
/// font for a specific script (e.g. adding Cyrillic support for Russian) only
/// requires updating this function and adding the new asset path in `styles.rs`.
///
/// Pass the result to [`AssetServer::load`] to obtain a [`Handle<Font>`].
pub fn font_for_lang(lang: Language) -> &'static str {
    match lang {
        Language::Japanese => FONT_TEXT_JP,
        Language::English => FONT_TEXT_EN,
    }
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Updates all [`TranslatableText`] nodes whenever [`GameSettings`] changes.
///
/// Queries every text entity tagged with [`TranslatableText`] and re-sets both
/// its content via [`t`] and its [`TextFont`] handle to the language-appropriate
/// font.  Only performs work on frames where [`GameSettings`] was modified.
pub fn update_translatable_texts(
    settings: Res<vs_core::resources::GameSettings>,
    asset_server: Option<Res<AssetServer>>,
    mut query: Query<(&mut Text, &mut TextFont, &TranslatableText)>,
) {
    if !settings.is_changed() {
        return;
    }
    let lang = settings.language;
    let new_font: Handle<Font> = asset_server
        .map(|s| s.load(font_for_lang(lang)))
        .unwrap_or_default();
    for (mut text, mut text_font, key) in query.iter_mut() {
        *text = Text::new(t(key.0, lang));
        text_font.font = new_font.clone();
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
            "btn_start_with_char",
            "label_locked",
            "label_unlock_cost",
            "label_hp",
            "label_speed",
            "label_weapon",
            "weapon_whip",
            "weapon_magic_wand",
            "weapon_knife",
            "weapon_garlic",
            "weapon_bible",
            "weapon_thunder_ring",
            "weapon_cross",
            "weapon_fire_wand",
            "level_up_title",
            "meta_shop_title",
            "pause_title",
            "btn_resume",
            "game_over_title",
            "victory_title",
            "stat_clear_time",
            "stat_level_reached",
            "stat_enemies_defeated",
            "stat_gold_earned",
        ];
        for key in &keys {
            assert_ne!(t(key, Language::Japanese), *key, "JP: {key}");
            assert_ne!(t(key, Language::English), *key, "EN: {key}");
        }
    }

    #[test]
    fn font_for_lang_covers_all_variants() {
        for lang in [Language::Japanese, Language::English] {
            assert!(
                !font_for_lang(lang).is_empty(),
                "font_for_lang({lang:?}) must not be empty"
            );
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
