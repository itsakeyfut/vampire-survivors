//! Weapon slots HUD configuration.
//!
//! Loaded from `assets/config/ui/hud/gameplay/weapon_slots.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

use crate::config::SrgbColor;

// ---------------------------------------------------------------------------
// Fallback constants (used while weapon_slots.ron is still loading)
// ---------------------------------------------------------------------------

const DEFAULT_SLOT_SIZE: f32 = 40.0;
const DEFAULT_SLOT_GAP: f32 = 4.0;
const DEFAULT_SLOT_RADIUS: f32 = 4.0;
const DEFAULT_LABEL_FONT_SIZE: f32 = 10.0;
const DEFAULT_EMPTY_COLOR: Color = Color::srgb(0.10, 0.07, 0.15);
const DEFAULT_ACTIVE_COLOR: Color = Color::srgb(0.35, 0.20, 0.55);
const DEFAULT_TEXT_COLOR: Color = Color::srgb(0.95, 0.90, 0.85);

/// Weapon slots HUD config loaded from
/// `config/ui/hud/gameplay/weapon_slots.ron`.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct WeaponSlotsHudConfig {
    /// Side length of each square slot in pixels.
    pub slot_size: f32,
    /// Gap between adjacent slots in pixels.
    pub slot_gap: f32,
    /// Corner radius of each slot in pixels.
    pub slot_radius: f32,
    /// Font size of the weapon abbreviation label in points.
    pub label_font_size: f32,
    /// Background color for empty (unfilled) slots.
    pub empty_color: SrgbColor,
    /// Background color for slots with a weapon equipped.
    pub active_color: SrgbColor,
    /// Text color for the weapon abbreviation label.
    pub text_color: SrgbColor,
}

/// Resource holding the handle to the loaded [`WeaponSlotsHudConfig`].
#[derive(Resource)]
pub struct WeaponSlotsHudConfigHandle(pub Handle<WeaponSlotsHudConfig>);

/// SystemParam for accessing [`WeaponSlotsHudConfig`].
///
/// Returns `None` while the asset is loading or the plugin is absent.
#[derive(SystemParam)]
pub struct WeaponSlotsHudParams<'w> {
    handle: Option<Res<'w, WeaponSlotsHudConfigHandle>>,
    assets: Option<Res<'w, Assets<WeaponSlotsHudConfig>>>,
}

impl<'w> WeaponSlotsHudParams<'w> {
    /// Returns the currently loaded config, or `None`.
    pub fn get(&self) -> Option<&WeaponSlotsHudConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }

    pub fn slot_size(&self) -> f32 {
        self.get().map(|c| c.slot_size).unwrap_or(DEFAULT_SLOT_SIZE)
    }

    pub fn slot_gap(&self) -> f32 {
        self.get().map(|c| c.slot_gap).unwrap_or(DEFAULT_SLOT_GAP)
    }

    pub fn slot_radius(&self) -> f32 {
        self.get()
            .map(|c| c.slot_radius)
            .unwrap_or(DEFAULT_SLOT_RADIUS)
    }

    pub fn label_font_size(&self) -> f32 {
        self.get()
            .map(|c| c.label_font_size)
            .unwrap_or(DEFAULT_LABEL_FONT_SIZE)
    }

    pub fn empty_color(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.empty_color))
            .unwrap_or(DEFAULT_EMPTY_COLOR)
    }

    pub fn active_color(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.active_color))
            .unwrap_or(DEFAULT_ACTIVE_COLOR)
    }

    pub fn text_color(&self) -> Color {
        self.get()
            .map(|c| Color::from(&c.text_color))
            .unwrap_or(DEFAULT_TEXT_COLOR)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const RON: &str = r#"
WeaponSlotsHudConfig(
    slot_size:       40.0,
    slot_gap:         4.0,
    slot_radius:      4.0,
    label_font_size:  10.0,
    empty_color:  (r: 0.10, g: 0.07, b: 0.15),
    active_color: (r: 0.35, g: 0.20, b: 0.55),
    text_color:   (r: 0.95, g: 0.90, b: 0.85),
)
"#;

    #[test]
    fn weapon_slots_hud_config_deserialization() {
        let cfg: WeaponSlotsHudConfig = ron::de::from_str(RON).expect("RON parse must succeed");
        assert_eq!(cfg.slot_size, 40.0);
        assert_eq!(cfg.slot_gap, 4.0);
        assert_eq!(cfg.slot_radius, 4.0);
        assert_eq!(cfg.label_font_size, 10.0);
        assert!((cfg.empty_color.r - 0.10).abs() < 1e-6);
        assert!((cfg.active_color.r - 0.35).abs() < 1e-6);
    }

    #[test]
    fn slot_dimensions_are_positive() {
        let cfg: WeaponSlotsHudConfig = ron::de::from_str(RON).unwrap();
        assert!(cfg.slot_size > 0.0);
        assert!(cfg.label_font_size > 0.0);
    }
}
