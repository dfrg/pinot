//! OS/2 and Windows metrics table.

use crate::parse_prelude::*;

/// Tag for the `OS/2` table.
pub const OS2: Tag = Tag::new(b"OS/2");

/// OS/2 and Windows metrics table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/os2>
pub struct Os2<'a>(Buffer<'a>);

impl<'a> Os2<'a> {
    /// Creates a new OS/2 and Windows metrics table from a byte slice
    /// containing the table data.
    pub fn new(data: &'a [u8]) -> Self {
        Self(Buffer::new(data))
    }

    /// Returns the version number for the OS/2 table: 0x0000 to 0x0005.
    pub fn version(&self) -> u16 {
        self.0.read(0).unwrap_or(0)
    }

    /// Returns the average advance width of all non-zero width glyphs in the
    /// font.
    pub fn average_char_width(&self) -> UfWord {
        self.0.read(2).unwrap_or(0)
    }

    /// Returns the visual weight class on a scale from 1 to 1000.  
    /// Common values:
    /// - 100: Thin
    /// - 200: Extra-light (Ultra-light)
    /// - 300: Light
    /// - 400: Normal (Regular)
    /// - 500: Medium
    /// - 600: Semi-bold
    /// - 700: Bold
    /// - 800: Extra-bold (Ultra-bold)
    /// - 900: Black (Heavy)
    pub fn weight_class(&self) -> u16 {
        self.0.read(4).unwrap_or(0)
    }

    /// Returns the visual width class-- a relative change from the normal aspect
    /// ratio.
    /// - 1: Ultra-condensed
    /// - 2: Extra-condensed
    /// - 3: Condensed
    /// - 4: Semi-condensed
    /// - 5: Medium (Normal)
    /// - 6: Semi-expanded
    /// - 7: Expanded
    /// - 8: Extra-expanded
    /// - 9: Ultra-expanded
    pub fn width_class(&self) -> u16 {
        self.0.read(6).unwrap_or(0)
    }

    /// Returns the font type bit flags.  
    /// Bits:
    /// - 0-3: Usage permissions
    /// - 4-7: Reserved (set to 0)
    /// - 8: No subsetting
    /// - 9: Bitmap embedding only
    /// - 10-15: Reserved (set to 0)
    pub fn type_flags(&self) -> u16 {
        self.0.read(8).unwrap_or(0)
    }

    /// Returns the recommended horizontal size units for subscripts.
    pub fn subscript_x_size(&self) -> FWord {
        self.0.read(10).unwrap_or(0)
    }

    /// Returns the recommended vertical size units for subscripts.
    pub fn subscript_y_size(&self) -> FWord {
        self.0.read(12).unwrap_or(0)
    }

    /// Returns the recommended horizontal offset units for subscripts.
    pub fn subscript_x_offset(&self) -> FWord {
        self.0.read(14).unwrap_or(0)
    }

    /// Returns the recommended vertical offset for subscripts.
    pub fn subscript_y_offset(&self) -> FWord {
        self.0.read(16).unwrap_or(0)
    }

    /// Returns the recommended horizontal size units for subscripts.
    pub fn superscript_x_size(&self) -> FWord {
        self.0.read(18).unwrap_or(0)
    }

    /// Returns the recommended vertical size in for subscripts.
    pub fn superscript_y_size(&self) -> FWord {
        self.0.read(20).unwrap_or(0)
    }

    /// Returns the recommended horizontal offset for subscripts.
    pub fn superscript_x_offset(&self) -> FWord {
        self.0.read(22).unwrap_or(0)
    }

    /// Returns the recommended vertical offset for subscripts.
    pub fn superscript_y_offset(&self) -> FWord {
        self.0.read(24).unwrap_or(0)
    }

    /// Returns the suggested thickness for the strikeout stroke.
    pub fn strikeout_size(&self) -> FWord {
        self.0.read(26).unwrap_or(0)
    }

    /// Returns the position of the top of the strikeout stroke relative
    /// to the baseline.
    pub fn strikeout_position(&self) -> FWord {
        self.0.read(28).unwrap_or(0)
    }

    /// Returns the font family class and subclass. For values:
    /// <https://docs.microsoft.com/en-us/typography/opentype/spec/ibmfc>
    pub fn family_class(&self) -> i16 {
        self.0.read(30).unwrap_or(0)
    }

    /// Returns a 10-byte PANOSE classification number.
    /// <https://monotype.github.io/panose/>
    pub fn panose(&self) -> &[u8] {
        self.0.read_bytes(32, 10).unwrap_or(&[0; 10])
    }

    /// Returns a 128-bit value describing the Unicode blocks that are
    /// supported by the font.
    pub fn unicode_range(&self) -> [u32; 4] {
        [
            self.0.read_u32(42).unwrap_or(0),
            self.0.read_u32(46).unwrap_or(0),
            self.0.read_u32(50).unwrap_or(0),
            self.0.read_u32(54).unwrap_or(0),
        ]
    }

    /// Returns a four character font vendor identifier.
    pub fn vendor_id(&self) -> &str {
        core::str::from_utf8(
            self.0
                .read_bytes(58, 4)
                .unwrap_or(&[b'n', b'o', b'n', b'e']),
        )
        .unwrap_or("none")
    }

    /// Returns the font selection bit flags.  
    /// Bits:
    /// - 0: Italic
    /// - 1: Underscore
    /// - 2: Negative
    /// - 3: Outlined
    /// - 4: Strikeout
    /// - 5: Bold
    /// - 6: Regular
    /// - 7: Use typographic metrics
    /// - 8: WWS (Weight/Width/Slope names)
    /// - 9: Oblique
    /// - 10-15: Reserved (set to 0)
    pub fn selection_flags(&self) -> u16 {
        self.0.read(62).unwrap_or(0)
    }

    /// Returns the minimum Unicode index supported by the font.
    pub fn first_char_index(&self) -> u16 {
        self.0.read_u16(64).unwrap_or(0)
    }

    /// Returns the maximum Unicode index supported by the font.
    pub fn last_char_index(&self) -> u16 {
        self.0.read_u16(66).unwrap_or(0)
    }

    /// Returns the typographic ascender.
    pub fn typographic_ascender(&self) -> FWord {
        self.0.read(68).unwrap_or(0)
    }

    /// Returns the typographic descender.
    pub fn typographic_descender(&self) -> FWord {
        self.0.read(70).unwrap_or(0)
    }

    /// Returns the typographic line gap.
    pub fn typographic_line_gap(&self) -> FWord {
        self.0.read(72).unwrap_or(0)
    }

    /// Returns a Windows specific value that defines the upper extent of
    /// the clipping region.
    pub fn win_ascent(&self) -> UfWord {
        self.0.read(74).unwrap_or(0)
    }

    /// Returns a Windows specific value that defines the lower extent of
    /// the clipping region.
    pub fn win_descent(&self) -> UfWord {
        self.0.read(76).unwrap_or(0)
    }

    /// Returns Windows specific code page ranges supported by the font.
    /// (table version >= 1)
    pub fn code_page_range(&self) -> Option<[u32; 2]> {
        if self.version() < 1 {
            return None;
        }
        Some([
            self.0.read_u32(78).unwrap_or(0),
            self.0.read_u32(82).unwrap_or(0),
        ])
    }

    /// Returns the approximate distance above the baseline for non-descending
    /// lowercase letters (table version >= 2)
    pub fn x_height(&self) -> Option<FWord> {
        if self.version() < 2 {
            None
        } else {
            self.0.read(86)
        }
    }

    /// Returns the approximate distance above the baseline for uppercase letters.
    /// (table version >= 2)
    pub fn cap_height(&self) -> Option<FWord> {
        if self.version() < 2 {
            None
        } else {
            self.0.read(88)
        }
    }

    /// Returns a Unicode codepoint for the default character to use if
    /// a requested character is not supported by the font.
    /// (table version >= 2)
    pub fn default_char(&self) -> Option<u16> {
        if self.version() < 2 {
            None
        } else {
            self.0.read(90)
        }
    }

    /// Returns a Unicode codepoint for the default character used to separate
    /// words and justify text. (table version >= 2)
    pub fn break_char(&self) -> Option<u16> {
        if self.version() < 2 {
            None
        } else {
            self.0.read(92)
        }
    }

    /// Returns the maximum length of a target glyph context for any feature in
    /// the font. (table version >= 2)
    pub fn max_context(&self) -> Option<u16> {
        if self.version() < 2 {
            None
        } else {
            self.0.read(94)
        }
    }

    /// Returns the lower value of the size range for which this font has been
    /// designed. The units are TWIPS (1/20 points). (table version >= 5)
    pub fn lower_optical_point_size(&self) -> Option<u16> {
        if self.version() < 5 {
            None
        } else {
            self.0.read(96)
        }
    }

    /// Returns the upper value of the size range for which this font has been
    /// designed. The units are TWIPS (1/20 points). (table version >= 5)
    pub fn upper_optical_point_size(&self) -> Option<u16> {
        if self.version() < 5 {
            None
        } else {
            self.0.read(98)
        }
    }
}
