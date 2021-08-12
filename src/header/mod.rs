//! Header tables.

pub mod value;

use crate::container::prelude::*;
use core::fmt;

/// Font header table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/head>
#[derive(Copy, Clone)]
pub struct Header<'a>(Buffer<'a>);

impl<'a> Header<'a> {
    /// Creates a new header from the `head` table.
    pub fn new(head: &'a [u8]) -> Self {
        Self(Buffer::new(head))
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

    /// Returns the value representation of the header.
    pub fn to_value(&self) -> value::Header {
        value::Header {
            major_version: self.major_version(),
            minor_version: self.minor_version(),
            revision: self.revision(),
            checksum_adjustment: self.checksum_adjustment(),
            magic_number: self.magic_number(),
            flags: self.flags(),
            units_per_em: self.units_per_em(),
            created: self.created(),
            modified: self.modified(),
            x_min: self.x_min(),
            y_min: self.y_min(),
            x_max: self.x_max(),
            y_max: self.y_max(),
            mac_style: self.mac_style(),
            lowest_recommended_ppem: self.lowest_recommended_ppem(),
            direction_hint: self.direction_hint(),
            index_to_location_format: self.index_to_location_format(),
            glyph_data_format: self.glyph_data_format(),
        }
    }
}

impl core::fmt::Debug for Header<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.to_value())
    }
}

/// Horizontal header table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/hhea>
#[derive(Copy, Clone)]
pub struct HoriHeader<'a>(Buffer<'a>);

impl<'a> HoriHeader<'a> {
    /// Creates a new header from the `hhea` table.
    pub fn new(hhea: &'a [u8]) -> Self {
        Self(Buffer::new(hhea))
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

    /// Returns the maximum advance width.
    pub fn max_advance(&self) -> UfWord {
        self.0.read(10).unwrap_or(0)
    }

    /// Returns the minimum left side-bearing.
    pub fn min_lsb(&self) -> FWord {
        self.0.read(12).unwrap_or(0)
    }

    /// Returns the minimum right side-bearing.
    pub fn min_rsb(&self) -> FWord {
        self.0.read(14).unwrap_or(0)
    }

    /// Returns the maximum extent: max(lsb + (x_max - x_min))
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

    /// Returns the value representation of the header.
    pub fn to_value(&self) -> value::HoriHeader {
        value::HoriHeader {
            major_version: self.major_version(),
            minor_version: self.minor_version(),
            ascender: self.ascender(),
            descender: self.descender(),
            line_gap: self.line_gap(),
            max_advance: self.max_advance(),
            min_lsb: self.min_lsb(),
            min_rsb: self.min_rsb(),
            max_extent: self.max_extent(),
            caret_rise: self.caret_rise(),
            caret_run: self.caret_run(),
            caret_offset: self.caret_offset(),
            metric_data_format: self.metric_data_format(),
            num_long_metrics: self.num_long_metrics(),
        }
    }
}

impl core::fmt::Debug for HoriHeader<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.to_value())
    }
}

/// Vertical header table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/vhea>
#[derive(Copy, Clone)]
pub struct VertHeader<'a>(Buffer<'a>);

impl<'a> VertHeader<'a> {
    /// Creates a new header from the `vhea` table.
    pub fn new(vhea: &'a [u8]) -> Self {
        Self(Buffer::new(vhea))
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

    /// Returns the value representation of the header.
    pub fn to_value(&self) -> value::VertHeader {
        value::VertHeader {
            major_version: self.major_version(),
            minor_version: self.minor_version(),
            ascender: self.ascender(),
            descender: self.descender(),
            line_gap: self.line_gap(),
            max_advance: self.max_advance(),
            min_tsb: self.min_tsb(),
            min_bsb: self.min_bsb(),
            max_extent: self.max_extent(),
            caret_rise: self.caret_rise(),
            caret_run: self.caret_run(),
            caret_offset: self.caret_offset(),
            metric_data_format: self.metric_data_format(),
            num_long_metrics: self.num_long_metrics(),
        }
    }
}

impl core::fmt::Debug for VertHeader<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.to_value())
    }
}

/// OS/2 and Windows table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/os2>
pub struct Windows<'a>(Buffer<'a>);

impl<'a> Windows<'a> {
    /// Creates a new header from the `OS/2` table.
    pub fn new(os2: &'a [u8]) -> Self {
        Self(Buffer::new(os2))
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

    /// Returns the recommended horizontal size for subscripts.
    pub fn subscript_x_size(&self) -> FWord {
        self.0.read(10).unwrap_or(0)
    }

    /// Returns the recommended vertical size for subscripts.
    pub fn subscript_y_size(&self) -> FWord {
        self.0.read(12).unwrap_or(0)
    }

    /// Returns the recommended horizontal offset for subscripts.
    pub fn subscript_x_offset(&self) -> FWord {
        self.0.read(14).unwrap_or(0)
    }

    /// Returns the recommended vertical offset for subscripts.
    pub fn subscript_y_offset(&self) -> FWord {
        self.0.read(16).unwrap_or(0)
    }

    /// Returns the recommended horizontal size for superscripts.
    pub fn superscript_x_size(&self) -> FWord {
        self.0.read(18).unwrap_or(0)
    }

    /// Returns the recommended vertical size for superscripts.
    pub fn superscript_y_size(&self) -> FWord {
        self.0.read(20).unwrap_or(0)
    }

    /// Returns the recommended horizontal offset for superscripts.
    pub fn superscript_x_offset(&self) -> FWord {
        self.0.read(22).unwrap_or(0)
    }

    /// Returns the recommended vertical offset for superscripts.
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

    /// Returns the value representation of the header.
    pub fn to_value(&self) -> value::Windows {
        let mut panose = [0; 10];
        panose.copy_from_slice(self.panose());
        let mut vendor_id = [0; 4];
        vendor_id.copy_from_slice(self.vendor_id().as_bytes());
        value::Windows {
            version: self.version(),
            average_char_width: self.average_char_width(),
            weight_class: self.weight_class(),
            width_class: self.width_class(),
            type_flags: self.type_flags(),
            subscript_x_size: self.subscript_x_size(),
            subscript_y_size: self.subscript_y_size(),
            subscript_x_offset: self.subscript_x_offset(),
            subscript_y_offset: self.subscript_y_offset(),
            superscript_x_size: self.superscript_x_size(),
            superscript_y_size: self.superscript_y_size(),
            superscript_x_offset: self.superscript_x_offset(),
            superscript_y_offset: self.superscript_y_offset(),
            strikeout_size: self.strikeout_size(),
            strikeout_position: self.strikeout_position(),
            family_class: self.family_class(),
            panose,
            unicode_range: self.unicode_range(),
            vendor_id,
            selection_flags: self.selection_flags(),
            first_char_index: self.first_char_index(),
            last_char_index: self.last_char_index(),
            typographic_ascender: self.typographic_ascender(),
            typographic_descender: self.typographic_descender(),
            typographic_line_gap: self.typographic_line_gap(),
            win_ascent: self.win_ascent(),
            win_descent: self.win_descent(),
            code_page_range: self.code_page_range(),
            x_height: self.x_height(),
            cap_height: self.cap_height(),
            default_char: self.default_char(),
            break_char: self.break_char(),
            max_context: self.max_context(),
            lower_optical_point_size: self.lower_optical_point_size(),
            upper_optical_point_size: self.upper_optical_point_size(),
        }
    }
}

impl core::fmt::Debug for Windows<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.to_value())
    }
}

/// Maximum profile table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/maxp>
#[derive(Copy, Clone)]
pub struct MaxProfile<'a>(Buffer<'a>);

impl<'a> MaxProfile<'a> {
    /// Creates a new header from the `maxp` table.
    pub fn new(maxp: &'a [u8]) -> Self {
        Self(Buffer::new(maxp))
    }

    /// Returns the version of the table.
    /// - 0x00005000: Version 0.5 - only `num_glyphs` will return a meaningful value.
    /// - 0x00010000: Version 1.0
    pub fn version(&self) -> Fixed {
        self.0.read(0).unwrap_or(Fixed::ZERO)
    }

    /// Returns the number of glyphs in the font.
    pub fn num_glyphs(&self) -> u16 {
        self.0.read(4).unwrap_or(0)
    }

    /// Returns the maximum points in a simple glyph.
    pub fn max_points(&self) -> u16 {
        self.0.read(6).unwrap_or(0)
    }

    /// Returns the maximum contours in a simple glyph.
    pub fn max_contours(&self) -> u16 {
        self.0.read(8).unwrap_or(0)
    }

    /// Returns the maximum points in a composite glyph.
    pub fn max_composite_points(&self) -> u16 {
        self.0.read(10).unwrap_or(0)
    }

    /// Returns the maximum contours in a composite glyph.
    pub fn max_composite_contours(&self) -> u16 {
        self.0.read(12).unwrap_or(0)
    }

    /// Returns 2 if instructions require a 'twilight zone' or 1 otherwise.
    pub fn max_zones(&self) -> u16 {
        self.0.read(14).unwrap_or(0)
    }

    /// Returns the maximum twilight points used in zone 0.
    pub fn max_twilight_points(&self) -> u16 {
        self.0.read(16).unwrap_or(0)
    }

    /// Returns the maximum storage area locations.
    pub fn max_storage(&self) -> u16 {
        self.0.read(18).unwrap_or(0)
    }

    /// Returns the maximum function definitions.
    pub fn max_function_defs(&self) -> u16 {
        self.0.read(20).unwrap_or(0)
    }

    /// Returns the maximum instruction definitions.
    pub fn max_instruction_defs(&self) -> u16 {
        self.0.read(22).unwrap_or(0)
    }

    /// Returns the maximum stack depth across all programs in the font.
    pub fn max_stack_depth(&self) -> u16 {
        self.0.read(24).unwrap_or(0)
    }

    /// Returns the maximum size of glyph instructions.
    pub fn max_instructions_size(&self) -> u16 {
        self.0.read(26).unwrap_or(0)
    }

    /// Returns the maximum number of components for a single composite glyph.
    pub fn max_component_elements(&self) -> u16 {
        self.0.read(28).unwrap_or(0)
    }

    /// Returns the maximum nesting level for any composite glyph.
    pub fn max_component_depth(&self) -> u16 {
        self.0.read(30).unwrap_or(0)
    }

    /// Returns the value representation of the header.
    pub fn to_value(&self) -> value::MaxProfile {
        value::MaxProfile {
            version: self.version(),
            num_glyphs: self.num_glyphs(),
            max_points: self.max_points(),
            max_contours: self.max_contours(),
            max_composite_points: self.max_composite_points(),
            max_composite_contours: self.max_composite_contours(),
            max_zones: self.max_zones(),
            max_twilight_points: self.max_twilight_points(),
            max_storage: self.max_storage(),
            max_function_defs: self.max_function_defs(),
            max_instruction_defs: self.max_instruction_defs(),
            max_stack_depth: self.max_stack_depth(),
            max_instructions_size: self.max_instructions_size(),
            max_component_elements: self.max_component_elements(),
            max_component_depth: self.max_component_depth(),
        }
    }
}

impl core::fmt::Debug for MaxProfile<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.to_value())
    }
}

/// PostScript table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/post>
#[derive(Copy, Clone)]
pub struct PostScript<'a>(Buffer<'a>);

impl<'a> PostScript<'a> {
    /// Creates a new header from the `post` table.
    pub fn new(post: &'a [u8]) -> Self {
        Self(Buffer::new(post))
    }

    /// Returns the version of the PostScript table.
    pub fn version(&self) -> Fixed {
        self.0.read(0).unwrap_or(Fixed::ZERO)
    }

    /// Returns the italic angle in counter-clockwise degrees from the vertical.
    pub fn italic_angle(&self) -> Fixed {
        self.0.read(4).unwrap_or(Fixed::ZERO)
    }

    /// Returns the suggested offset of the top of the underline stroke from
    /// the baseline.
    pub fn underline_position(&self) -> FWord {
        self.0.read(8).unwrap_or(0)
    }

    /// Returns the suggested thickness for the underline stroke.
    pub fn underline_thickness(&self) -> FWord {
        self.0.read(10).unwrap_or(0)
    }

    /// Returns true if the font is not proportionally spaced (i.e. monospaced).
    pub fn is_fixed_pitch(&self) -> bool {
        self.0.read_u32(12).unwrap_or(0) != 0
    }

    /// Returns true if the table can provide glyph names. Only versions 1.0
    /// (0x00010000) and 2.0 (0x00020000).
    pub fn has_glyph_names(&self) -> bool {
        let v = self.version().0;
        v == 0x10000 || v == 0x20000
    }

    /// Returns the name of the specified glyph if available.
    pub fn glyph_name(&self, glyph_id: GlyphId) -> Option<&'a str> {
        if !self.has_glyph_names() {
            return None;
        }
        let v = self.version().0;
        if v == 0x10000 {
            if glyph_id >= 258 {
                return None;
            }
            return Some(DEFAULT_GLYPH_NAMES[glyph_id as usize]);
        } else if v == 0x20000 {
            let b = &self.0;
            let count = b.read_u16(32)?;
            if glyph_id >= count {
                return None;
            }
            let mut index = b.read_u16(34 + glyph_id as usize * 2)? as usize;
            if index < 258 {
                return Some(DEFAULT_GLYPH_NAMES[index]);
            }
            index -= 258;
            let mut base = 34 + count as usize * 2;
            for _ in 0..index {
                let len = b.read::<u8>(base)? as usize;
                base += len + 1;
            }
            let len = b.read::<u8>(base)? as usize;
            base += 1;
            let bytes = b.read_bytes(base, len)?;
            return core::str::from_utf8(bytes).ok();
        }
        None
    }

    /// Returns an iterator over the available glyph names.
    pub fn glyph_names(&self) -> impl Iterator<Item = Option<&'a str>> + 'a + Clone {
        let len = match self.version().0 {
            0x10000 => 258,
            0x20000 => self.0.read_u16(32).unwrap_or(0),
            _ => 0,
        };
        Names {
            post: *self,
            cur: 0,
            len,
        }
    }

    /// Returns the value representation of the header.    
    pub fn to_value(&self) -> value::PostScript {
        value::PostScript {
            version: self.version(),
            italic_angle: self.italic_angle(),
            underline_position: self.underline_position(),
            underline_thickness: self.underline_thickness(),
            is_fixed_pitch: self.is_fixed_pitch(),
        }
    }
}

impl core::fmt::Debug for PostScript<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.to_value())
    }
}

#[derive(Clone)]
struct Names<'a> {
    post: PostScript<'a>,
    cur: u16,
    len: u16,
}

impl<'a> Iterator for Names<'a> {
    type Item = Option<&'a str>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur >= self.len {
            None
        } else {
            let cur = self.cur;
            self.cur += 1;
            Some(self.post.glyph_name(cur))
        }
    }
}

/// Tag for the `head` table.
pub const HEAD: Tag = tag::from_bytes(b"head");

/// Tag for the `hhea` table.
pub const HHEA: Tag = tag::from_bytes(b"hhea");

/// Tag for the `vhea` table.
pub const VHEA: Tag = tag::from_bytes(b"vhea");

/// Tag for the `OS/2` table.
pub const OS_2: Tag = tag::from_bytes(b"OS/2");

/// Tag for the `maxp` table.
pub const MAXP: Tag = tag::from_bytes(b"maxp");

/// Tag for the `post` table.
pub const POST: Tag = tag::from_bytes(b"post");

#[rustfmt::skip]
const DEFAULT_GLYPH_NAMES: [&str; 258] = [
    ".notdef", ".null", "nonmarkingreturn", "space", "exclam", "quotedbl", "numbersign", "dollar", 
    "percent", "ampersand", "quotesingle", "parenleft", "parenright", "asterisk", "plus", "comma", 
    "hyphen", "period", "slash", "zero", "one", "two", "three", "four", "five", "six", "seven", 
    "eight", "nine", "colon", "semicolon", "less", "equal", "greater", "question", "at", "A", "B", 
    "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", 
    "V", "W", "X", "Y", "Z", "bracketleft", "backslash", "bracketright", "asciicircum", 
    "underscore", "grave", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", 
    "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", "braceleft", "bar", "braceright", 
    "asciitilde", "Adieresis", "Aring", "Ccedilla", "Eacute", "Ntilde", "Odieresis", "Udieresis", 
    "aacute", "agrave", "acircumflex", "adieresis", "atilde", "aring", "ccedilla", "eacute", 
    "egrave", "ecircumflex", "edieresis", "iacute", "igrave", "icircumflex", "idieresis", "ntilde", 
    "oacute", "ograve", "ocircumflex", "odieresis", "otilde", "uacute", "ugrave", "ucircumflex", 
    "udieresis", "dagger", "degree", "cent", "sterling", "section", "bullet", "paragraph", 
    "germandbls", "registered", "copyright", "trademark", "acute", "dieresis", "notequal", "AE", 
    "Oslash", "infinity", "plusminus", "lessequal", "greaterequal", "yen", "mu", "partialdiff", 
    "summation", "product", "pi", "integral", "ordfeminine", "ordmasculine", "Omega", "ae", 
    "oslash", "questiondown", "exclamdown", "logicalnot", "radical", "florin", "approxequal", 
    "Delta", "guillemotleft", "guillemotright", "ellipsis", "nonbreakingspace", "Agrave", "Atilde", 
    "Otilde", "OE", "oe", "endash", "emdash", "quotedblleft", "quotedblright", "quoteleft", 
    "quoteright", "divide", "lozenge", "ydieresis", "Ydieresis", "fraction", "currency", 
    "guilsinglleft", "guilsinglright", "fi", "fl", "daggerdbl", "periodcentered", "quotesinglbase", 
    "quotedblbase", "perthousand", "Acircumflex", "Ecircumflex", "Aacute", "Edieresis", "Egrave", 
    "Iacute", "Icircumflex", "Idieresis", "Igrave", "Oacute", "Ocircumflex", "apple", "Ograve", 
    "Uacute", "Ucircumflex", "Ugrave", "dotlessi", "circumflex", "tilde", "macron", "breve", 
    "dotaccent", "ring", "cedilla", "hungarumlaut", "ogonek", "caron", "Lslash", "lslash", 
    "Scaron", "scaron", "Zcaron", "zcaron", "brokenbar", "Eth", "eth", "Yacute", "yacute", "Thorn", 
    "thorn", "minus", "multiply", "onesuperior", "twosuperior", "threesuperior", "onehalf", 
    "onequarter", "threequarters", "franc", "Gbreve", "gbreve", "Idotaccent", "Scedilla", 
    "scedilla", "Cacute", "cacute", "Ccaron", "ccaron", "dcroat",     
];
