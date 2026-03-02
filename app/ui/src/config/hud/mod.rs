//! HUD-specific configuration modules.
//!
//! Each HUD widget has its own config file loaded from
//! `assets/config/ui/hud/<name>.ron`.
//!
//! | File                               | Config type                | Controls                           |
//! |------------------------------------|----------------------------|------------------------------------|
//! | `config/ui/hud/screen_heading.ron` | [`ScreenHeadingHudConfig`] | Heading font size and bottom margin |
//! | `config/ui/hud/menu_button.ron`    | [`MenuButtonHudConfig`]    | Button dimensions, colors, font    |
//! | `config/ui/hud/upgrade_card.ron`   | [`UpgradeCardHudConfig`]   | Card layout, colors, typography    |

pub mod menu_button;
pub mod screen_heading;
pub mod upgrade_card;

pub use menu_button::{MenuButtonHudConfig, MenuButtonHudConfigHandle, MenuButtonHudParams};
pub use screen_heading::{
    ScreenHeadingHudConfig, ScreenHeadingHudConfigHandle, ScreenHeadingHudParams,
};
pub use upgrade_card::{UpgradeCardHudConfig, UpgradeCardHudConfigHandle, UpgradeCardHudParams};
