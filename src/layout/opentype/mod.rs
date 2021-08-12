//! OpenType layout.
//! 
//! The module covers the specification at <https://docs.microsoft.com/en-us/typography/opentype/spec/ttochap1>.
//! Documentation is currently sparse, but the main entry points are [`GlyphDef`] for access to the `GDEF`
//! table and [`Layout`] for access to the `GSUB` and `GPOS` tables.
//! 

mod context;
mod def;
mod lookup;
mod pos;
mod shared;
mod sub;
mod table;

pub use context::*;
pub use def::GlyphDef;
pub use lookup::{
    Lookup, LookupFilter, LookupFlag, LookupKind, LookupRecord, Subtable, SubtableKind,
    SubtableRecord,
};
pub use pos::*;
pub use shared::{ClassDef, Coverage, CoverageArray, Covered};
pub use sub::*;
pub use table::{
    Condition, ConditionSet, Feature, FeatureIndex, FeatureRecord, FeatureSubst, FeatureVariations,
    Language, LanguageRecord, Script, ScriptRecord,
};

use crate::container::prelude::*;

/// Tag for the `GDEF` table.
pub const GDEF: Tag = tag::from_bytes(b"GDEF");

/// Tag for the `GSUB` table.
pub const GSUB: Tag = tag::from_bytes(b"GSUB");

/// Tag for the `GPOS` table.
pub const GPOS: Tag = tag::from_bytes(b"GPOS");

/// Represents the two phases of layout.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Stage {
    /// Stage where glyphs are substituted based on typographic rules.
    Substitution,
    /// Stage where glyphs are positioned based on typographic rules.
    PositionAdjustment,
}

/// Layout table for a single stage.
#[derive(Copy, Clone)]
pub struct Layout<'a> {
    stage: Stage,
    data: Buffer<'a>,
    glyphs: Option<GlyphDef<'a>>,
}

/// Type alias for a glyph class identifier.
pub type GlyphClass = u16;

/// Type alias for a mark class identifier.
pub type MarkAttachClass = u16;
