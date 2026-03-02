//! Level-up upgrade card selection screen.
//!
//! Shown when the game enters [`AppState::LevelUp`].  Displays up to three
//! upgrade cards read from [`LevelUpChoices`].  Clicking a card resumes the
//! run by transitioning back to [`AppState::Playing`].
//!
//! All entities carry [`DespawnOnExit`]`(`[`AppState::LevelUp`]`)` so Bevy
//! cleans them up automatically when the state leaves.

use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;
use vs_core::resources::LevelUpChoices;
use vs_core::states::AppState;
use vs_core::types::{PassiveItemType, UpgradeChoice, WeaponType};

use crate::components::{ButtonAction, MenuButton};
use crate::styles::{
    DEFAULT_FONT_SIZE_LARGE, DEFAULT_FONT_SIZE_MEDIUM, DEFAULT_FONT_SIZE_SMALL, DEFAULT_TEXT_COLOR,
};

// ---------------------------------------------------------------------------
// Local layout constants
// ---------------------------------------------------------------------------

/// Semi-transparent overlay behind the cards.
const OVERLAY_COLOR: Color = Color::srgba(0.02, 0.02, 0.06, 0.92);

/// "LEVEL UP!" heading color — gold.
const HEADING_COLOR: Color = Color::srgb(1.0, 0.85, 0.20);

/// Card background color (resting state).
const CARD_NORMAL: Color = Color::srgb(0.12, 0.08, 0.28);

/// Card background color on hover.
const CARD_HOVER: Color = Color::srgb(0.22, 0.14, 0.48);

/// Card background color while pressed.
const CARD_PRESSED: Color = Color::srgb(0.08, 0.05, 0.18);

/// Subtitle color — dim gold that identifies the upgrade type.
const SUBTITLE_COLOR: Color = Color::srgb(0.85, 0.70, 0.30);

/// Width of each upgrade card in pixels.
const CARD_WIDTH: f32 = 260.0;

/// Height of each upgrade card in pixels.
const CARD_HEIGHT: f32 = 320.0;

/// Horizontal gap between adjacent cards in pixels.
const CARD_GAP: f32 = 30.0;

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

/// Marks the root overlay node of the level-up screen.
#[derive(Component)]
pub struct LevelUpScreenBg;

/// Marks the row container that holds all upgrade cards.
#[derive(Component)]
pub struct LevelUpCardRow;

/// Marks an individual upgrade card button.
///
/// The `index` field corresponds to the card's position in
/// [`LevelUpChoices::choices`].
#[derive(Component)]
pub struct LevelUpCard {
    /// Zero-based index of this card in the choices list.
    pub index: usize,
}

// ---------------------------------------------------------------------------
// Display text helpers
// ---------------------------------------------------------------------------

fn choice_subtitle(choice: &UpgradeChoice) -> &'static str {
    match choice {
        UpgradeChoice::NewWeapon(_) => "New Weapon",
        UpgradeChoice::WeaponUpgrade(_) => "Weapon Upgrade",
        UpgradeChoice::PassiveItem(_) => "New Passive",
        UpgradeChoice::PassiveUpgrade(_) => "Passive Upgrade",
    }
}

fn choice_name(choice: &UpgradeChoice) -> &'static str {
    match choice {
        UpgradeChoice::NewWeapon(wt) | UpgradeChoice::WeaponUpgrade(wt) => weapon_name(*wt),
        UpgradeChoice::PassiveItem(pt) | UpgradeChoice::PassiveUpgrade(pt) => passive_name(*pt),
    }
}

fn choice_description(choice: &UpgradeChoice) -> &'static str {
    match choice {
        UpgradeChoice::NewWeapon(wt) | UpgradeChoice::WeaponUpgrade(wt) => {
            weapon_description(*wt)
        }
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
// System
// ---------------------------------------------------------------------------

/// Spawns the level-up card selection overlay when entering [`AppState::LevelUp`].
///
/// Reads [`LevelUpChoices`] and spawns one interactive card per choice.
/// Each card is tagged with [`DespawnOnExit`]`(`[`AppState::LevelUp`]`)` via
/// the root overlay, so all nodes are automatically removed when the overlay
/// closes.
pub fn setup_level_up_screen(
    mut commands: Commands,
    choices: Res<LevelUpChoices>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    // Guard against an empty pool (all items maxed, or player query failed).
    // Without cards there is no way to dismiss the overlay, which would
    // soft-lock the game.  Return to Playing immediately in that case.
    if choices.choices.is_empty() {
        next_state.set(AppState::Playing);
        return;
    }

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(40.0),
                ..default()
            },
            BackgroundColor(OVERLAY_COLOR),
            DespawnOnExit(AppState::LevelUp),
            LevelUpScreenBg,
        ))
        .with_children(|root| {
            // "LEVEL UP!" heading.
            root.spawn((
                Text::new("LEVEL UP!"),
                TextFont {
                    font_size: DEFAULT_FONT_SIZE_LARGE,
                    ..default()
                },
                TextColor(HEADING_COLOR),
            ));

            // Row of upgrade cards.
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(CARD_GAP),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Stretch,
                    ..default()
                },
                LevelUpCardRow,
            ))
            .with_children(|row| {
                for (i, choice) in choices.choices.iter().enumerate() {
                    spawn_card(row, i, choice);
                }
            });
        });
}

/// Spawns a single upgrade card button inside the card row.
fn spawn_card(parent: &mut ChildSpawnerCommands, index: usize, choice: &UpgradeChoice) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(CARD_WIDTH),
                height: Val::Px(CARD_HEIGHT),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(CARD_NORMAL),
            MenuButton {
                action: ButtonAction::SelectUpgrade(index),
            },
            LevelUpCard { index },
        ))
        .with_children(|card| {
            // Subtitle: upgrade type label.
            card.spawn((
                Text::new(choice_subtitle(choice)),
                TextFont {
                    font_size: DEFAULT_FONT_SIZE_SMALL,
                    ..default()
                },
                TextColor(SUBTITLE_COLOR),
            ));

            // Item name.
            card.spawn((
                Text::new(choice_name(choice)),
                TextFont {
                    font_size: DEFAULT_FONT_SIZE_MEDIUM,
                    ..default()
                },
                TextColor(DEFAULT_TEXT_COLOR),
            ));

            // Effect description.
            card.spawn((
                Text::new(choice_description(choice)),
                TextFont {
                    font_size: DEFAULT_FONT_SIZE_SMALL,
                    ..default()
                },
                TextColor(DEFAULT_TEXT_COLOR),
                Node {
                    max_width: Val::Px(CARD_WIDTH - 32.0),
                    ..default()
                },
            ));
        });
}

/// Query filter for interactive [`LevelUpCard`] nodes.
type ChangedCard = (Changed<Interaction>, With<LevelUpCard>);

/// Overrides card button colors to use the card-specific palette instead of
/// the standard menu-button palette.
///
/// Runs every frame in all states so card hover/press feedback is always
/// available.
pub fn handle_card_interaction(
    mut card_q: Query<(&Interaction, &mut BackgroundColor), ChangedCard>,
) {
    for (interaction, mut bg) in card_q.iter_mut() {
        *bg = BackgroundColor(match interaction {
            Interaction::Pressed => CARD_PRESSED,
            Interaction::Hovered => CARD_HOVER,
            Interaction::None => CARD_NORMAL,
        });
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;
    use vs_core::types::{PassiveItemType, UpgradeChoice, WeaponType};

    use super::*;

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<AppState>();
        app.insert_resource(LevelUpChoices::default());
        app
    }

    fn enter_level_up(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::LevelUp);
        app.update();
        app.update();
    }

    fn populate_choices(app: &mut App, choices: Vec<UpgradeChoice>) {
        app.world_mut().resource_mut::<LevelUpChoices>().choices = choices;
    }

    // --- Display text helpers ---

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

    // --- Screen spawning ---

    /// No choices — soft-lock guard returns to Playing without spawning the
    /// overlay.
    #[test]
    fn setup_level_up_screen_with_no_choices_returns_to_playing() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::LevelUp), setup_level_up_screen);

        enter_level_up(&mut app);

        // Overlay must NOT have been spawned.
        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<LevelUpScreenBg>>();
        assert_eq!(
            q.iter(app.world()).count(),
            0,
            "no LevelUpScreenBg should exist when choices are empty"
        );

        // State should have been set back to Playing.
        assert_eq!(
            *app.world().resource::<State<AppState>>(),
            AppState::Playing,
            "state should return to Playing when there are no choices"
        );
    }

    /// Three choices produce exactly three LevelUpCard entities.
    #[test]
    fn setup_level_up_screen_spawns_one_card_per_choice() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::LevelUp), setup_level_up_screen);

        populate_choices(
            &mut app,
            vec![
                UpgradeChoice::NewWeapon(WeaponType::Whip),
                UpgradeChoice::PassiveItem(PassiveItemType::Spinach),
                UpgradeChoice::WeaponUpgrade(WeaponType::MagicWand),
            ],
        );

        enter_level_up(&mut app);

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<LevelUpCard>>();
        assert_eq!(
            q.iter(app.world()).count(),
            3,
            "exactly 3 LevelUpCard entities expected for 3 choices"
        );
    }

    /// Cards are interactive buttons.
    #[test]
    fn level_up_cards_are_buttons() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::LevelUp), setup_level_up_screen);

        populate_choices(
            &mut app,
            vec![
                UpgradeChoice::NewWeapon(WeaponType::Whip),
                UpgradeChoice::PassiveItem(PassiveItemType::Wings),
            ],
        );

        enter_level_up(&mut app);

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, (With<Button>, With<LevelUpCard>)>();
        assert_eq!(
            q.iter(app.world()).count(),
            2,
            "each card must be a Bevy Button"
        );
    }

    /// Cards carry SelectUpgrade actions with the correct index.
    #[test]
    fn cards_have_select_upgrade_actions_with_correct_indices() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::LevelUp), setup_level_up_screen);

        populate_choices(
            &mut app,
            vec![
                UpgradeChoice::NewWeapon(WeaponType::Knife),
                UpgradeChoice::PassiveItem(PassiveItemType::Clover),
            ],
        );

        enter_level_up(&mut app);

        let mut q = app.world_mut().query::<(&LevelUpCard, &MenuButton)>();
        let mut pairs: Vec<(usize, ButtonAction)> = q
            .iter(app.world())
            .map(|(card, btn)| (card.index, btn.action))
            .collect();
        pairs.sort_by_key(|(i, _)| *i);

        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0], (0, ButtonAction::SelectUpgrade(0)));
        assert_eq!(pairs[1], (1, ButtonAction::SelectUpgrade(1)));
    }

    /// Screen despawns when leaving LevelUp state.
    #[test]
    fn level_up_screen_despawns_on_exit() {
        let mut app = build_app();
        app.add_systems(OnEnter(AppState::LevelUp), setup_level_up_screen);

        populate_choices(
            &mut app,
            vec![UpgradeChoice::NewWeapon(WeaponType::Whip)],
        );

        enter_level_up(&mut app);

        // Verify overlay present.
        {
            let mut q = app
                .world_mut()
                .query_filtered::<Entity, With<LevelUpScreenBg>>();
            assert_eq!(q.iter(app.world()).count(), 1);
        }

        // Leave LevelUp state.
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Playing);
        app.update();
        app.update();

        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<LevelUpScreenBg>>();
        assert_eq!(
            q.iter(app.world()).count(),
            0,
            "level-up overlay must despawn after leaving LevelUp state"
        );
    }
}
