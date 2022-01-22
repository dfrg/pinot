//! Fast, high-fidelity OpenType parser.

#![no_std]
#![cfg(feature = "write")]
extern crate alloc;

pub mod avar;
pub mod binary;
pub mod cmap;
pub mod colr;
pub mod cpal;
pub mod fvar;
pub mod fvar1;
pub mod gdef;
pub mod gpos;
pub mod gsub;
pub mod head;
pub mod hhea;
pub mod hmtx;
pub mod hvar;
pub mod maxp;
pub mod name;
pub mod os2;
pub mod otl;
pub mod parse;
pub mod post;
pub mod types;
pub mod var;
pub mod vhea;
pub mod vmtx;
pub mod vorg;
pub mod vvar;

mod font;

pub use font::*;

/// Helper module for common parsing imports.
mod parse_prelude {
    pub use super::parse::*;
    pub use super::types::*;
}
