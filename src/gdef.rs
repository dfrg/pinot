//! Glyph definition table.

use crate::parse_prelude::*;

/// Tag for the `GDEF` table.
pub const GDEF: Tag = Tag::new(b"GDEF");

#[doc(inline)]
pub use crate::otl::Gdef;
