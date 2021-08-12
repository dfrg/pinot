//! Value representations of the header tables.

use crate::container::prelude::*;

/// Font header table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/head>
#[derive(Copy, Clone, Default, Debug)]
pub struct Header {
    /// Major version.
    pub major_version: u16,
    /// Minor version.
    pub minor_version: u16,
    /// Revision value. Set by font manufacturer.
    pub revision: Fixed,
    /// Checksum adjustment value.
    pub checksum_adjustment: u32,
    /// Magic number for validation. Set to 0x5F0F3CF5.
    pub magic_number: u32,
    /// Set of header bit flags.
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
    pub flags: u16,
    /// Design units per em. Valid values are 16..=16384.
    pub units_per_em: u16,
    /// Number of seconds since 12:00 midnight that started January 1st 1904 in GMT/UTC time zone.
    pub created: u64,
    /// Number of seconds since 12:00 midnight that started January 1st 1904 in GMT/UTC time zone.
    pub modified: u64,
    /// Minimum x value for all glyph bounding boxes.
    pub x_min: i16,
    /// Minimum y value for all glyph bounding boxes.
    pub y_min: i16,
    /// Maximum x value for all glyph bounding boxes.
    pub x_max: i16,
    /// Maximum y value for all glyph bounding boxes.
    pub y_max: i16,
    /// Mac style bit flags.
    /// - 0: Bold
    /// - 1: Italic
    /// - 2: Underline
    /// - 3: Outline
    /// - 4: Shadow
    /// - 5: Condensed
    /// - 6: Extended
    /// - 7-15: Reserved
    pub mac_style: u16,
    /// Smallest readable size in pixels.
    pub lowest_recommended_ppem: u16,
    /// Deprecated. Hint about the directionality of the glyphs. Set to 2.
    pub direction_hint: u16,
    /// Format the the offset array in the 'loca' table.
    /// - 0: 16-bit offsets (divided by 2)
    /// - 1: 32-bit offsets
    pub index_to_location_format: i16,
    /// Unused. Set to 0.
    pub glyph_data_format: i16,
}

/// Horizontal header table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/hhea>
#[derive(Copy, Clone, Default, Debug)]
pub struct HoriHeader {
    /// Major version number.
    pub major_version: u16,
    /// Minor version number.
    pub minor_version: u16,
    /// Typographic ascender.
    pub ascender: FWord,
    /// Typographic descender.
    pub descender: FWord,
    /// Typographic line gap.
    pub line_gap: FWord,
    /// Maximum advance width.
    pub max_advance: UfWord,
    /// Minimum left side-bearing.
    pub min_lsb: FWord,
    /// Minimum right side-bearing.
    pub min_rsb: FWord,
    /// Maximum extent: max(lsb + (x_max - x_min))
    pub max_extent: FWord,
    /// Numerator for the suggested slope of the caret.
    pub caret_rise: i16,
    /// Denominator for the suggested slope of the caret.
    pub caret_run: i16,
    /// Amount by which a slanted highlight on a glyph should be
    /// shifted.
    pub caret_offset: i16,
    /// Unused in current format. Set to 0.
    pub metric_data_format: i16,
    /// Number of "long" metric entries in the horizontal metrics table.
    pub num_long_metrics: u16,
}

/// Vertical header table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/vhea>
#[derive(Copy, Clone, Default, Debug)]
pub struct VertHeader {
    /// Major version.
    pub major_version: u16,
    /// Minor version.
    pub minor_version: u16,
    /// Typographic ascender.
    pub ascender: FWord,
    /// Typographic descender.
    pub descender: FWord,
    /// Typographic line grap.
    pub line_gap: FWord,
    /// Maximum advance height.
    pub max_advance: FWord,
    /// Minimum top side-bearing.
    pub min_tsb: FWord,
    /// Minimum bottom side-bearing.
    pub min_bsb: FWord,
    /// Maximum extent: max(tsb + (y_max - y_min))
    pub max_extent: FWord,
    /// Numerator for the suggested slope of the caret.
    pub caret_rise: i16,
    /// Denominator for the suggested slope of the caret.
    pub caret_run: i16,
    /// Amount by which a slanted highlight on a glyph should be
    /// shifted.
    pub caret_offset: i16,
    /// Unused in current format. Set to 0.
    pub metric_data_format: i16,
    /// Number of "long" metric entries in the vertical metrics table.
    pub num_long_metrics: u16,
}

/// OS/2 and Windows table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/os2>
#[derive(Copy, Clone, Default, Debug)]
pub struct Windows {
    /// Version number for the OS/2 table: 0x0000 to 0x0005.
    pub version: u16,
    /// Average advance width of all non-zero width glyphs in the
    /// font.
    pub average_char_width: UfWord,
    /// Visual weight class on a scale from 1 to 1000.  
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
    pub weight_class: u16,
    /// Visual width class-- a relative change from the normal aspect
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
    pub width_class: u16,
    /// Font type bit flags.  
    /// Bits:
    /// - 0-3: Usage permissions
    /// - 4-7: Reserved (set to 0)
    /// - 8: No subsetting
    /// - 9: Bitmap embedding only
    /// - 10-15: Reserved (set to 0)
    pub type_flags: u16,
    /// Recommended horizontal size units for subscripts.
    pub subscript_x_size: FWord,
    /// Recommended vertical size units for subscripts.
    pub subscript_y_size: FWord,
    /// Recommended horizontal offset units for subscripts.
    pub subscript_x_offset: FWord,
    /// Recommended vertical offset for subscripts.
    pub subscript_y_offset: FWord,
    /// Recommended horizontal size units for subscripts.
    pub superscript_x_size: FWord,
    /// Recommended vertical size in for subscripts.
    pub superscript_y_size: FWord,
    /// Recommended horizontal offset for subscripts.
    pub superscript_x_offset: FWord,
    /// Recommended vertical offset for subscripts.
    pub superscript_y_offset: FWord,
    /// Suggested thickness for the strikeout stroke.
    pub strikeout_size: FWord,
    /// Position of the top of the strikeout stroke relative
    /// to the baseline.
    pub strikeout_position: FWord,
    /// Font family class and subclass. For values:
    /// <https://docs.microsoft.com/en-us/typography/opentype/spec/ibmfc>
    pub family_class: i16,
    /// 10-byte PANOSE classification number.
    /// <https://monotype.github.io/panose/>
    pub panose: [u8; 10],
    /// 128-bit value describing the Unicode blocks that are
    /// supported by the font.
    pub unicode_range: [u32; 4],
    /// Four character font vendor identifier.
    pub vendor_id: [u8; 4],
    /// Font selection bit flags.  
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
    pub selection_flags: u16,
    /// Minimum Unicode index supported by the font.
    pub first_char_index: u16,
    /// Maximum Unicode index supported by the font.
    pub last_char_index: u16,
    /// Typographic ascender.
    pub typographic_ascender: FWord,
    /// Typographic descender.
    pub typographic_descender: FWord,
    /// Typographic line gap.
    pub typographic_line_gap: FWord,
    /// Windows specific value that defines the upper extent of
    /// the clipping region.
    pub win_ascent: UfWord,
    /// Windows specific value that defines the lower extent of
    /// the clipping region.
    pub win_descent: UfWord,
    /// Windows specific code page ranges supported by the font.
    /// (table version >= 1)
    pub code_page_range: Option<[u32; 2]>,
    /// Approximate distance above the baseline for non-descending
    /// lowercase letters (table version >= 2)
    pub x_height: Option<FWord>,
    /// Approximate distance above the baseline for uppercase letters.
    /// (table version >= 2)
    pub cap_height: Option<FWord>,
    /// Unicode codepoint for the default character to use if
    /// a requested character is not supported by the font.
    /// (table version >= 2)
    pub default_char: Option<u16>,
    /// Unicode codepoint for the default character used to separate
    /// words and justify text. (table version >= 2)
    pub break_char: Option<u16>,
    /// Maximum length of a target glyph context for any feature in
    /// the font. (table version >= 2)
    pub max_context: Option<u16>,
    /// Lower value of the size range for which this font has been
    /// designed. The units are TWIPS (1/20 points). (table version >= 5)
    pub lower_optical_point_size: Option<u16>,
    /// Upper value of the size range for which this font has been
    /// designed. The units are TWIPS (1/20 points). (table version >= 5)
    pub upper_optical_point_size: Option<u16>,
}

/// Maximum profile table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/maxp>
#[derive(Copy, Clone, Default, Debug)]
pub struct MaxProfile {
    /// Version of the table.
    /// - 0x00005000: Version 0.5 - only `num_glyphs` has a meaningful value.
    /// - 0x00010000: Version 1.0
    pub version: Fixed,
    /// Number of glyphs in the font.
    pub num_glyphs: u16,
    /// Maximum points in a simple glyph.
    pub max_points: u16,
    /// Maximum contours in a simple glyph.    
    pub max_contours: u16,
    /// Maximum points in a composite glyph.
    pub max_composite_points: u16,
    /// Maximum contours in a composite glyph.
    pub max_composite_contours: u16,
    /// Returns 2 if instructions require a 'twilight zone' or 1 otherwise.
    pub max_zones: u16,
    /// Maximum twilight points used in zone 0.
    pub max_twilight_points: u16,
    /// Maximum storage area locations.
    pub max_storage: u16,
    /// Maximum function definitions.
    pub max_function_defs: u16,
    /// Maximum instruction definitions.
    pub max_instruction_defs: u16,
    /// Maximum stack depth across all programs in the font.
    pub max_stack_depth: u16,
    /// Maximum size of glyph instructions.
    pub max_instructions_size: u16,
    /// Maximum number of components for a single composite glyph.
    pub max_component_elements: u16,
    /// Maximum nesting level for any composite glyph.      
    pub max_component_depth: u16,
}

/// PostScript table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/post>
#[derive(Copy, Clone, Default, Debug)]
pub struct PostScript {
    /// Returns the version of the PostScript table.
    pub version: Fixed,
    /// Returns the italic angle in counter-clockwise degrees from the vertical.
    pub italic_angle: Fixed,
    /// Returns the suggested offset of the top of the underline stroke from
    /// the baseline.
    pub underline_position: FWord,
    /// Returns the suggested thickness for the underline stroke.
    pub underline_thickness: FWord,
    /// Returns true if the font is not proportionally spaced (i.e. monospaced).
    pub is_fixed_pitch: bool,
}
