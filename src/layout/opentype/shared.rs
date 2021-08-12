//! Common support for OpenType layout.

use super::GlyphClass;
use crate::container::prelude::*;

/// Coverage table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#coverage-table>
#[derive(Copy, Clone)]
pub struct Coverage<'a> {
    data: Buffer<'a>,
    offset: u32,
}

impl<'a> Coverage<'a> {
    pub(super) fn new(data: Buffer<'a>, offset: u32) -> Self {
        Self { data, offset }
    }

    /// Returns true if the coverage table is valid.
    pub fn is_valid(&self) -> bool {
        validate_coverage(&self.data, self.offset).is_some()
    }

    /// Returns the coverage index for the specified glyph.
    pub fn get(&self, glyph_id: GlyphId) -> Option<u16> {
        get_coverage(&self.data, self.offset, glyph_id)
    }

    /// Invokes the specified closure with all (glyph, coverage) pairs in the table.
    /// Returning `false` from the closure will end enumeration early.
    pub fn indices_with(&self, f: impl FnMut(GlyphId, u16) -> bool) -> Option<bool> {
        enumerate_coverage(&self.data, self.offset as usize, f)
    }
}

/// Glyph that is represented in a coverage table.
#[derive(Copy, Clone, Debug)]
pub struct Covered {
    glyph_id: u16,
    coverage_index: u16,
}

impl Covered {
    pub(super) fn new(glyph_id: u16, coverage_index: u16) -> Self {
        Self {
            glyph_id,
            coverage_index,
        }
    }

    /// Returns the glyph identifier.
    pub fn glyph_id(self) -> GlyphId {
        self.glyph_id
    }

    /// Returns the coverage index.
    pub fn coverage_index(self) -> u16 {
        self.coverage_index
    }
}

/// Sequence of coverage tables.
#[derive(Copy, Clone)]
pub struct CoverageArray<'a> {
    data: Buffer<'a>,
    base: u32,
    offsets: Slice<'a, u16>,
}

impl<'a> CoverageArray<'a> {
    pub(super) fn new(data: Buffer<'a>, base: u32, offsets: Slice<'a, u16>) -> Self {
        Self {
            data,
            base,
            offsets,
        }
    }

    /// Returns the number of coverage tables in the sequence.
    pub fn len(&self) -> usize {
        self.offsets.len()
    }

    /// Returns true if the sequence is empty.
    pub fn is_empty(&self) -> bool {
        self.offsets.is_empty()
    }

    /// Returns the coverage table at the specified index.
    pub fn get(&self, index: usize) -> Option<Coverage<'a>> {
        Some(Coverage::new(
            self.data,
            self.base + self.offsets.get(index)? as u32,
        ))
    }

    /// Returns an iterator over the sequence of coverage tables.
    pub fn iter(&self) -> impl Iterator<Item = Coverage<'a>> + '_ + Clone {
        self.offsets
            .iter()
            .map(move |offset| Coverage::new(self.data, self.base + offset as u32))
    }
}

/// Class definition table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#class-definition-table>
#[derive(Copy, Clone)]
pub struct ClassDef<'a> {
    data: Buffer<'a>,
    offset: u32,
}

impl<'a> ClassDef<'a> {
    pub(super) fn new(data: Buffer<'a>, offset: u32) -> Self {
        Self { data, offset }
    }

    /// Returns the class for the specified glyph.
    pub fn get(&self, glyph_id: GlyphId) -> GlyphClass {
        get_class(&self.data, self.offset, glyph_id)
    }

    /// Invokes the specified closure with all (glyph, class) pairs in the table.
    /// Returning `false` from the closure will end enumeration early.
    pub fn classes_with(&self, f: impl FnMut(GlyphId, GlyphClass) -> bool) -> Option<bool> {
        enumerate_classes(&self.data, self.offset as usize, f)
    }
}

pub fn validate_coverage(b: &Buffer, coverage_offset: u32) -> Option<()> {
    if coverage_offset == 0 {
        return None;
    }
    let base = coverage_offset as usize;
    let fmt = b.read::<u16>(base)?;
    let len = b.read::<u16>(base + 2)? as usize;
    let arr = base + 4;
    match fmt {
        1 => {
            if !b.check_range(arr, len * 2) {
                None
            } else {
                Some(())
            }
        }
        2 => {
            if !b.check_range(arr, len * 6) {
                None
            } else {
                Some(())
            }
        }
        _ => None,
    }
}

pub unsafe fn _get_coverage_unchecked(
    b: &Buffer,
    coverage_offset: u32,
    glyph_id: u16,
) -> Option<u16> {
    let base = coverage_offset as usize;
    let fmt = b.read_unchecked::<u16>(base);
    let len = b.read_unchecked::<u16>(base + 2) as usize;
    let arr = base + 4;
    if fmt == 1 {
        let mut lo = 0;
        let mut hi = len;
        while lo < hi {
            use core::cmp::Ordering::*;
            let i = (lo + hi) / 2;
            let g = b.read_unchecked::<u16>(arr + i * 2);
            match glyph_id.cmp(&g) {
                Less => hi = i,
                Greater => lo = i + 1,
                Equal => return Some(i as u16),
            }
        }
    } else if fmt == 2 {
        let mut lo = 0;
        let mut hi = len;
        while lo < hi {
            let i = (lo + hi) / 2;
            let rec = arr + i * 6;
            let start = b.read_unchecked::<u16>(rec);
            if glyph_id < start {
                hi = i;
            } else if glyph_id > b.read_unchecked::<u16>(rec + 2) {
                lo = i + 1;
            } else {
                let base = b.read_unchecked::<u16>(rec + 4);
                return Some(base + glyph_id - start);
            }
        }
    }
    None
}

pub fn get_coverage(b: &Buffer, coverage_offset: u32, glyph_id: u16) -> Option<u16> {
    if coverage_offset == 0 {
        return None;
    }
    let base = coverage_offset as usize;
    let fmt = b.read::<u16>(base)?;
    let len = b.read::<u16>(base + 2)? as usize;
    let arr = base + 4;
    if fmt == 1 {
        if !b.check_range(arr, len * 2) {
            return None;
        }
        let mut lo = 0;
        let mut hi = len;
        while lo < hi {
            use core::cmp::Ordering::*;
            let i = (lo + hi) / 2;
            let g = unsafe { b.read_unchecked::<u16>(arr + i * 2) };
            match glyph_id.cmp(&g) {
                Less => hi = i,
                Greater => lo = i + 1,
                Equal => return Some(i as u16),
            }
        }
    } else if fmt == 2 {
        if !b.check_range(arr, len * 6) {
            return None;
        }
        let mut l = 0;
        let mut h = len;
        while l < h {
            let i = (l + h) / 2;
            let rec = arr + i * 6;
            let start = unsafe { b.read_unchecked::<u16>(rec) };
            if glyph_id < start {
                h = i;
            } else if glyph_id > unsafe { b.read_unchecked::<u16>(rec + 2) } {
                l = i + 1;
            } else {
                let base = unsafe { b.read_unchecked::<u16>(rec + 4) };
                return Some(base + (glyph_id - start));
            }
        }
    }
    None
}

pub fn get_class(b: &Buffer, classdef_offset: u32, glyph_id: u16) -> u16 {
    if classdef_offset == 0 {
        return 0;
    }
    let base = classdef_offset as usize;
    let fmt = b.read_or_default::<u16>(base);
    if fmt == 1 {
        let start = b.read_or_default::<u16>(base + 2);
        let len = b.read_or_default::<u16>(base + 4);
        let end = start + len - 1;
        let arr = base + 6;
        if glyph_id >= start && glyph_id <= end {
            return b.read_or_default::<u16>(arr + (glyph_id - start) as usize * 2);
        }
        return 0;
    } else if fmt == 2 {
        let len = b.read_or_default::<u16>(base + 2) as usize;
        let arr = base + 4;
        if !b.check_range(arr, len * 6) {
            return 0;
        }
        let mut l = 0;
        let mut h = len;
        while l < h {
            let i = (l + h) / 2;
            let rec = arr + i * 6;
            let start = unsafe { b.read_unchecked::<u16>(rec) };
            if glyph_id < start {
                h = i;
            } else if glyph_id > unsafe { b.read_unchecked::<u16>(rec + 2) } {
                l = i + 1;
            } else {
                return unsafe { b.read_unchecked::<u16>(rec + 4) };
            }
        }
    }
    0
}

pub fn enumerate_coverage(
    d: &Buffer,
    offset: usize,
    mut f: impl FnMut(u16, u16) -> bool,
) -> Option<bool> {
    if offset == 0 {
        return None;
    }
    let fmt = d.read_u16(offset)?;
    let len = d.read::<u16>(offset + 2)? as usize;
    let arr = offset + 4;
    if fmt == 1 {
        let glyphs = d.read_slice::<u16>(arr, len)?;
        for (i, id) in glyphs.iter().enumerate() {
            if !f(id, i as u16) {
                return Some(false);
            }
        }
        Some(true)
    } else if fmt == 2 {
        if !d.check_range(arr, len * 6) {
            return None;
        }
        let mut c = d.cursor_at(arr)?;
        for _ in 0..len {
            let start = c.read_u16()?;
            let end = c.read_u16()?;
            let start_index = c.read_u16()? as usize;
            for (i, id) in (start..=end).enumerate() {
                if !f(id, (start_index + i) as u16) {
                    return Some(true);
                }
            }
        }
        Some(true)
    } else {
        None
    }
}

pub fn enumerate_classes(
    d: &Buffer,
    offset: usize,
    mut f: impl FnMut(u16, u16) -> bool,
) -> Option<bool> {
    if offset == 0 {
        return None;
    }
    let fmt = d.read_u16(offset)?;
    if fmt == 1 {
        let start = d.read_u16(offset + 2)? as usize;
        let end = d.read_u16(offset + 4)? as usize + start;
        if end < start {
            return None;
        }
        let glyphs = d.read_slice::<u16>(offset + 6, end - start)?;
        for (i, value) in glyphs.iter().enumerate() {
            if !f((i + start) as u16, value) {
                return Some(false);
            }
        }
        Some(true)
    } else if fmt == 2 {
        let mut c = d.cursor_at(offset)?;
        c.skip(2)?;
        let count = c.read_u16()? as usize;
        for _ in 0..count {
            let start = c.read_u16()?;
            let end = c.read_u16()?;
            if end < start {
                return None;
            }
            let class = c.read_u16()?;
            for id in start..=end {
                if !f(id as u16, class) {
                    return Some(false);
                }
            }
        }
        Some(true)
    } else {
        None
    }
}
