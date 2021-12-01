//! OpenType layout common types.

mod context;
mod def;
mod lookup;
mod pos;
mod shared;
mod sub;
mod table;

pub use context::*;
pub use def::Gdef;
pub use lookup::{
    Lookup, LookupFilter, LookupFlag, LookupKind, LookupRecord, Subtable, SubtableKind,
    SubtableRecord,
};
pub use pos::*;
pub use shared::{ClassDef, Coverage, CoverageArray, Covered};
pub use sub::*;
pub use table::{
    Condition, ConditionSet, Feature, FeatureRecord, FeatureSubst, FeatureVariations, Language,
    LanguageRecord, Script, ScriptRecord,
};

use crate::parse_prelude::*;

#[doc(hidden)]
/// Represents the two phases of layout.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Stage {
    /// Stage where glyphs are substituted based on typographic rules.
    Substitution,
    /// Stage where glyphs are positioned based on typographic rules.
    PositionAdjustment,
}

#[doc(hidden)]
/// Layout table for a single stage.
#[derive(Copy, Clone)]
pub struct Layout<'a> {
    stage: Stage,
    data: Buffer<'a>,
    gdef: Option<Gdef<'a>>,
}

/// Type alias for a glyph class identifier.
pub type GlyphClass = u16;

/// Type alias for a mark class identifier.
pub type MarkAttachClass = u16;
