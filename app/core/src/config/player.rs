//! Player configuration loaded from `assets/config/player.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

use crate::{resources::MetaProgress, types::MetaUpgradeType};

// ---------------------------------------------------------------------------
// Fallback constants (used while player.ron is still loading)
// ---------------------------------------------------------------------------

const DEFAULT_BASE_HP: f32 = 100.0;
const DEFAULT_BASE_SPEED: f32 = 200.0;
const DEFAULT_BASE_DAMAGE_MULT: f32 = 1.0;
const DEFAULT_BASE_COOLDOWN_REDUCTION: f32 = 0.0;
const DEFAULT_BASE_PROJECTILE_SPEED: f32 = 1.0;
const DEFAULT_BASE_DURATION_MULT: f32 = 1.0;
const DEFAULT_BASE_AREA_MULT: f32 = 1.0;
const DEFAULT_BASE_LUCK: f32 = 1.0;
const DEFAULT_BASE_HP_REGEN: f32 = 0.0;
const DEFAULT_BASE_XP_MULT: f32 = 1.0;
const DEFAULT_PICKUP_RADIUS: f32 = 80.0;
const DEFAULT_INVINCIBILITY_TIME: f32 = 0.5;
const DEFAULT_COLLIDER_RADIUS: f32 = 12.0;
const DEFAULT_COLLIDER_PROJECTILE_SMALL: f32 = 5.0;
const DEFAULT_COLLIDER_PROJECTILE_LARGE: f32 = 10.0;
const DEFAULT_COLLIDER_XP_GEM: f32 = 6.0;
const DEFAULT_COLLIDER_GOLD_COIN: f32 = 6.0;
const DEFAULT_COLLIDER_TREASURE: f32 = 20.0;
const DEFAULT_GEM_ATTRACTION_SPEED: f32 = 200.0;
const DEFAULT_GEM_ABSORPTION_RADIUS: f32 = 8.0;

// ---------------------------------------------------------------------------
// Asset type
// ---------------------------------------------------------------------------

/// Deserialization mirror of [`PlayerConfig`] — every field is `Option<T>` so
/// RON files with missing fields still load and emit a `warn!` instead of failing.
#[derive(Deserialize, Default)]
#[serde(default, rename = "PlayerConfig")]
pub(super) struct PlayerConfigPartial {
    pub base_hp: Option<f32>,
    pub base_speed: Option<f32>,
    pub base_damage_mult: Option<f32>,
    pub base_cooldown_reduction: Option<f32>,
    pub base_projectile_speed: Option<f32>,
    pub base_duration_mult: Option<f32>,
    pub base_area_mult: Option<f32>,
    pub base_luck: Option<f32>,
    pub base_hp_regen: Option<f32>,
    pub base_xp_mult: Option<f32>,
    pub pickup_radius: Option<f32>,
    pub invincibility_time: Option<f32>,
    pub collider_radius: Option<f32>,
    pub collider_projectile_small: Option<f32>,
    pub collider_projectile_large: Option<f32>,
    pub collider_xp_gem: Option<f32>,
    pub collider_gold_coin: Option<f32>,
    pub collider_treasure: Option<f32>,
    pub gem_attraction_speed: Option<f32>,
    pub gem_absorption_radius: Option<f32>,
}

/// Player base stats and collider radii, loaded from `assets/config/player.ron`.
///
/// Hot-reloading this file during gameplay immediately updates the live
/// `PlayerStats` component on the player entity.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct PlayerConfig {
    // Base player stats
    pub base_hp: f32,
    pub base_speed: f32,
    pub base_damage_mult: f32,
    pub base_cooldown_reduction: f32,
    pub base_projectile_speed: f32,
    pub base_duration_mult: f32,
    pub base_area_mult: f32,
    pub base_luck: f32,
    pub base_hp_regen: f32,
    /// Base XP gain multiplier (1.0 = no bonus; boosted by BonusXp meta upgrade).
    pub base_xp_mult: f32,
    pub pickup_radius: f32,
    pub invincibility_time: f32,
    // Collider radii (pixels)
    pub collider_radius: f32,
    pub collider_projectile_small: f32,
    pub collider_projectile_large: f32,
    pub collider_xp_gem: f32,
    pub collider_gold_coin: f32,
    pub collider_treasure: f32,
    // XP gem attraction
    pub gem_attraction_speed: f32,
    pub gem_absorption_radius: f32,
}

impl From<PlayerConfigPartial> for PlayerConfig {
    fn from(p: PlayerConfigPartial) -> Self {
        PlayerConfig {
            base_hp: p.base_hp.unwrap_or_else(|| {
                warn!("player.ron: `base_hp` missing → using default {DEFAULT_BASE_HP}");
                DEFAULT_BASE_HP
            }),
            base_speed: p.base_speed.unwrap_or_else(|| {
                warn!("player.ron: `base_speed` missing → using default {DEFAULT_BASE_SPEED}");
                DEFAULT_BASE_SPEED
            }),
            base_damage_mult: p.base_damage_mult.unwrap_or_else(|| {
                warn!(
                    "player.ron: `base_damage_mult` missing → using default {DEFAULT_BASE_DAMAGE_MULT}"
                );
                DEFAULT_BASE_DAMAGE_MULT
            }),
            base_cooldown_reduction: p.base_cooldown_reduction.unwrap_or_else(|| {
                warn!(
                    "player.ron: `base_cooldown_reduction` missing → using default {DEFAULT_BASE_COOLDOWN_REDUCTION}"
                );
                DEFAULT_BASE_COOLDOWN_REDUCTION
            }),
            base_projectile_speed: p.base_projectile_speed.unwrap_or_else(|| {
                warn!(
                    "player.ron: `base_projectile_speed` missing → using default {DEFAULT_BASE_PROJECTILE_SPEED}"
                );
                DEFAULT_BASE_PROJECTILE_SPEED
            }),
            base_duration_mult: p.base_duration_mult.unwrap_or_else(|| {
                warn!(
                    "player.ron: `base_duration_mult` missing → using default {DEFAULT_BASE_DURATION_MULT}"
                );
                DEFAULT_BASE_DURATION_MULT
            }),
            base_area_mult: p.base_area_mult.unwrap_or_else(|| {
                warn!(
                    "player.ron: `base_area_mult` missing → using default {DEFAULT_BASE_AREA_MULT}"
                );
                DEFAULT_BASE_AREA_MULT
            }),
            base_luck: p.base_luck.unwrap_or_else(|| {
                warn!("player.ron: `base_luck` missing → using default {DEFAULT_BASE_LUCK}");
                DEFAULT_BASE_LUCK
            }),
            base_hp_regen: p.base_hp_regen.unwrap_or_else(|| {
                warn!(
                    "player.ron: `base_hp_regen` missing → using default {DEFAULT_BASE_HP_REGEN}"
                );
                DEFAULT_BASE_HP_REGEN
            }),
            base_xp_mult: p.base_xp_mult.unwrap_or_else(|| {
                warn!(
                    "player.ron: `base_xp_mult` missing → using default {DEFAULT_BASE_XP_MULT}"
                );
                DEFAULT_BASE_XP_MULT
            }),
            pickup_radius: p.pickup_radius.unwrap_or_else(|| {
                warn!(
                    "player.ron: `pickup_radius` missing → using default {DEFAULT_PICKUP_RADIUS}"
                );
                DEFAULT_PICKUP_RADIUS
            }),
            invincibility_time: p.invincibility_time.unwrap_or_else(|| {
                warn!(
                    "player.ron: `invincibility_time` missing → using default {DEFAULT_INVINCIBILITY_TIME}"
                );
                DEFAULT_INVINCIBILITY_TIME
            }),
            collider_radius: p.collider_radius.unwrap_or_else(|| {
                warn!(
                    "player.ron: `collider_radius` missing → using default {DEFAULT_COLLIDER_RADIUS}"
                );
                DEFAULT_COLLIDER_RADIUS
            }),
            collider_projectile_small: p.collider_projectile_small.unwrap_or_else(|| {
                warn!(
                    "player.ron: `collider_projectile_small` missing → using default {DEFAULT_COLLIDER_PROJECTILE_SMALL}"
                );
                DEFAULT_COLLIDER_PROJECTILE_SMALL
            }),
            collider_projectile_large: p.collider_projectile_large.unwrap_or_else(|| {
                warn!(
                    "player.ron: `collider_projectile_large` missing → using default {DEFAULT_COLLIDER_PROJECTILE_LARGE}"
                );
                DEFAULT_COLLIDER_PROJECTILE_LARGE
            }),
            collider_xp_gem: p.collider_xp_gem.unwrap_or_else(|| {
                warn!(
                    "player.ron: `collider_xp_gem` missing → using default {DEFAULT_COLLIDER_XP_GEM}"
                );
                DEFAULT_COLLIDER_XP_GEM
            }),
            collider_gold_coin: p.collider_gold_coin.unwrap_or_else(|| {
                warn!(
                    "player.ron: `collider_gold_coin` missing → using default {DEFAULT_COLLIDER_GOLD_COIN}"
                );
                DEFAULT_COLLIDER_GOLD_COIN
            }),
            collider_treasure: p.collider_treasure.unwrap_or_else(|| {
                warn!(
                    "player.ron: `collider_treasure` missing → using default {DEFAULT_COLLIDER_TREASURE}"
                );
                DEFAULT_COLLIDER_TREASURE
            }),
            gem_attraction_speed: p.gem_attraction_speed.unwrap_or_else(|| {
                warn!(
                    "player.ron: `gem_attraction_speed` missing → using default {DEFAULT_GEM_ATTRACTION_SPEED}"
                );
                DEFAULT_GEM_ATTRACTION_SPEED
            }),
            gem_absorption_radius: p.gem_absorption_radius.unwrap_or_else(|| {
                warn!(
                    "player.ron: `gem_absorption_radius` missing → using default {DEFAULT_GEM_ABSORPTION_RADIUS}"
                );
                DEFAULT_GEM_ABSORPTION_RADIUS
            }),
        }
    }
}

/// Resource holding the handle to the loaded player configuration.
#[derive(Resource)]
pub struct PlayerConfigHandle(pub Handle<PlayerConfig>);

// ---------------------------------------------------------------------------
// SystemParam bundle
// ---------------------------------------------------------------------------

/// SystemParam bundle for accessing [`PlayerConfig`].
///
/// Returns `None` while the asset is still loading or when
/// [`GameConfigPlugin`] has not been registered (e.g. in unit tests).
/// Call `.get()` to obtain `Option<&PlayerConfig>`.
///
/// [`GameConfigPlugin`]: crate::config::GameConfigPlugin
#[derive(SystemParam)]
pub struct PlayerParams<'w> {
    handle: Option<Res<'w, PlayerConfigHandle>>,
    assets: Option<Res<'w, Assets<PlayerConfig>>>,
}

impl<'w> PlayerParams<'w> {
    /// Returns the currently loaded [`PlayerConfig`], or `None` while loading.
    pub fn get(&self) -> Option<&PlayerConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }

    pub fn base_hp(&self) -> f32 {
        self.get().map(|c| c.base_hp).unwrap_or(DEFAULT_BASE_HP)
    }

    pub fn base_speed(&self) -> f32 {
        self.get()
            .map(|c| c.base_speed)
            .unwrap_or(DEFAULT_BASE_SPEED)
    }

    pub fn base_damage_mult(&self) -> f32 {
        self.get()
            .map(|c| c.base_damage_mult)
            .unwrap_or(DEFAULT_BASE_DAMAGE_MULT)
    }

    pub fn base_cooldown_reduction(&self) -> f32 {
        self.get()
            .map(|c| c.base_cooldown_reduction)
            .unwrap_or(DEFAULT_BASE_COOLDOWN_REDUCTION)
    }

    pub fn base_projectile_speed(&self) -> f32 {
        self.get()
            .map(|c| c.base_projectile_speed)
            .unwrap_or(DEFAULT_BASE_PROJECTILE_SPEED)
    }

    pub fn base_duration_mult(&self) -> f32 {
        self.get()
            .map(|c| c.base_duration_mult)
            .unwrap_or(DEFAULT_BASE_DURATION_MULT)
    }

    pub fn base_area_mult(&self) -> f32 {
        self.get()
            .map(|c| c.base_area_mult)
            .unwrap_or(DEFAULT_BASE_AREA_MULT)
    }

    pub fn base_luck(&self) -> f32 {
        self.get().map(|c| c.base_luck).unwrap_or(DEFAULT_BASE_LUCK)
    }

    pub fn base_hp_regen(&self) -> f32 {
        self.get()
            .map(|c| c.base_hp_regen)
            .unwrap_or(DEFAULT_BASE_HP_REGEN)
    }

    pub fn base_xp_mult(&self) -> f32 {
        self.get()
            .map(|c| c.base_xp_mult)
            .unwrap_or(DEFAULT_BASE_XP_MULT)
    }

    pub fn pickup_radius(&self) -> f32 {
        self.get()
            .map(|c| c.pickup_radius)
            .unwrap_or(DEFAULT_PICKUP_RADIUS)
    }

    pub fn invincibility_time(&self) -> f32 {
        self.get()
            .map(|c| c.invincibility_time)
            .unwrap_or(DEFAULT_INVINCIBILITY_TIME)
    }

    pub fn collider_radius(&self) -> f32 {
        self.get()
            .map(|c| c.collider_radius)
            .unwrap_or(DEFAULT_COLLIDER_RADIUS)
    }

    pub fn collider_projectile_small(&self) -> f32 {
        self.get()
            .map(|c| c.collider_projectile_small)
            .unwrap_or(DEFAULT_COLLIDER_PROJECTILE_SMALL)
    }

    pub fn collider_projectile_large(&self) -> f32 {
        self.get()
            .map(|c| c.collider_projectile_large)
            .unwrap_or(DEFAULT_COLLIDER_PROJECTILE_LARGE)
    }

    pub fn collider_xp_gem(&self) -> f32 {
        self.get()
            .map(|c| c.collider_xp_gem)
            .unwrap_or(DEFAULT_COLLIDER_XP_GEM)
    }

    pub fn collider_gold_coin(&self) -> f32 {
        self.get()
            .map(|c| c.collider_gold_coin)
            .unwrap_or(DEFAULT_COLLIDER_GOLD_COIN)
    }

    pub fn collider_treasure(&self) -> f32 {
        self.get()
            .map(|c| c.collider_treasure)
            .unwrap_or(DEFAULT_COLLIDER_TREASURE)
    }

    pub fn gem_attraction_speed(&self) -> f32 {
        self.get()
            .map(|c| c.gem_attraction_speed)
            .unwrap_or(DEFAULT_GEM_ATTRACTION_SPEED)
    }

    pub fn gem_absorption_radius(&self) -> f32 {
        self.get()
            .map(|c| c.gem_absorption_radius)
            .unwrap_or(DEFAULT_GEM_ABSORPTION_RADIUS)
    }
}

// ---------------------------------------------------------------------------
// Hot-reload system
// ---------------------------------------------------------------------------

/// Handles hot-reloading of player configuration.
///
/// On `Modified`, propagates all stat changes to the live [`PlayerStats`]
/// component. `current_hp` is intentionally left unchanged so the player
/// does not suddenly heal or die mid-run when tweaking values.
pub fn hot_reload_player_config(
    mut events: MessageReader<AssetEvent<PlayerConfig>>,
    config_assets: Res<Assets<PlayerConfig>>,
    config_handle: Res<PlayerConfigHandle>,
    mut player_q: Query<&mut crate::components::PlayerStats, With<crate::components::Player>>,
    meta: Res<MetaProgress>,
    game_params: super::GameParams,
) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id: _ } => {
                info!("✅ Player config loaded");
            }
            AssetEvent::Modified { id: _ } => {
                if let Some(cfg) = config_assets.get(&config_handle.0) {
                    info!("🔥 Hot-reloading player config!");
                    if let Ok(mut stats) = player_q.single_mut() {
                        stats.max_hp = cfg.base_hp;
                        // current_hp intentionally not reset — avoids instant kill/heal mid-run.
                        stats.move_speed = cfg.base_speed;
                        stats.damage_multiplier = cfg.base_damage_mult;
                        stats.cooldown_reduction = cfg.base_cooldown_reduction;
                        stats.projectile_speed_mult = cfg.base_projectile_speed;
                        stats.duration_multiplier = cfg.base_duration_mult;
                        stats.pickup_radius = cfg.pickup_radius;
                        stats.gem_attraction_speed = cfg.gem_attraction_speed;
                        stats.gem_absorption_radius = cfg.gem_absorption_radius;
                        stats.area_multiplier = cfg.base_area_mult;
                        stats.luck = cfg.base_luck;
                        stats.hp_regen = cfg.base_hp_regen;
                        // Reset to config base, then re-apply any BonusXp meta purchases
                        // so hot-reloading player.ron mid-run does not strip the bonus.
                        stats.xp_multiplier = cfg.base_xp_mult;
                        let xp_bonus: f32 = meta
                            .purchased_upgrades
                            .iter()
                            .filter(|&&u| u == MetaUpgradeType::BonusXp)
                            .count() as f32
                            * game_params.upgrade_stat_bonus(MetaUpgradeType::BonusXp);
                        stats.xp_multiplier += xp_bonus;
                        info!("✨ PlayerStats updated from hot-reload");
                    }
                }
            }
            AssetEvent::Removed { id: _ } => {
                warn!("⚠️ Player config removed");
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_config_deserialization() {
        let ron_data = r#"
PlayerConfig(
    base_hp: 100.0,
    base_speed: 200.0,
    base_damage_mult: 1.0,
    base_cooldown_reduction: 0.0,
    base_projectile_speed: 1.0,
    base_duration_mult: 1.0,
    base_area_mult: 1.0,
    base_luck: 1.0,
    base_hp_regen: 0.0,
    base_xp_mult: 1.0,
    pickup_radius: 80.0,
    invincibility_time: 0.5,
    collider_radius: 12.0,
    collider_projectile_small: 5.0,
    collider_projectile_large: 10.0,
    collider_xp_gem: 6.0,
    collider_gold_coin: 6.0,
    collider_treasure: 20.0,
    gem_attraction_speed: 200.0,
    gem_absorption_radius: 8.0,
)
"#;
        let partial: PlayerConfigPartial = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(ron_data)
            .unwrap();
        let config = PlayerConfig::from(partial);
        assert_eq!(config.base_hp, 100.0);
        assert_eq!(config.base_speed, 200.0);
        assert_eq!(config.base_damage_mult, 1.0);
        assert_eq!(config.pickup_radius, 80.0);
        assert_eq!(config.gem_attraction_speed, 200.0);
        assert_eq!(config.gem_absorption_radius, 8.0);
        assert_eq!(config.collider_radius, 12.0);
        assert_eq!(config.collider_projectile_small, 5.0);
        assert_eq!(config.base_xp_mult, 1.0);
    }
}
