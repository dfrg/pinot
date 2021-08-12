use super::shared::*;
use crate::container::prelude::*;
use crate::variation::item::Store;

/// Glyph definition table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/gdef>
#[derive(Copy, Clone)]
pub struct GlyphDef<'a> {
    data: Buffer<'a>,
    classes: u16,
    mark_classes: u16,
    mark_sets: u16,
    var_store: u32,
}

impl<'a> GlyphDef<'a> {
    /// Creates a new glyph definition table.
    pub fn new(gdef: &'a [u8]) -> Option<Self> {
        let b = Buffer::new(gdef);
        let major = b.read::<u16>(0)?;
        let minor = b.read::<u16>(2)?;
        let classes = b.read::<u16>(4)?;
        let mark_classes = b.read::<u16>(10)?;
        let mark_sets = if major > 1 || minor >= 2 {
            b.read_or_default::<u16>(12)
        } else {
            0
        };
        let var_store = if major > 1 || minor >= 3 {
            b.read_or_default::<u32>(14)
        } else {
            0
        };
        Some(Self {
            data: b,
            classes,
            mark_classes,
            mark_sets,
            var_store,
        })
    }

    /// Returns true if glyph classes are available.
    pub fn has_classes(&self) -> bool {
        self.classes != 0
    }

    /// Returns the class for the specified glyph.
    pub fn class(&self, glyph_id: u16) -> u16 {
        get_class(&self.data, self.classes as u32, glyph_id)
    }

    /// Returns the class definition table.
    pub fn classes(&self) -> Option<ClassDef<'a>> {
        if self.classes != 0 {
            Some(ClassDef::new(self.data, self.classes as u32))
        } else {
            None
        }
    }

    /// Returns true if mark classes are available.
    pub fn has_mark_classes(&self) -> bool {
        self.mark_classes != 0
    }

    /// Returns the mark class for the specified glyph.
    pub fn mark_class(&self, glyph_id: u16) -> u16 {
        get_class(&self.data, self.mark_classes as u32, glyph_id)
    }

    /// Returns the mark glyph class definition table.
    pub fn mark_classes(&self) -> Option<ClassDef<'a>> {
        if self.mark_classes != 0 {
            Some(ClassDef::new(self.data, self.mark_classes as u32))
        } else {
            None
        }
    }

    /// Returns true if mark filtering sets are available.
    pub fn has_mark_sets(&self) -> bool {
        self.mark_sets != 0
    }

    /// Returns the number of available mark filtering sets.
    pub fn num_mark_sets(&self) -> u16 {
        if self.mark_sets != 0 {
            self.data
                .read::<u16>(self.mark_sets as usize + 2)
                .unwrap_or(0)
        } else {
            0
        }
    }

    /// Returns the mark filtering set at the specified index.
    pub fn mark_set(&self, index: u16) -> Option<Coverage<'a>> {
        Some(Coverage::new(self.data, self.mark_set_offset(index)?))
    }

    /// Returns an iterator over the mark filtering sets.
    pub fn mark_sets(&self) -> impl Iterator<Item = Coverage<'a>> + '_ + Clone {
        let len = self.num_mark_sets();
        (0..len).map(move |index| {
            let offset = self.mark_set_offset(index).unwrap_or(0);
            Coverage::new(self.data, offset)
        })
    }

    /// Returns true if variations are supported.
    pub fn supports_variations(&self) -> bool {
        self.var_store != 0
    }

    /// Returns the item variation store.
    pub fn variations(&self) -> Option<Store<'a>> {
        if self.var_store != 0 {
            Some(Store::new(self.data.data(), self.var_store))
        } else {
            None
        }
    }

    pub(super) fn _mark_set_coverage(&self, set_offset: u32, glyph_id: u16) -> Option<u16> {
        if set_offset == 0 {
            return None;
        }
        // Coverage is validated by mark_set_offset() below.
        unsafe { _get_coverage_unchecked(&self.data, set_offset, glyph_id) }
    }

    pub(super) fn mark_set_offset(&self, set_index: u16) -> Option<u32> {
        if self.mark_sets == 0 {
            return None;
        }
        let set = set_index as usize;
        let b = &self.data;
        let sets_base = self.mark_sets as usize;
        let len = b.read::<u16>(sets_base + 2)? as usize;
        if set >= len {
            return None;
        }
        let offset = b.read::<u32>(sets_base + 4 + set * 4)?;
        let set_offset = sets_base as u32 + offset;
        if offset != 0 && validate_coverage(b, set_offset).is_some() {
            return Some(set_offset);
        }
        None
    }
}
