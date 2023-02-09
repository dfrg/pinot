//! Vertical metrics table.

use super::parse_prelude::*;

/// Tag for the `vmtx` table.
pub const VMTX: Tag = Tag::new(b"vmtx");

/// Paired advance height and top side bearing values.
#[derive(Copy, Clone, Debug)]
pub struct VMetric {
    /// Advance height in font units.
    pub advance_height: u16,
    /// Top side bearing in font units.
    pub tsb: i16,
}

impl ReadData for VMetric {
    unsafe fn read_data_unchecked(buf: &[u8], offset: usize) -> Self {
        Self {
            advance_height: u16::read_data_unchecked(buf, offset),
            tsb: i16::read_data_unchecked(buf, offset + 2),
        }
    }
}

/// Vertical metrics table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/vmtx>
#[derive(Copy, Clone)]
pub struct Vmtx<'a> {
    data: Buffer<'a>,
    num_glyphs: u16,
    num_vmetrics: u16,
}

impl<'a> Vmtx<'a> {
    /// Creates a new horizontal metrics table from a byte slice containing the
    /// table data, the number of glyphs and the number of vertical metrics
    /// from the `vhea` table.
    pub fn new(data: &'a [u8], num_glyphs: u16, num_vmetrics: u16) -> Self {
        Self {
            data: Buffer::new(data),
            num_glyphs,
            num_vmetrics,
        }
    }

    /// Returns the number of glyphs.
    pub fn num_glyphs(&self) -> u16 {
        self.num_glyphs
    }

    /// Returns the slice of vertical metrics.
    pub fn vmetrics(&self) -> Slice<'a, VMetric> {
        self.data
            .read_slice(0, self.num_vmetrics as usize)
            .unwrap_or_default()
    }

    /// Returns the remaining top side bearings.
    pub fn tsbs(&self) -> Slice<'a, FWord> {
        let offset = self.num_vmetrics as usize * 4;
        let len = (self.num_glyphs as usize).saturating_sub(self.num_vmetrics as usize);
        self.data.read_slice(offset, len).unwrap_or_default()
    }
}
