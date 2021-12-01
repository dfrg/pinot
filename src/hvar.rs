//! Horizontal metrics variation table.

use super::parse_prelude::*;
use super::var::item::{DeltaSetIndexMap, ItemVariationStore};

/// Tag for the `HVAR` table.
pub const HVAR: Tag = Tag::new(b"HVAR");

/// Horizontal metrics variation table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/hvar>
#[derive(Copy, Clone)]
pub struct Hvar<'a>(Buffer<'a>);

impl<'a> Hvar<'a> {
    /// Creates a new horizontal metrics variation table from a byte slice
    /// containing the table data.
    pub fn new(data: &'a [u8]) -> Self {
        Self(Buffer::new(data))
    }

    /// Returns the major version.
    pub fn major_version(&self) -> u16 {
        self.0.read(0).unwrap_or(0)
    }

    /// Returns the minor version.
    pub fn minor_version(&self) -> u16 {
        self.0.read(2).unwrap_or(0)
    }

    /// Returns the item variation store.
    pub fn ivs(&self) -> Option<ItemVariationStore<'a>> {
        let offset = self.0.read_offset32(4, 0)?;
        ItemVariationStore::new(self.0, offset)
    }

    /// Returns the delta set index mapping for advance widths.
    pub fn advance_mapping(&self) -> Option<DeltaSetIndexMap<'a>> {
        let offset = self.0.read_offset32(8, 0)?;
        DeltaSetIndexMap::new(self.0, offset)
    }

    /// Returns the delta set index mapping for left side bearings.
    pub fn lsb_mapping(&self) -> Option<DeltaSetIndexMap<'a>> {
        let offset = self.0.read_offset32(12, 0)?;
        DeltaSetIndexMap::new(self.0, offset)
    }

    /// Returns the delta set index mapping for right side bearings.
    pub fn rsb_mapping(&self) -> Option<DeltaSetIndexMap<'a>> {
        let offset = self.0.read_offset32(16, 0)?;
        DeltaSetIndexMap::new(self.0, offset)
    }
}
