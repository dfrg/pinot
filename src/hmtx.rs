//! Horizontal metrics table.

use super::parse_prelude::*;

/// Tag for the `hmtx` table.
pub const HMTX: Tag = Tag::new(b"hmtx");

/// Paired advance width and left side bearing values.
#[derive(Copy, Clone, Debug)]
pub struct HMetric {
    /// Advance width in font units.
    pub advance_width: u16,
    /// Left side bearing in font units.
    pub lsb: i16,
}

impl ReadData for HMetric {
    unsafe fn read_data_unchecked(buf: &[u8], offset: usize) -> Self {
        Self {
            advance_width: u16::read_data_unchecked(buf, offset),
            lsb: i16::read_data_unchecked(buf, offset + 2),
        }
    }
}

/// Horizontal metrics table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/hmtx>
#[derive(Copy, Clone)]
pub struct Hmtx<'a> {
    data: Buffer<'a>,
    num_glyphs: u16,
    num_hmetrics: u16,
}

impl<'a> Hmtx<'a> {
    /// Creates a new horizontal metrics table from a byte slice containing
    /// the table data, the number of glyphs and the number of horizontal
    /// metrics from the `hhea` table.
    pub fn new(data: &'a [u8], num_glyphs: u16, num_hmetrics: u16) -> Self {
        Self {
            data: Buffer::new(data),
            num_glyphs,
            num_hmetrics,
        }
    }

    /// Returns the number of glyphs.
    pub fn num_glyphs(&self) -> u16 {
        self.num_glyphs
    }

    /// Returns the slice of horizontal metrics.
    pub fn hmetrics(&self) -> Slice<'a, HMetric> {
        self.data
            .read_slice(0, self.num_hmetrics as usize)
            .unwrap_or_default()
    }

    /// Returns the remaining left side bearings.
    pub fn lsbs(&self) -> Slice<'a, FWord> {
        let offset = self.num_hmetrics as usize * 4;
        let len = (self.num_glyphs as usize).saturating_sub(self.num_hmetrics as usize);
        self.data.read_slice(offset, len).unwrap_or_default()
    }
}
