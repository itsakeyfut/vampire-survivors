//! Upgrade card HUD widget.
//!
//! Spawns individual upgrade selection cards and handles their interaction
//! colors.  Display text helpers (weapon/passive names and descriptions) also
//! live here so they can be reused from other screens in the future (e.g. the
//! meta-progression shop).

use bevy::prelude::*;
use vs_core::types::{PassiveItemType, UpgradeChoice, WeaponType};

use crate::components::{ButtonAction, MenuButton};
use crate::config::hud::upgrade_card::{UpgradeCardHudConfig, UpgradeCardHudParams};

// ---------------------------------------------------------------------------
// Fallback constants
// ---------------------------------------------------------------------------

const DEFAULT_CARD_NORMAL: Color = Color::srgb(0.12, 0.08, 0.28);
const DEFAULT_CARD_HOVER: Color = Color::srgb(0.22, 0.14, 0.48);
const DEFAULT_CARD_PRESSED: Color = Color::srgb(0.08, 0.05, 0.18);
const DEFAULT_SUBTITLE_COLOR: Color = Color::srgb(0.85, 0.70, 0.30);
const DEFAULT_TEXT_COLOR: Color = Color::srgb(0.95, 0.90, 0.85);
const DEFAULT_CARD_WIDTH: f32 = 260.0;
const DEFAULT_CARD_HEIGHT: f32 = 320.0;
const DEFAULT_PADDING: f32 = 16.0;
const DEFAULT_INNER_GAP: f32 = 12.0;
const DEFAULT_FONT_SIZE_NAME: f32 = 32.0;
const DEFAULT_FONT_SIZE_SUBTITLE: f32 = 24.0;
const DEFAULT_FONT_SIZE_DESC: f32 = 24.0;

// ---------------------------------------------------------------------------
// Marker component
// ---------------------------------------------------------------------------

/// Marker component placed on every upgrade card button entity.
///
/// The `index` field corresponds to the card's position in
/// [`vs_core::resources::LevelUpChoices::choices`].
#[derive(Component)]
pub struct UpgradeCardHud {
    /// Zero-based index of this card in the choices list.
    pub index: usize,
}

// ---------------------------------------------------------------------------
// Display text helpers
// ---------------------------------------------------------------------------

/// Returns the upgrade-type subtitle (e.g. "New Weapon").
pub fn choice_subtitle(choice: &UpgradeChoice) -> &'static str {
    match choice {
        UpgradeChoice::NewWeapon(_) => "New Weapon",
        UpgradeChoice::WeaponUpgrade(_) => "Weapon Upgrade",
        UpgradeChoice::PassiveItem(_) => "New Passive",
        UpgradeChoice::PassiveUpgrade(_) => "Passive Upgrade",
    }
}

/// Returns the item name for a choice.
pub fn choice_name(choice: &UpgradeChoice) -> &'static str {
    match choice {
        UpgradeChoice::NewWeapon(wt) | UpgradeChoice::WeaponUpgrade(wt) => weapon_name(*wt),
        UpgradeChoice::PassiveItem(pt) | UpgradeChoice::PassiveUpgrade(pt) => passive_name(*pt),
    }
}

/// Returns the one-line effect description for a choice.
pub fn choice_description(choice: &UpgradeChoice) -> &'static str {
    match choice {
        UpgradeChoice::NewWeapon(wt) | UpgradeChoice::WeaponUpgrade(wt) => weapon_description(*wt),
        UpgradeChoice::PassiveItem(pt) | UpgradeChoice::PassiveUpgrade(pt) => {
            passive_description(*pt)
        }
    }
}

fn weapon_name(wt: WeaponType) -> &'static str {
    match wt {
        WeaponType::Whip => "Whip",
        WeaponType::MagicWand => "Magic Wand",
        WeaponType::Knife => "Knife",
        WeaponType::Garlic => "Garlic",
        WeaponType::Bible => "Bible",
        WeaponType::ThunderRing => "Thunder Ring",
        WeaponType::Cross => "Cross",
        WeaponType::FireWand => "Fire Wand",
        WeaponType::BloodyTear => "Bloody Tear",
        WeaponType::HolyWand => "Holy Wand",
        WeaponType::ThousandEdge => "Thousand Edge",
        WeaponType::SoulEater => "Soul Eater",
        WeaponType::UnholyVespers => "Unholy Vespers",
        WeaponType::LightningRing => "Lightning Ring",
    }
}

fn weapon_description(wt: WeaponType) -> &'static str {
    match wt {
        WeaponType::Whip => "Fan-shaped swing, alternating sides.",
        WeaponType::MagicWand => "Homing projectile toward nearest enemy.",
        WeaponType::Knife => "Fast piercing shot in movement direction.",
        WeaponType::Garlic => "Continuous damage aura around player.",
        WeaponType::Bible => "Orbiting projectile that circles the player.",
        WeaponType::ThunderRing => "Random lightning strikes across the screen.",
        WeaponType::Cross => "Boomerang that flies out and returns.",
        WeaponType::FireWand => "Fireball targeting the highest-HP enemy.",
        WeaponType::BloodyTear => "Evolved Whip — massive area slash.",
        WeaponType::HolyWand => "Evolved Magic Wand — rapid homing bolts.",
        WeaponType::ThousandEdge => "Evolved Knife — endless blade flurry.",
        WeaponType::SoulEater => "Evolved Garlic — drains life from enemies.",
        WeaponType::UnholyVespers => "Evolved Bible — infinite orbiting blades.",
        WeaponType::LightningRing => "Evolved Thunder Ring — storm of lightning.",
    }
}

fn passive_name(pt: PassiveItemType) -> &'static str {
    match pt {
        PassiveItemType::Spinach => "Spinach",
        PassiveItemType::Wings => "Wings",
        PassiveItemType::HollowHeart => "Hollow Heart",
        PassiveItemType::Clover => "Clover",
        PassiveItemType::EmptyTome => "Empty Tome",
        PassiveItemType::Bracer => "Bracer",
        PassiveItemType::Spellbinder => "Spellbinder",
        PassiveItemType::Duplicator => "Duplicator",
        PassiveItemType::Pummarola => "Pummarola",
    }
}

fn passive_description(pt: PassiveItemType) -> &'static str {
    match pt {
        PassiveItemType::Spinach => "+10% damage per level.",
        PassiveItemType::Wings => "+10% move speed per level.",
        PassiveItemType::HollowHeart => "+20% max HP per level.",
        PassiveItemType::Clover => "+10% luck per level.",
        PassiveItemType::EmptyTome => "-8% weapon cooldown per level.",
        PassiveItemType::Bracer => "+10% projectile speed per level.",
        PassiveItemType::Spellbinder => "+10% weapon duration per level.",
        PassiveItemType::Duplicator => "+1 projectile count per level.",
        PassiveItemType::Pummarola => "+0.5 HP regeneration/s per level.",
    }
}

// ---------------------------------------------------------------------------
// Spawn function
// ---------------------------------------------------------------------------

/// Spawns a single upgrade card button inside `parent`.
///
/// - `index`  — zero-based position in the choices list (stored in [`UpgradeCardHud`]).
/// - `choice` — the upgrade to display.
/// - `cfg`    — layout/color config; pass `card_params.get()`. Falls back to
///   `DEFAULT_*` constants when the asset is not yet loaded.
///
/// Card dimensions are clamped: `card_width >= 64` and `card_height >= 64` to
/// prevent zero-sized or negative-`max_width` layouts from invalid RON values.
pub fn spawn_upgrade_card(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    choice: &UpgradeChoice,
    cfg: Option<&UpgradeCardHudConfig>,
) {
    let card_width = cfg
        .map(|c| c.card_width)
        .unwrap_or(DEFAULT_CARD_WIDTH)
        .max(64.0);
    let card_height = cfg
        .map(|c| c.card_height)
        .unwrap_or(DEFAULT_CARD_HEIGHT)
        .max(64.0);
    let padding = cfg.map(|c| c.padding).unwrap_or(DEFAULT_PADDING).max(0.0);
    let inner_gap = cfg
        .map(|c| c.inner_gap)
        .unwrap_or(DEFAULT_INNER_GAP)
        .max(0.0);
    let card_normal = cfg
        .map(|c| Color::from(&c.card_normal))
        .unwrap_or(DEFAULT_CARD_NORMAL);
    let subtitle_color = cfg
        .map(|c| Color::from(&c.subtitle_color))
        .unwrap_or(DEFAULT_SUBTITLE_COLOR);
    let text_color = cfg
        .map(|c| Color::from(&c.text_color))
        .unwrap_or(DEFAULT_TEXT_COLOR);
    let font_size_name = cfg
        .map(|c| c.font_size_name)
        .unwrap_or(DEFAULT_FONT_SIZE_NAME)
        .max(1.0);
    let font_size_subtitle = cfg
        .map(|c| c.font_size_subtitle)
        .unwrap_or(DEFAULT_FONT_SIZE_SUBTITLE)
        .max(1.0);
    let font_size_desc = cfg
        .map(|c| c.font_size_desc)
        .unwrap_or(DEFAULT_FONT_SIZE_DESC)
        .max(1.0);

    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(card_width),
                height: Val::Px(card_height),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(padding)),
                row_gap: Val::Px(inner_gap),
                ..default()
            },
            BackgroundColor(card_normal),
            MenuButton {
                action: ButtonAction::SelectUpgrade(index),
            },
            UpgradeCardHud { index },
        ))
        .with_children(|card| {
            // Subtitle: upgrade type label.
            card.spawn((
                Text::new(choice_subtitle(choice)),
                TextFont {
                    font_size: font_size_subtitle,
                    ..default()
                },
                TextColor(subtitle_color),
            ));

            // Item name.
            card.spawn((
                Text::new(choice_name(choice)),
                TextFont {
                    font_size: font_size_name,
                    ..default()
                },
                TextColor(text_color),
            ));

            // Effect description.
            card.spawn((
                Text::new(choice_description(choice)),
                TextFont {
                    font_size: font_size_desc,
                    ..default()
                },
                TextColor(text_color),
                Node {
                    max_width: Val::Px((card_width - padding * 2.0).max(0.0)),
                    ..default()
                },
            ));
        });
}

// ---------------------------------------------------------------------------
// Interaction handler
// ---------------------------------------------------------------------------

/// Query filter for changed [`UpgradeCardHud`] interactions.
type ChangedCard = (Changed<Interaction>, With<UpgradeCardHud>);

/// Overrides card button colors with the card-specific palette.
///
/// Runs every frame so hover and press feedback is always available.
/// Must run after [`crate::components::handle_button_interaction`] so card
/// colors take precedence (cards carry both [`MenuButton`] and [`UpgradeCardHud`]).
pub fn handle_card_interaction(
    mut card_q: Query<(&Interaction, &mut BackgroundColor), ChangedCard>,
    card_cfg: UpgradeCardHudParams,
) {
    let (normal, hover, pressed) = if let Some(c) = card_cfg.get() {
        (
            Color::from(&c.card_normal),
            Color::from(&c.card_hover),
            Color::from(&c.card_pressed),
        )
    } else {
        (
            DEFAULT_CARD_NORMAL,
            DEFAULT_CARD_HOVER,
            DEFAULT_CARD_PRESSED,
        )
    };

    for (interaction, mut bg) in card_q.iter_mut() {
        *bg = BackgroundColor(match interaction {
            Interaction::Pressed => pressed,
            Interaction::Hovered => hover,
            Interaction::None => normal,
        });
    }
}

// ---------------------------------------------------------------------------
// Hot-reload system
// ---------------------------------------------------------------------------

/// Logs when `config/ui/hud/upgrade_card.ron` is loaded or hot-reloaded.
///
/// # Design note — no live entity updates
///
/// Upgrade cards are spawned as children of a transient level-up overlay
/// (`DespawnOnExit(AppState::LevelUp)`).  Because the overlay is recreated
/// fresh on every `OnEnter(LevelUp)`, layout dimensions cannot be patched on
/// existing entities without a full despawn/respawn cycle.
///
/// - **Hover/press colors** are already live: [`handle_card_interaction`]
///   reads from [`UpgradeCardHudParams`] every frame.
/// - Edits to `upgrade_card.ron` take full effect the next time the level-up
///   overlay opens, which is the correct hot-reload behaviour for a transient screen.
pub fn hot_reload_upgrade_card_hud(mut events: MessageReader<AssetEvent<UpgradeCardHudConfig>>) {
    for event in events.read() {
        match event {
            AssetEvent::Added { .. } => {
                info!("✅ Upgrade card HUD config loaded");
            }
            AssetEvent::Modified { .. } => {
                info!("🔥 Upgrade card HUD config hot-reloaded");
            }
            AssetEvent::Removed { .. } => {
                warn!("⚠️ Upgrade card HUD config removed");
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
    fn weapon_name_covers_all_variants() {
        let weapons = [
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
        for wt in weapons {
            assert!(
                !weapon_name(wt).is_empty(),
                "weapon_name({wt:?}) must not be empty"
            );
            assert!(
                !weapon_description(wt).is_empty(),
                "weapon_description({wt:?}) must not be empty"
            );
        }
    }

    #[test]
    fn passive_name_covers_all_variants() {
        let passives = [
            PassiveItemType::Spinach,
            PassiveItemType::Wings,
            PassiveItemType::HollowHeart,
            PassiveItemType::Clover,
            PassiveItemType::EmptyTome,
            PassiveItemType::Bracer,
            PassiveItemType::Spellbinder,
            PassiveItemType::Duplicator,
            PassiveItemType::Pummarola,
        ];
        for pt in passives {
            assert!(
                !passive_name(pt).is_empty(),
                "passive_name({pt:?}) must not be empty"
            );
            assert!(
                !passive_description(pt).is_empty(),
                "passive_description({pt:?}) must not be empty"
            );
        }
    }

    #[test]
    fn choice_subtitle_returns_correct_category() {
        assert_eq!(
            choice_subtitle(&UpgradeChoice::NewWeapon(WeaponType::Whip)),
            "New Weapon"
        );
        assert_eq!(
            choice_subtitle(&UpgradeChoice::WeaponUpgrade(WeaponType::Whip)),
            "Weapon Upgrade"
        );
        assert_eq!(
            choice_subtitle(&UpgradeChoice::PassiveItem(PassiveItemType::Spinach)),
            "New Passive"
        );
        assert_eq!(
            choice_subtitle(&UpgradeChoice::PassiveUpgrade(PassiveItemType::Spinach)),
            "Passive Upgrade"
        );
    }

    #[test]
    fn spawn_upgrade_card_produces_button_with_marker() {
        use bevy::state::app::StatesPlugin;
        use vs_core::states::AppState;

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<AppState>();

        let choice = UpgradeChoice::NewWeapon(WeaponType::Whip);
        let mut cmds = app.world_mut().commands();
        cmds.spawn(Node::default()).with_children(|parent| {
            spawn_upgrade_card(parent, 0, &choice, None);
        });
        app.world_mut().flush();

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, (With<Button>, With<UpgradeCardHud>)>();
        assert_eq!(
            q.iter(app.world()).count(),
            1,
            "spawn_upgrade_card must produce one Button+UpgradeCardHud entity"
        );
    }
}
