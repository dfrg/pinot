//! Mathematical typesetting table.

use super::otl::Coverage;
use super::parse_prelude::*;

/// Tag for the `math` table.
pub const MATH: Tag = Tag::new(b"MATH");

/// Mathematical typesetting table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/math>
///
/// The math constants and math glyph information subtables are not (yet)
/// implemented.
#[derive(Copy, Clone)]
pub struct Math<'a>(Buffer<'a>);

impl<'a> Math<'a> {
    /// Creates a new math table from a byte slice containing the table data.
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

    /// Returns the MathVariants subtable.
    pub fn variants(&self) -> Option<MathVariants> {
        let offset = self.0.read_offset16(8, 0)?;
        Some(MathVariants { math: self, offset })
    }
}

/// Mathematical variants subtable.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/math#mathvariants-table>
#[derive(Copy, Clone)]
pub struct MathVariants<'a> {
    math: &'a Math<'a>,
    offset: u32,
}

impl<'a> MathVariants<'a> {
    /// Returns the minimum overlap of connecting glyphs during glyph
    /// construction.
    pub fn min_connector_overlap(&self) -> UfWord {
        self.math.0.read(self.offset as usize).unwrap_or(0)
    }

    /// Returns the number of glyphs for which information is provided for
    /// vertically growing variants.
    pub fn vert_glyph_count(&self) -> u16 {
        self.math.0.read(self.offset as usize + 6).unwrap_or(0)
    }

    /// Returns the number of glyphs for which information is provided for
    /// horizontally growing variants.
    pub fn horiz_glyph_count(&self) -> u16 {
        self.math.0.read(self.offset as usize + 8).unwrap_or(0)
    }

    /// Returns the coverage table associated with vertically growing glyphs.
    pub fn vert_glyph_coverage(&self) -> Option<Coverage> {
        let offset = self
            .math
            .0
            .read_offset16(self.offset as usize + 2, self.offset)?;
        Some(Coverage::new(self.math.0, offset))
    }

    /// Returns the coverage table associated with horizontally growing glyphs.
    pub fn horiz_glyph_coverage(&self) -> Option<Coverage> {
        let offset = self
            .math
            .0
            .read_offset16(self.offset as usize + 4, self.offset)?;
        Some(Coverage::new(self.math.0, offset))
    }

    /// Returns information about how to a construct vertically growing glyph,
    /// based on its coverage index.
    pub fn vert_glyph_construction(&self, coverage_index: u16) -> Option<MathGlyphConstruction> {
        let offset = self.offset as usize + 10 + 2 * coverage_index as usize;
        let offset = self.math.0.read_offset16(offset, self.offset)?;
        Some(MathGlyphConstruction {
            math: self.math,
            offset: offset,
        })
    }

    /// Returns information about how to a construct horizontally growing glyph,
    /// based on its coverage index.
    pub fn horiz_glyph_construction(&self, coverage_index: u16) -> Option<MathGlyphConstruction> {
        let offset = self.offset as usize
            + 10
            + 2 * self.vert_glyph_count() as usize
            + 2 * coverage_index as usize;
        let offset = self.math.0.read_offset16(offset, self.offset)?;
        Some(MathGlyphConstruction {
            math: self.math,
            offset: offset,
        })
    }
}

/// Mathematical glyph construction subtable.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/math#mathvariants-table>
///
/// The "glyph assembly" subtable is not (yet) implemented.
#[derive(Copy, Clone)]
pub struct MathGlyphConstruction<'a> {
    math: &'a Math<'a>,
    offset: u32,
}

impl<'a> MathGlyphConstruction<'a> {
    /// Returns the number of growing variants for this glyph.
    pub fn variant_count(&self) -> u16 {
        self.math.0.read(self.offset as usize + 2).unwrap_or(0)
    }

    /// Return the growing variants associated with this glyph.
    pub fn variants(&self) -> Option<Slice<'a, MathGlyphVariantRecord>> {
        self.math
            .0
            .read_slice(self.offset as usize + 4, self.variant_count() as usize)
    }
}

/// Information about a math glyph variant.
#[derive(Copy, Clone, Debug)]
pub struct MathGlyphVariantRecord {
    /// The variant glyph
    pub variant_glyph: GlyphId,
    /// The advance width/height of the variant, in the direction of this
    /// record's associated table.
    pub advance_measurement: UfWord,
}

impl ReadData for MathGlyphVariantRecord {
    unsafe fn read_data_unchecked(buf: &[u8], offset: usize) -> Self {
        Self {
            variant_glyph: GlyphId::read_data_unchecked(buf, offset),
            advance_measurement: UfWord::read_data_unchecked(buf, offset + 2),
        }
    }
}
