//! Per-weapon configuration modules.
//!
//! Each weapon has its own RON file and Rust module so that adding a new
//! weapon only requires a new file — no shared struct needs to grow.
//!
//! | RON file                            | Config type           | Weapon                       |
//! |-------------------------------------|-----------------------|------------------------------|
//! | `config/weapons/whip.ron`           | [`WhipConfig`]        | Whip / BloodyTear            |
//! | `config/weapons/magic_wand.ron`     | [`MagicWandConfig`]   | Magic Wand / HolyWand        |
//! | `config/weapons/knife.ron`          | [`KnifeConfig`]       | Knife / ThousandEdge         |
//! | `config/weapons/garlic.ron`         | [`GarlicConfig`]      | Garlic / SoulEater           |
//! | `config/weapons/bible.ron`          | [`BibleConfig`]       | Bible / UnholyVespers        |
//! | `config/weapons/thunder_ring.ron`   | [`ThunderRingConfig`] | Thunder Ring / LightningRing |
//! | `config/weapons/cross.ron`          | [`CrossConfig`]       | Cross                        |
//! | `config/weapons/fire_wand.ron`      | [`FireWandConfig`]    | Fire Wand                    |

pub mod bible;
pub mod cross;
pub mod fire_wand;
pub mod garlic;
pub mod knife;
pub mod magic_wand;
pub mod thunder_ring;
pub mod whip;

pub use bible::{BibleConfig, BibleConfigHandle, BibleParams};
pub use cross::{CrossConfig, CrossConfigHandle, CrossParams};
pub use fire_wand::{FireWandConfig, FireWandConfigHandle, FireWandParams};
pub use garlic::{GarlicConfig, GarlicConfigHandle, GarlicParams};
pub use knife::{KnifeConfig, KnifeConfigHandle, KnifeParams};
pub use magic_wand::{MagicWandConfig, MagicWandConfigHandle, MagicWandParams};
pub use thunder_ring::{ThunderRingConfig, ThunderRingConfigHandle, ThunderRingParams};
pub use whip::{WhipConfig, WhipConfigHandle, WhipParams};

// Re-export Partial types so config/mod.rs loaders can reference them.
pub(crate) use bible::BibleConfigPartial;
pub(crate) use cross::CrossConfigPartial;
pub(crate) use fire_wand::FireWandConfigPartial;
pub(crate) use garlic::GarlicConfigPartial;
pub(crate) use knife::KnifeConfigPartial;
pub(crate) use magic_wand::MagicWandConfigPartial;
pub(crate) use thunder_ring::ThunderRingConfigPartial;
pub(crate) use whip::WhipConfigPartial;
