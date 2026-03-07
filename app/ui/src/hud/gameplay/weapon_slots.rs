//! Weapon slots HUD widget.
//!
//! Displays 6 square slots near the bottom of the screen showing the player's
//! currently equipped weapons.  Empty slots are shown as dark boxes; equipped
//! slots show a colored box with a short weapon abbreviation.
//!
//! ```text
//! [ Wh ][ MW ][    ][    ][    ][    ]
//! ```

use bevy::prelude::*;
use vs_core::components::{Player, WeaponInventory};
use vs_core::types::WeaponType;

use crate::config::hud::gameplay::WeaponSlotsHudConfig;

// ---------------------------------------------------------------------------
// Fallback constants
// ---------------------------------------------------------------------------

const DEFAULT_SLOT_SIZE: f32 = 40.0;
const DEFAULT_SLOT_GAP: f32 = 4.0;
const DEFAULT_SLOT_RADIUS: f32 = 4.0;
const DEFAULT_LABEL_FONT_SIZE: f32 = 10.0;
const DEFAULT_EMPTY_COLOR: Color = Color::srgb(0.10, 0.07, 0.15);
const DEFAULT_ACTIVE_COLOR: Color = Color::srgb(0.35, 0.20, 0.55);
const DEFAULT_TEXT_COLOR: Color = Color::srgb(0.95, 0.90, 0.85);

/// Total number of weapon slots displayed.
pub const WEAPON_SLOT_COUNT: usize = 6;

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

/// Marks a weapon slot background node.
///
/// `index` is the slot's zero-based position in [`WeaponInventory::weapons`].
#[derive(Component, Debug)]
pub struct HudWeaponSlot {
    pub index: usize,
}

/// Marks the text label inside a weapon slot.
///
/// `index` matches the parent [`HudWeaponSlot`].
#[derive(Component, Debug)]
pub struct HudWeaponSlotLabel {
    pub index: usize,
}

// ---------------------------------------------------------------------------
// Display helpers
// ---------------------------------------------------------------------------

/// Returns a short abbreviation for a weapon type, shown inside the slot.
pub fn weapon_abbr(wt: WeaponType) -> &'static str {
    match wt {
        WeaponType::Whip => "Wh",
        WeaponType::MagicWand => "MW",
        WeaponType::Knife => "Kn",
        WeaponType::Garlic => "Ga",
        WeaponType::Bible => "Bi",
        WeaponType::ThunderRing => "TR",
        WeaponType::Cross => "Cr",
        WeaponType::FireWand => "FW",
        WeaponType::BloodyTear => "BT",
        WeaponType::HolyWand => "HW",
        WeaponType::ThousandEdge => "TE",
        WeaponType::SoulEater => "SE",
        WeaponType::UnholyVespers => "UV",
        WeaponType::LightningRing => "LR",
    }
}

// ---------------------------------------------------------------------------
// Spawn
// ---------------------------------------------------------------------------

/// Spawns 6 weapon slots in a horizontal row as children of `parent`.
///
/// `cfg` is `None` while the RON asset is loading; fallback constants are used
/// in that case.
pub fn spawn_weapon_slots(parent: &mut ChildSpawnerCommands, cfg: Option<&WeaponSlotsHudConfig>) {
    let slot_size = cfg.map(|c| c.slot_size).unwrap_or(DEFAULT_SLOT_SIZE);
    let slot_gap = cfg.map(|c| c.slot_gap).unwrap_or(DEFAULT_SLOT_GAP);
    let slot_radius = cfg.map(|c| c.slot_radius).unwrap_or(DEFAULT_SLOT_RADIUS);
    let label_font_size = cfg
        .map(|c| c.label_font_size)
        .unwrap_or(DEFAULT_LABEL_FONT_SIZE);
    let empty_color = cfg
        .map(|c| Color::from(&c.empty_color))
        .unwrap_or(DEFAULT_EMPTY_COLOR);
    let text_color = cfg
        .map(|c| Color::from(&c.text_color))
        .unwrap_or(DEFAULT_TEXT_COLOR);

    // Row container.
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(slot_gap),
            ..default()
        })
        .with_children(|row| {
            for i in 0..WEAPON_SLOT_COUNT {
                row.spawn((
                    Node {
                        width: Val::Px(slot_size),
                        height: Val::Px(slot_size),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(empty_color),
                    BorderRadius::all(Val::Px(slot_radius)),
                    HudWeaponSlot { index: i },
                ))
                .with_children(|slot| {
                    slot.spawn((
                        Text::new(""),
                        TextFont {
                            font_size: label_font_size,
                            ..default()
                        },
                        TextColor(text_color),
                        Visibility::Hidden,
                        HudWeaponSlotLabel { index: i },
                    ));
                });
            }
        });
}

// ---------------------------------------------------------------------------
// Update system
// ---------------------------------------------------------------------------

/// Updates weapon slot backgrounds and labels from [`WeaponInventory`] each frame.
///
/// - Empty slots: dark background, hidden label.
/// - Occupied slots: purple background, visible label with weapon abbreviation.
pub fn update_weapon_slots(
    player_q: Query<&WeaponInventory, With<Player>>,
    mut slot_q: Query<(&HudWeaponSlot, &mut BackgroundColor)>,
    mut label_q: Query<(&HudWeaponSlotLabel, &mut Text, &mut Visibility)>,
    cfg: crate::config::hud::gameplay::WeaponSlotsHudParams<'_>,
) {
    let Ok(inv) = player_q.single() else {
        return;
    };

    let active_color = cfg
        .get()
        .map(|c| Color::from(&c.active_color))
        .unwrap_or(DEFAULT_ACTIVE_COLOR);
    let empty_color = cfg
        .get()
        .map(|c| Color::from(&c.empty_color))
        .unwrap_or(DEFAULT_EMPTY_COLOR);

    for (slot, mut bg) in slot_q.iter_mut() {
        bg.0 = if slot.index < inv.weapons.len() {
            active_color
        } else {
            empty_color
        };
    }

    for (label, mut text, mut vis) in label_q.iter_mut() {
        if let Some(weapon_state) = inv.weapons.get(label.index) {
            *text = Text::new(weapon_abbr(weapon_state.weapon_type));
            *vis = Visibility::Visible;
        } else {
            *text = Text::new("");
            *vis = Visibility::Hidden;
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce as _;

    use super::*;
    use vs_core::types::WeaponState;

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app
    }

    fn spawn_slots(app: &mut App) -> Vec<(Entity, Entity)> {
        (0..WEAPON_SLOT_COUNT)
            .map(|i| {
                let label = app
                    .world_mut()
                    .spawn((
                        Text::new(""),
                        Visibility::Hidden,
                        HudWeaponSlotLabel { index: i },
                    ))
                    .id();
                let slot = app
                    .world_mut()
                    .spawn((
                        BackgroundColor(DEFAULT_EMPTY_COLOR),
                        HudWeaponSlot { index: i },
                    ))
                    .id();
                (slot, label)
            })
            .collect()
    }

    fn spawn_player_with_weapons(app: &mut App, weapons: &[WeaponType]) {
        let inv = WeaponInventory {
            weapons: weapons.iter().map(|&wt| WeaponState::new(wt)).collect(),
        };
        app.world_mut().spawn((Player, inv));
    }

    /// With no weapons equipped, all slots show the empty color.
    #[test]
    fn no_weapons_all_slots_empty() {
        let mut app = build_app();
        let slots = spawn_slots(&mut app);
        spawn_player_with_weapons(&mut app, &[]);

        app.world_mut()
            .run_system_once(update_weapon_slots)
            .unwrap();

        for (slot_entity, _) in &slots {
            let bg = app.world().get::<BackgroundColor>(*slot_entity).unwrap();
            assert_eq!(
                bg.0.to_srgba(),
                DEFAULT_EMPTY_COLOR.to_srgba(),
                "slot should be empty color"
            );
        }
    }

    /// Equipped slots use the active color; remaining slots stay empty.
    #[test]
    fn equipped_slots_show_active_color() {
        let mut app = build_app();
        let slots = spawn_slots(&mut app);
        spawn_player_with_weapons(&mut app, &[WeaponType::Whip, WeaponType::Knife]);

        app.world_mut()
            .run_system_once(update_weapon_slots)
            .unwrap();

        // First two slots should be active.
        for i in 0..2 {
            let bg = app.world().get::<BackgroundColor>(slots[i].0).unwrap();
            assert_eq!(
                bg.0.to_srgba(),
                DEFAULT_ACTIVE_COLOR.to_srgba(),
                "slot {i} should be active"
            );
        }
        // Remaining slots should be empty.
        for i in 2..WEAPON_SLOT_COUNT {
            let bg = app.world().get::<BackgroundColor>(slots[i].0).unwrap();
            assert_eq!(
                bg.0.to_srgba(),
                DEFAULT_EMPTY_COLOR.to_srgba(),
                "slot {i} should be empty"
            );
        }
    }

    /// The label for an equipped slot is visible and shows the weapon abbreviation.
    #[test]
    fn equipped_slot_label_is_visible_with_abbr() {
        let mut app = build_app();
        let slots = spawn_slots(&mut app);
        spawn_player_with_weapons(&mut app, &[WeaponType::Whip]);

        app.world_mut()
            .run_system_once(update_weapon_slots)
            .unwrap();

        let (_, label_entity) = slots[0];
        let vis = app.world().get::<Visibility>(label_entity).unwrap();
        assert_eq!(*vis, Visibility::Visible);
        let text = app.world().get::<Text>(label_entity).unwrap();
        assert_eq!(text.0.as_str(), weapon_abbr(WeaponType::Whip));
    }

    /// The label for an empty slot is hidden.
    #[test]
    fn empty_slot_label_is_hidden() {
        let mut app = build_app();
        let slots = spawn_slots(&mut app);
        spawn_player_with_weapons(&mut app, &[]);

        app.world_mut()
            .run_system_once(update_weapon_slots)
            .unwrap();

        let (_, label_entity) = slots[0];
        let vis = app.world().get::<Visibility>(label_entity).unwrap();
        assert_eq!(*vis, Visibility::Hidden);
    }

    #[test]
    fn weapon_abbr_all_types_covered() {
        let types = [
            WeaponType::Whip,
            WeaponType::MagicWand,
            WeaponType::Knife,
            WeaponType::Garlic,
            WeaponType::Bible,
            WeaponType::ThunderRing,
            WeaponType::Cross,
            WeaponType::FireWand,
            WeaponType::BloodyTear,
            WeaponType::HolyWand,
            WeaponType::ThousandEdge,
            WeaponType::SoulEater,
            WeaponType::UnholyVespers,
            WeaponType::LightningRing,
        ];
        for wt in types {
            let abbr = weapon_abbr(wt);
            assert!(
                !abbr.is_empty(),
                "abbreviation for {wt:?} must not be empty"
            );
            assert!(
                abbr.len() <= 4,
                "abbreviation for {wt:?} should be short: '{abbr}'"
            );
        }
    }
}
