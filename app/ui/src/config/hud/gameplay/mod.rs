//! Gameplay HUD configuration modules.
//!
//! | File                                              | Config type                       | Controls                              |
//! |---------------------------------------------------|-----------------------------------|---------------------------------------|
//! | `config/ui/hud/gameplay/hp_bar.ron`               | [`HpBarHudConfig`]                | HP bar dimensions, radius, colors     |
//! | `config/ui/hud/gameplay/xp_bar.ron`               | [`XpBarHudConfig`]                | XP bar height and colors              |
//! | `config/ui/hud/gameplay/timer.ron`                | [`TimerHudConfig`]                | Timer font size and color             |
//! | `config/ui/hud/gameplay/level.ron`                | [`LevelHudConfig`]                | Level label font size and color       |
//! | `config/ui/hud/gameplay/layout.ron`               | [`GameplayHudLayoutConfig`]       | Widget anchor positions               |
//! | `config/ui/hud/gameplay/evolution_notification.ron` | [`EvolutionNotificationHudConfig`] | Notification timing, font, color    |

pub mod evolution_notification;
pub mod hp_bar;
pub mod layout;
pub mod level;
pub mod timer;
pub mod xp_bar;

pub use evolution_notification::{
    EvolutionNotificationHudConfig, EvolutionNotificationHudConfigHandle,
    EvolutionNotificationHudParams,
};
pub use hp_bar::{HpBarHudConfig, HpBarHudConfigHandle, HpBarHudParams};
pub use layout::{GameplayHudLayoutConfig, GameplayHudLayoutConfigHandle, GameplayHudLayoutParams};
pub use level::{LevelHudConfig, LevelHudConfigHandle, LevelHudParams};
pub use timer::{TimerHudConfig, TimerHudConfigHandle, TimerHudParams};
pub use xp_bar::{XpBarHudConfig, XpBarHudConfigHandle, XpBarHudParams};
