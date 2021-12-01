//! Vertical origin table.

use super::parse_prelude::*;

/// Tag for the `VORG` table.
pub const VORG: Tag = Tag::new(b"VORG");

/// Glyph identifier and Y coordinate of the vertical origin.
#[derive(Copy, Clone, Debug)]
pub struct VyMetric {
    pub gid: GlyphId,
    pub y: FWord,
}

impl ReadData for VyMetric {
    unsafe fn read_data_unchecked(buf: &[u8], offset: usize) -> Self {
        Self {
            gid: u16::read_data_unchecked(buf, offset),
            y: i16::read_data_unchecked(buf, offset + 2),
        }
    }
}

/// Vertical origin table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/vorg>
#[derive(Copy, Clone)]
pub struct Vorg<'a>(Buffer<'a>);

impl<'a> Vorg<'a> {
    /// Creates a new vertical origin table from a byte slice containing the
    /// table data.
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

    /// Returns the Y coordinate of a glyph’s vertical origin, in font units, to be used
    /// if no entry is present.
    pub fn default_vymetric(&self) -> FWord {
        self.0.read(4).unwrap_or(0)
    }

    /// Returns a list of Y coordinates of a glyph's vertical origin, sorted by
    /// glyph identifier.
    pub fn vymetrics(&self) -> Slice<'a, VyMetric> {
        self.0.read_slice16(8).unwrap_or_default()
    }
}
