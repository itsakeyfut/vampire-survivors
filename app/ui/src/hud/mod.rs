//! Reusable HUD widget modules.
//!
//! Each module provides a spawn function, marker components, and a hot-reload
//! system for a single self-contained UI widget.
//!
//! | Module           | Spawn function             | Marker(s)                                    |
//! |------------------|----------------------------|----------------------------------------------|
//! | [`screen_heading`] | `spawn_screen_heading`   | [`screen_heading::ScreenHeadingHud`]         |
//! | [`menu_button`]    | `spawn_large_menu_button`| [`menu_button::LargeMenuButtonHud`], [`menu_button::LargeMenuButtonLabelHud`] |
//! | [`upgrade_card`]   | `spawn_upgrade_card`     | [`upgrade_card::UpgradeCardHud`]             |

pub mod menu_button;
pub mod screen_heading;
pub mod upgrade_card;
