//! Fast, high-fidelity OpenType parser.

#![no_std]

pub mod cmap;
pub mod head;
pub mod hhea;
pub mod maxp;
pub mod os2;
pub mod parse;
pub mod post;
pub mod types;
pub mod vhea;

mod font;

pub use font::*;

/// Helper module for common parsing imports.
mod parse_prelude {
    pub use super::parse::*;
    pub use super::types::*;
}
