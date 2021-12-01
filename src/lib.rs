//! Fast, high-fidelity OpenType parser.

#![no_std]

pub mod parse;
pub mod types;

/// Helper module for common parsing imports.
mod parse_prelude {
    pub use super::parse::*;
    pub use super::types::*;
}
