//! Maximum profile table.

use crate::parse_prelude::*;

/// Tag for the `maxp` table.
pub const MAXP: Tag = Tag::new(b"maxp");

/// Maximum profile table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/maxp>
#[derive(Copy, Clone)]
pub struct Maxp<'a>(Buffer<'a>);

impl<'a> Maxp<'a> {
    /// Creates a new maximum profile table from a byte slice containing the
    /// table data.
    pub fn new(data: &'a [u8]) -> Self {
        Self(Buffer::new(data))
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
}
