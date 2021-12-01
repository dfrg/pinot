//! Vertical header table.

use crate::parse_prelude::*;

/// Tag for the `vhea` table.
pub const VHEA: Tag = Tag::new(b"vhea");

/// Vertical header table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/vhea>
#[derive(Copy, Clone)]
pub struct Vhea<'a>(Buffer<'a>);

impl<'a> Vhea<'a> {
    /// Creates a new vertical header table from a byte slice containing the
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

    /// Returns the typographic ascender.
    pub fn ascender(&self) -> FWord {
        self.0.read(4).unwrap_or(0)
    }

    /// Returns the typographic descender.
    pub fn descender(&self) -> FWord {
        self.0.read(6).unwrap_or(0)
    }

    /// Returns the typographic line gap.
    pub fn line_gap(&self) -> FWord {
        self.0.read(8).unwrap_or(0)
    }

    /// Returns the maximum advance height.
    pub fn max_advance(&self) -> FWord {
        self.0.read(10).unwrap_or(0)
    }

    /// Returns the minimum top side-bearing.
    pub fn min_tsb(&self) -> FWord {
        self.0.read(12).unwrap_or(0)
    }

    /// Returns the minimum bottom side-bearing.
    pub fn min_bsb(&self) -> FWord {
        self.0.read(14).unwrap_or(0)
    }

    /// Returns the maximum extent: max(tsb + (y_max - y_min))
    pub fn max_extent(&self) -> FWord {
        self.0.read(16).unwrap_or(0)
    }

    /// Returns the numerator for the suggested slope of the caret.
    pub fn caret_rise(&self) -> i16 {
        self.0.read(18).unwrap_or(0)
    }

    /// Returns the denominator for the suggested slope of the caret.
    pub fn caret_run(&self) -> i16 {
        self.0.read(20).unwrap_or(0)
    }

    /// Returns the amount by which a slanted highlight on a glyph should be
    /// shifted.
    pub fn caret_offset(&self) -> i16 {
        self.0.read(22).unwrap_or(0)
    }

    /// Unused in current format. Set to 0.
    pub fn metric_data_format(&self) -> i16 {
        self.0.read(32).unwrap_or(0)
    }

    /// Returns the number of "long" metric entries in the horizonal metrics
    /// table.
    pub fn num_long_metrics(&self) -> u16 {
        self.0.read(34).unwrap_or(0)
    }
}
