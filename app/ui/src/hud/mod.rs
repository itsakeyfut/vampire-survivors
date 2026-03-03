//! HUD widget modules.
//!
//! ## Reusable screen widgets
//!
//! | Module             | Spawn function             | Marker(s)                                    |
//! |--------------------|----------------------------|----------------------------------------------|
//! | [`screen_heading`] | `spawn_screen_heading`     | [`screen_heading::ScreenHeadingHud`]         |
//! | [`menu_button`]    | `spawn_large_menu_button`  | [`menu_button::LargeMenuButtonHud`], [`menu_button::LargeMenuButtonLabelHud`] |
//! | [`upgrade_card`]   | `spawn_upgrade_card`       | [`upgrade_card::UpgradeCardHud`]             |
//!
//! ## Gameplay HUD
//!
//! [`gameplay`] groups in-game overlay widgets (HP bar, XP bar, timer, level).
//! See [`gameplay::setup_gameplay_hud`] for the entry point.

pub mod gameplay;
pub mod menu_button;
pub mod screen_heading;
pub mod upgrade_card;
