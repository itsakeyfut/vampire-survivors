//! HUD-specific configuration modules.
//!
//! Each HUD widget has its own config file loaded from
//! `assets/config/ui/hud/<name>.ron`.
//!
//! | File                                              | Config type                  | Controls                           |
//! |---------------------------------------------------|------------------------------|------------------------------------|
//! | `config/ui/hud/screen_heading.ron`                | [`ScreenHeadingHudConfig`]   | Heading font size and bottom margin |
//! | `config/ui/hud/menu_button.ron`                   | [`MenuButtonHudConfig`]      | Button dimensions, colors, font    |
//! | `config/ui/hud/upgrade_card.ron`                  | [`UpgradeCardHudConfig`]     | Card layout, colors, typography    |
//! | `config/ui/hud/gameplay/hp_bar.ron`               | [`HpBarHudConfig`]           | HP bar dimensions, radius, colors  |
//! | `config/ui/hud/gameplay/xp_bar.ron`               | [`XpBarHudConfig`]           | XP bar height and colors           |
//! | `config/ui/hud/gameplay/timer.ron`                | [`TimerHudConfig`]           | Timer font size and color          |
//! | `config/ui/hud/gameplay/level.ron`                | [`LevelHudConfig`]           | Level label font size and color    |
//! | `config/ui/hud/gameplay/layout.ron`               | [`GameplayHudLayoutConfig`]  | Widget anchor positions            |

pub mod gameplay;
pub mod menu_button;
pub mod screen_heading;
pub mod upgrade_card;

pub use gameplay::{
    EvolutionNotificationHudConfig, EvolutionNotificationHudConfigHandle,
    EvolutionNotificationHudParams, GameplayHudLayoutConfig, GameplayHudLayoutConfigHandle,
    GameplayHudLayoutParams, HpBarHudConfig, HpBarHudConfigHandle, HpBarHudParams, LevelHudConfig,
    LevelHudConfigHandle, LevelHudParams, TimerHudConfig, TimerHudConfigHandle, TimerHudParams,
    XpBarHudConfig, XpBarHudConfigHandle, XpBarHudParams,
};
pub use menu_button::{MenuButtonHudConfig, MenuButtonHudConfigHandle, MenuButtonHudParams};
pub use screen_heading::{
    ScreenHeadingHudConfig, ScreenHeadingHudConfigHandle, ScreenHeadingHudParams,
};
pub use upgrade_card::{UpgradeCardHudConfig, UpgradeCardHudConfigHandle, UpgradeCardHudParams};
