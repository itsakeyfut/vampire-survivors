//! Non-tunable UI assets (font paths) shared across all screens.
//!
//! Screen-level color fallbacks (`DEFAULT_BG_COLOR`, `DEFAULT_TITLE_COLOR`)
//! live as private `const`s inside each consumer module, following the
//! project-wide convention of keeping `DEFAULT_*` constants local to their
//! implementation file rather than in a shared constants module.

// ---------------------------------------------------------------------------
// Font paths
// ---------------------------------------------------------------------------

/// Path to the Japanese text font (DotGothic16).
///
/// Used for Japanese UI text.  Currently shares the same file as
/// [`FONT_TEXT_EN`]; swap this path when a dedicated JP font is added.
/// Relative to the `assets/` directory; pass to [`AssetServer::load`].
pub const FONT_TEXT_JP: &str = "fonts/DotGothic16/DotGothic16-Regular.ttf";

/// Path to the Latin-script text font (DotGothic16).
///
/// Used for English, German, and other Latin-alphabet languages.
/// Currently shares the same file as [`FONT_TEXT_JP`]; swap this path when
/// a dedicated Latin font is added (e.g. for Cyrillic, add `FONT_TEXT_RU`).
/// Relative to the `assets/` directory; pass to [`AssetServer::load`].
pub const FONT_TEXT_EN: &str = "fonts/DotGothic16/DotGothic16-Regular.ttf";

/// Path to the symbol / UI-icon font (Noto Sans JP).
///
/// Used for characters outside DotGothic16's coverage, such as punctuation
/// symbols not present in the pixel font.
/// Relative to the `assets/` directory; pass to [`AssetServer::load`].
pub const FONT_SYMBOL: &str = "fonts/NotoSansJP/NotoSansJP-Regular.ttf";
