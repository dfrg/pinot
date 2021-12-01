//! Glyph substitution table.

use crate::otl::*;
use crate::parse_prelude::*;

/// Tag for the `GSUB` table.
pub const GSUB: Tag = Tag::new(b"GSUB");

/// Glyph substitution table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/gsub>
#[derive(Copy, Clone)]
pub struct Gsub<'a>(pub Layout<'a>);

impl<'a> Gsub<'a> {
    /// Creates a new glyph substitution table from a byte slice containing
    /// the table data and an optional glyph definition table.
    pub fn new(data: &'a [u8], gdef: Option<Gdef<'a>>) -> Self {
        Self(Layout::new(Stage::Substitution, data, gdef))
    }

    /// Returns the associated glyph definitions.
    pub fn gdef(&self) -> Option<&Gdef<'a>> {
        self.0.gdef()
    }

    /// Returns the number of available scripts.
    pub fn num_scripts(&self) -> u16 {
        self.0.num_scripts()
    }

    /// Returns the script at the specified index.
    pub fn script(&'a self, index: u16) -> Option<Script<'a>> {
        self.0.script(index)
    }

    /// Returns an iterator over the available scripts.
    pub fn scripts(&'a self) -> impl Iterator<Item = Script<'a>> + 'a + Clone {
        self.0.scripts()
    }

    /// Returns the number of available features.
    pub fn num_features(&self) -> u16 {
        self.0.num_features()
    }

    /// Returns the feature at the specified index.
    pub fn feature(&'a self, index: u16) -> Option<Feature<'a>> {
        self.0.feature(index)
    }

    /// Returns an iterator over the available features.
    pub fn features(&'a self) -> impl Iterator<Item = Feature<'a>> + 'a + Clone {
        self.0.features()
    }

    /// Returns feature variation support for the layout table.
    pub fn feature_variations(&'a self) -> Option<FeatureVariations<'a>> {
        self.0.feature_variations()
    }

    /// Returns the number of available lookups.
    pub fn num_lookups(&self) -> u16 {
        self.0.num_lookups()
    }

    /// Returns the lookup at the specified index.
    pub fn lookup(&'a self, index: u16) -> Option<Lookup<'a>> {
        self.0.lookup(index)
    }

    /// Returns an iterator over the available lookups.
    pub fn lookups(&'a self) -> impl Iterator<Item = Lookup<'a>> + 'a + Clone {
        self.0.lookups()
    }
}
