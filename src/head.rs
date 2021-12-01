//! Font header table.

use crate::parse_prelude::*;

/// Tag for the `head` table.
pub const HEAD: Tag = Tag::new(b"head");

/// Font header table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/head>
#[derive(Copy, Clone)]
pub struct Head<'a>(Buffer<'a>);

impl<'a> Head<'a> {
    /// Creates a new font header table from a byte slice containing the table
    /// data.
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

    /// Returns a revision value. Set by font manufacturer.
    pub fn revision(&self) -> Fixed {
        self.0.read(4).unwrap_or(Fixed::ZERO)
    }

    /// Returns a checksum adjustment value.
    pub fn checksum_adjustment(&self) -> u32 {
        self.0.read(8).unwrap_or(0)
    }

    /// Returns a magic number for validation. Set to 0x5F0F3CF5.
    pub fn magic_number(&self) -> u32 {
        self.0.read(12).unwrap_or(0)
    }

    /// Returns a set of header bit flags.
    /// - 0: Baseline at y = 0
    /// - 1: Left sidebearing at x = 0
    /// - 2: Instructions may depend on point size
    /// - 3: Force ppem to integer values
    /// - 4: Instructions may alter advance width
    /// - 5-10: Unused
    /// - 11: Font data is lossless
    /// - 12: Font has been converted
    /// - 13: Optimized for ClearType
    /// - 14: Last resort font
    pub fn flags(&self) -> u16 {
        self.0.read(16).unwrap_or(0)
    }

    /// Returns the design units per em. Valid values are 16..=16384.
    pub fn units_per_em(&self) -> u16 {
        self.0.read(18).unwrap_or(0)
    }

    /// Number of seconds since 12:00 midnight that started January 1st 1904 in GMT/UTC time zone.
    pub fn created(&self) -> u64 {
        self.0.read(20).unwrap_or(0)
    }

    /// Number of seconds since 12:00 midnight that started January 1st 1904 in GMT/UTC time zone.
    pub fn modified(&self) -> u64 {
        self.0.read(28).unwrap_or(0)
    }

    /// Minimum x value for all glyph bounding boxes.
    pub fn x_min(&self) -> i16 {
        self.0.read(36).unwrap_or(0)
    }

    /// Minimum y value for all glyph bounding boxes.
    pub fn y_min(&self) -> i16 {
        self.0.read(38).unwrap_or(0)
    }

    /// Maximum x value for all glyph bounding boxes.
    pub fn x_max(&self) -> i16 {
        self.0.read(40).unwrap_or(0)
    }

    /// Maximum y value for all glyph bounding boxes.
    pub fn y_max(&self) -> i16 {
        self.0.read(42).unwrap_or(0)
    }

    /// Returns the mac style bit flags.
    /// - 0: Bold
    /// - 1: Italic
    /// - 2: Underline
    /// - 3: Outline
    /// - 4: Shadow
    /// - 5: Condensed
    /// - 6: Extended
    /// - 7-15: Reserved
    pub fn mac_style(&self) -> u16 {
        self.0.read(44).unwrap_or(0)
    }

    /// Returns the smallest readable size in pixels.
    pub fn lowest_recommended_ppem(&self) -> u16 {
        self.0.read(46).unwrap_or(0)
    }

    /// Deprecated. Returns a hint about the directionality of the glyphs.
    /// Set to 2.
    pub fn direction_hint(&self) -> u16 {
        self.0.read(48).unwrap_or(0)
    }

    /// Returns the format the the offset array in the 'loca' table.
    /// - 0: 16-bit offsets (divided by 2)
    /// - 1: 32-bit offsets
    pub fn index_to_location_format(&self) -> i16 {
        self.0.read(50).unwrap_or(0)
    }

    /// Unused. Set to 0.
    pub fn glyph_data_format(&self) -> i16 {
        self.0.read(52).unwrap_or(0)
    }
}
