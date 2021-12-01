//! Character mapping table.

use crate::parse_prelude::*;
use core::cmp::Ordering;

/// Tag for the `cmap` table.
pub const CMAP: Tag = Tag::new(b"cmap");

/// Character to glyph index mapping table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/cmap>
#[derive(Copy, Clone)]
pub struct Cmap<'a>(Buffer<'a>);

impl<'a> Cmap<'a> {
    /// Creates a new character to glyph index mapping table from a byte slice
    /// containing the table data.
    pub fn new(data: &'a [u8]) -> Self {
        Self(Buffer::new(data))
    }

    /// Returns the version.
    pub fn version(&self) -> u16 {
        self.0.read(0).unwrap_or(0)
    }

    /// Returns the array of encodings.
    pub fn encodings(&self) -> Slice<'a, EncodingRecord> {
        let len = self.0.read_u16(2).unwrap_or_default() as usize;
        self.0.read_slice(4, len).unwrap_or_default()
    }

    /// Returns an iterator over the subtables.
    pub fn subtables(self) -> impl Iterator<Item = Subtable<'a>> + 'a + Clone {
        self.encodings().iter().map(move |encoding| Subtable {
            cmap: self,
            encoding,
        })
    }

    /// Maps a codepoint to a glyph identifier.
    pub fn map(&self, codepoint: u32) -> Option<GlyphId> {
        self.subtables()
            .filter_map(|subtable| subtable.map(codepoint))
            .next()
    }

    /// Maps a codepoint with variation selector to a glyph identifier.
    pub fn map_variant(&self, codepoint: u32, variation_selector: u32) -> Option<MapVariant> {
        self.subtables()
            .filter_map(|subtable| subtable.map_variant(codepoint, variation_selector))
            .next()
    }
}

/// Encoding and offset to subtable.
#[derive(Copy, Clone, Debug)]
pub struct EncodingRecord {
    /// Platform identifier.
    pub platform_id: u16,
    /// Platform specific encoding identifier.
    pub encoding_id: u16,
    /// Offset from beginning of table to the subtable for the encoding.
    pub offset: u32,
}

impl ReadData for EncodingRecord {
    unsafe fn read_data_unchecked(buf: &[u8], offset: usize) -> Self {
        Self {
            platform_id: u16::read_data_unchecked(buf, offset),
            encoding_id: u16::read_data_unchecked(buf, offset + 2),
            offset: u32::read_data_unchecked(buf, offset + 4),
        }
    }
}

/// Character to glyph index mapping subtable.
#[derive(Copy, Clone)]
pub struct Subtable<'a> {
    /// Parent table.
    pub cmap: Cmap<'a>,
    /// Encoding record.
    pub encoding: EncodingRecord,
}

impl<'a> Subtable<'a> {
    /// Returns the subtable format.
    pub fn format(&self) -> u16 {
        self.cmap
            .0
            .read_u16(self.encoding.offset as usize)
            .unwrap_or_default()
    }

    /// Maps a codepoint to a glyph identifier.
    pub fn map(&self, codepoint: u32) -> Option<GlyphId> {
        map(
            self.cmap.0.data(),
            self.encoding.offset,
            self.format(),
            codepoint,
        )
    }

    /// Maps a codepoint with variation selector to a glyph identifier.
    pub fn map_variant(&self, codepoint: u32, variation_selector: u32) -> Option<MapVariant> {
        if self.format() == 14 {
            map_variant(
                self.cmap.0.data(),
                self.encoding.offset,
                codepoint,
                variation_selector,
            )
        } else {
            None
        }
    }
}

/// Maps a codepoint to a glyph identifer using the subtable of the given
/// format at the specified offset in data.
///
/// Supports the following formats:
/// - Format 4: <https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-4-segment-mapping-to-delta-values>
/// - Format 12: <https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-12-segmented-coverage>
/// - Format 13: <https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-13-many-to-one-range-mappings>
pub fn map(data: &[u8], offset: u32, format: u16, codepoint: u32) -> Option<GlyphId> {
    match format {
        4 => map_format4(data, offset, codepoint),
        12 => map_format12(data, offset, codepoint),
        13 => map_format13(data, offset, codepoint),
        _ => None,
    }
}

fn map_format4(data: &[u8], offset: u32, codepoint: u32) -> Option<GlyphId> {
    if codepoint >= 65535 {
        return None;
    }
    let codepoint = codepoint as u16;
    let b = Buffer::with_offset(data, offset as usize)?;
    let segcount_x2 = b.read_u16(6)? as usize;
    let segcount = segcount_x2 / 2;
    b.ensure_range(0, 16 + segcount_x2 * 4)?;
    let end_codes_offset = 14;
    let start_codes_offset = end_codes_offset + segcount_x2 + 2;
    let mut lo = 0;
    let mut hi = segcount;
    while lo < hi {
        let i = (lo + hi) / 2;
        let i2 = i * 2;
        let start = unsafe { b.read_unchecked::<u16>(start_codes_offset + i2) };
        if codepoint < start {
            hi = i;
        } else if codepoint > unsafe { b.read_unchecked::<u16>(end_codes_offset + i2) } {
            lo = i + 1;
        } else {
            let deltas_offset = start_codes_offset + segcount_x2;
            let ranges_offset = deltas_offset + segcount_x2;
            let mut range_base = ranges_offset + i2;
            let range = unsafe { b.read_unchecked::<u16>(range_base) as usize };
            let delta = unsafe { b.read_unchecked::<i16>(deltas_offset + i2) as i32 };
            if range == 0 {
                return Some((codepoint as i32 + delta) as u16);
            }
            range_base += range;
            let diff = (codepoint - start) as usize * 2;
            let id = b.read::<u16>(range_base + diff).unwrap_or(0);
            return if id != 0 {
                Some((id as i32 + delta as i32) as u16)
            } else {
                Some(0)
            };
        }
    }
    None
}

fn map_format12(data: &[u8], offset: u32, codepoint: u32) -> Option<GlyphId> {
    let (start, delta) = map_format12_13(data, offset, codepoint)?;
    Some((codepoint.wrapping_sub(start).wrapping_add(delta)) as u16)
}

fn map_format13(data: &[u8], offset: u32, codepoint: u32) -> Option<GlyphId> {
    let (_, glyph_id) = map_format12_13(data, offset, codepoint)?;
    Some(glyph_id as u16)
}

/// Common code for formats 12 and 13.
fn map_format12_13(data: &[u8], offset: u32, codepoint: u32) -> Option<(u32, u32)> {
    let b = Buffer::with_offset(data, offset as usize)?;
    let base = 16;
    let len = b.read_u32(base - 4)? as usize;
    b.ensure_range(base, len * 12)?;
    let mut lo = 0;
    let mut hi = len;
    while lo < hi {
        let i = (lo + hi) / 2;
        let rec = base + i * 12;
        let start = unsafe { b.read_unchecked::<u32>(rec) };
        if codepoint < start {
            hi = i;
        } else if codepoint > unsafe { b.read_unchecked::<u32>(rec + 4) } {
            lo = i + 1;
        } else {
            return Some((start, unsafe { b.read_unchecked::<u32>(rec + 8) }));
        }
    }
    None
}

/// Result of the mapping a codepoint with a variation selector.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum MapVariant {
    /// Use the default glyph mapping.
    UseDefault,
    /// Use the specified variant.
    Variant(GlyphId),
}

/// Maps a codepoint with variation selector to a glyph identifer using the
/// format 14 subtable at the specified offset in data.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-14-unicode-variation-sequences>
pub fn map_variant(
    data: &[u8],
    offset: u32,
    codepoint: u32,
    variation_selector: u32,
) -> Option<MapVariant> {
    let b = Buffer::with_offset(data, offset as usize)?;
    let len = b.read_u32(6)? as usize;
    let base = 10;
    let mut lo = 0;
    let mut hi = len;
    let mut default_uvs_offset = 0;
    let mut non_default_uvs_offset = 0;
    while lo < hi {
        let i = (lo + hi) / 2;
        let rec = base + i * 11;
        let vs = b.read_u24(rec)?;
        match variation_selector.cmp(&vs) {
            Ordering::Less => hi = i,
            Ordering::Greater => lo = i + 1,
            Ordering::Equal => {
                default_uvs_offset = b.read_u32(rec + 3)? as usize;
                non_default_uvs_offset = b.read_u32(rec + 7)? as usize;
                break;
            }
        }
    }
    if default_uvs_offset != 0 {
        let base = default_uvs_offset;
        let len = b.read_u32(base)? as usize;
        let mut lo = 0;
        let mut hi = len;
        while lo < hi {
            let i = (lo + hi) / 2;
            let rec = base + 4 + i * 4;
            let start = b.read_u24(rec)?;
            if codepoint < start {
                hi = i;
            } else if codepoint > (start + b.read_u8(rec + 3)? as u32) {
                lo = i + 1;
            } else {
                // Fallback to standard mapping.
                return Some(MapVariant::UseDefault);
            }
        }
    }
    if non_default_uvs_offset != 0 {
        let base = non_default_uvs_offset;
        let len = b.read_u32(base)? as usize;
        let mut lo = 0;
        let mut hi = len;
        while lo < hi {
            let i = (lo + hi) / 2;
            let rec = base + 4 + i * 5;
            let value = b.read_u24(rec)?;
            match codepoint.cmp(&value) {
                Ordering::Less => hi = i,
                Ordering::Greater => lo = i + 1,
                Ordering::Equal => {
                    return Some(MapVariant::Variant(b.read_u16(rec + 3)?));
                }
            }
        }
    }
    None
}
