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
//! | `config/ui/hud/gameplay/gold.ron`                 | [`GoldHudConfig`]                 | Gold label font and color             |
//! | `config/ui/hud/gameplay/weapon_slots.ron`         | [`WeaponSlotsHudConfig`]          | Weapon slot dimensions and colors     |

pub mod boss_hp_bar;
pub mod boss_warning;
pub mod evolution_notification;
pub mod gold;
pub mod hp_bar;
pub mod kill_count;
pub mod layout;
pub mod level;
pub mod timer;
pub mod weapon_slots;
pub mod xp_bar;

pub(crate) use boss_hp_bar::BossHpBarHudConfigPartial;
pub use boss_hp_bar::{BossHpBarHudConfig, BossHpBarHudConfigHandle, BossHpBarHudParams};
pub(crate) use boss_warning::BossWarningHudConfigPartial;
pub use boss_warning::{BossWarningHudConfig, BossWarningHudConfigHandle, BossWarningHudParams};
pub(crate) use evolution_notification::EvolutionNotificationHudConfigPartial;
pub use evolution_notification::{
    EvolutionNotificationHudConfig, EvolutionNotificationHudConfigHandle,
    EvolutionNotificationHudParams,
};
pub(crate) use gold::GoldHudConfigPartial;
pub use gold::{GoldHudConfig, GoldHudConfigHandle, GoldHudParams};
pub(crate) use hp_bar::HpBarHudConfigPartial;
pub use hp_bar::{HpBarHudConfig, HpBarHudConfigHandle, HpBarHudParams};
pub(crate) use kill_count::KillCountHudConfigPartial;
pub use kill_count::{KillCountHudConfig, KillCountHudConfigHandle, KillCountHudParams};
pub(crate) use layout::GameplayHudLayoutConfigPartial;
pub use layout::{GameplayHudLayoutConfig, GameplayHudLayoutConfigHandle, GameplayHudLayoutParams};
pub(crate) use level::LevelHudConfigPartial;
pub use level::{LevelHudConfig, LevelHudConfigHandle, LevelHudParams};
pub(crate) use timer::TimerHudConfigPartial;
pub use timer::{TimerHudConfig, TimerHudConfigHandle, TimerHudParams};
pub(crate) use weapon_slots::WeaponSlotsHudConfigPartial;
pub use weapon_slots::{WeaponSlotsHudConfig, WeaponSlotsHudConfigHandle, WeaponSlotsHudParams};
pub(crate) use xp_bar::XpBarHudConfigPartial;
pub use xp_bar::{XpBarHudConfig, XpBarHudConfigHandle, XpBarHudParams};
