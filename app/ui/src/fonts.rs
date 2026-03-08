//! Font preloading — warms the asset cache at startup so screens load fonts
//! from cache rather than disk, preventing a first-frame blink.
//!
//! Each screen calls `asset_server.load(font_for_lang(lang))` to obtain the
//! correct font handle.  This module ensures all font assets are already in
//! flight before any `OnEnter` handler fires.
//!
//! When a new language script is added (e.g. Cyrillic for Russian), add its
//! font constant to `styles.rs`, wire it in [`crate::i18n::font_for_lang`],
//! and add a corresponding field here so the asset is preloaded.

use bevy::prelude::*;

use crate::styles::{FONT_SYMBOL, FONT_TEXT_EN, FONT_TEXT_JP};

// ---------------------------------------------------------------------------
// Private resource — keeps strong handles alive so the cache is never evicted.
// ---------------------------------------------------------------------------

#[derive(Resource)]
struct FontCache {
    _text_jp: Handle<Font>,
    _text_en: Handle<Font>,
    _symbol: Handle<Font>,
}

// ---------------------------------------------------------------------------
// Startup system
// ---------------------------------------------------------------------------

/// Preloads all game fonts at app startup.
///
/// Inserts [`FontCache`] which holds strong handles to prevent eviction.
/// After this runs, every `asset_server.load(font_for_lang(lang))` call in
/// screen setup systems returns an already-loading (or loaded) cached handle.
pub fn preload_fonts(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(FontCache {
        _text_jp: asset_server.load(FONT_TEXT_JP),
        _text_en: asset_server.load(FONT_TEXT_EN),
        _symbol: asset_server.load(FONT_SYMBOL),
    });
}
