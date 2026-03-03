//! Per-weapon configuration modules.
//!
//! Each weapon has its own RON file and Rust module so that adding a new
//! weapon only requires a new file — no shared struct needs to grow.
//!
//! | RON file                            | Config type         | Weapon                     |
//! |-------------------------------------|---------------------|----------------------------|
//! | `config/weapons/whip.ron`           | [`WhipConfig`]      | Whip / BloodyTear          |
//! | `config/weapons/magic_wand.ron`     | [`MagicWandConfig`] | Magic Wand / HolyWand      |
//! | `config/weapons/knife.ron`          | [`KnifeConfig`]     | Knife / ThousandEdge       |

pub mod knife;
pub mod magic_wand;
pub mod whip;

pub use knife::{KnifeConfig, KnifeConfigHandle, KnifeParams};
pub use magic_wand::{MagicWandConfig, MagicWandConfigHandle, MagicWandParams};
pub use whip::{WhipConfig, WhipConfigHandle, WhipParams};
