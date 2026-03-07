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
//! | `config/ui/hud/gameplay/boss_warning.ron`         | [`BossWarningHudConfig`]          | Boss warning timing, font, color      |
//! | `config/ui/hud/gameplay/boss_hp_bar.ron`          | [`BossHpBarHudConfig`]            | Boss HP bar dimensions and colors     |
//! | `config/ui/hud/gameplay/kill_count.ron`           | [`KillCountHudConfig`]            | Kill count label font and color       |
//! | `config/ui/hud/gameplay/weapon_slots.ron`         | [`WeaponSlotsHudConfig`]          | Weapon slot dimensions and colors     |

pub mod boss_hp_bar;
pub mod boss_warning;
pub mod evolution_notification;
pub mod hp_bar;
pub mod kill_count;
pub mod layout;
pub mod level;
pub mod timer;
pub mod weapon_slots;
pub mod xp_bar;

pub use boss_hp_bar::{BossHpBarHudConfig, BossHpBarHudConfigHandle, BossHpBarHudParams};
pub use boss_warning::{BossWarningHudConfig, BossWarningHudConfigHandle, BossWarningHudParams};
pub use evolution_notification::{
    EvolutionNotificationHudConfig, EvolutionNotificationHudConfigHandle,
    EvolutionNotificationHudParams,
};
pub use hp_bar::{HpBarHudConfig, HpBarHudConfigHandle, HpBarHudParams};
pub use kill_count::{KillCountHudConfig, KillCountHudConfigHandle, KillCountHudParams};
pub use layout::{GameplayHudLayoutConfig, GameplayHudLayoutConfigHandle, GameplayHudLayoutParams};
pub use level::{LevelHudConfig, LevelHudConfigHandle, LevelHudParams};
pub use timer::{TimerHudConfig, TimerHudConfigHandle, TimerHudParams};
pub use weapon_slots::{WeaponSlotsHudConfig, WeaponSlotsHudConfigHandle, WeaponSlotsHudParams};
pub use xp_bar::{XpBarHudConfig, XpBarHudConfigHandle, XpBarHudParams};
