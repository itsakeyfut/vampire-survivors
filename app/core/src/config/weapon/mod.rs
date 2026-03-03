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

pub mod bible;
pub mod cross;
pub mod garlic;
pub mod knife;
pub mod magic_wand;
pub mod thunder_ring;
pub mod whip;

pub use bible::{BibleConfig, BibleConfigHandle, BibleParams};
pub use cross::{CrossConfig, CrossConfigHandle, CrossParams};
pub use garlic::{GarlicConfig, GarlicConfigHandle, GarlicParams};
pub use knife::{KnifeConfig, KnifeConfigHandle, KnifeParams};
pub use magic_wand::{MagicWandConfig, MagicWandConfigHandle, MagicWandParams};
pub use thunder_ring::{ThunderRingConfig, ThunderRingConfigHandle, ThunderRingParams};
pub use whip::{WhipConfig, WhipConfigHandle, WhipParams};
