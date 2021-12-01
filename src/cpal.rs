//! Color palette table.

use super::name::NameId;
use super::parse_prelude::*;

/// Tag for the `CPAL` table.
pub const CPAL: Tag = Tag::new(b"CPAL");

/// Color palette table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/cpal>
#[derive(Copy, Clone)]
pub struct Cpal<'a> {
    data: Buffer<'a>,
    version: u16,
    len: u16,
    offset: u32,
}

impl<'a> Cpal<'a> {
    /// Creates a new color palette table from a byte slice containing the
    /// table data.
    pub fn new(data: &'a [u8]) -> Self {
        let data = Buffer::new(data);
        let version = data.read_or_default(0);
        let len = data.read_or_default(4);
        let offset = data.read_or_default(8);
        Self {
            data,
            version,
            len,
            offset,
        }
    }

    /// Returns the version of the table.
    pub fn version(&self) -> u16 {
        self.version
    }

    /// Returns the number of palettes in the table.
    pub fn len(&self) -> u16 {
        self.len
    }

    /// Returns true if the table is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the palette at the specified index.
    pub fn get(&self, index: u16) -> Option<Palette<'a>> {
        if index >= self.len {
            return None;
        }
        let d = &self.data;
        let name_id = (|| {
            if self.version == 0 {
                return None;
            }
            let base = 16 + self.len as usize * 2;
            let labels_offset = d.read_u32(base)? as usize;
            if labels_offset == 0 {
                return None;
            }
            d.read_u16(labels_offset + index as usize * 2)
        })();
        let flags = (|| {
            if self.version == 0 {
                return None;
            }
            let base = 12 + self.len as usize * 2;
            let types_offset = d.read_u32(base)? as usize;
            if types_offset == 0 {
                return None;
            }
            d.read_u32(types_offset + index as usize * 4)
        })()
        .unwrap_or(0);
        let theme = match flags & 0b11 {
            0b01 => Theme::Light,
            0b10 => Theme::Dark,
            _ => Theme::Any,
        };
        let len = d.read::<u16>(2)? as usize;
        let first = d.read_u32(12 + index as usize * 2)? as usize;
        let offset = self.offset as usize + first;
        let colors = d.read_slice(offset, len)?;
        Some(Palette {
            index,
            name_id,
            theme,
            colors,
        })
    }

    /// Returns an iterator over the palettes in the table.
    pub fn palettes(&self) -> impl Iterator<Item = Palette<'a>> + 'a + Clone {
        let copy = *self;
        (0..self.len).filter_map(move |i| copy.get(i))
    }
}

/// Collection of colors.
#[derive(Copy, Clone)]
pub struct Palette<'a> {
    /// Index of the palette.
    pub index: u16,
    /// Identifier for the name of the palette.
    pub name_id: Option<NameId>,
    /// Theme of the palette.
    pub theme: Theme,
    /// Color values.
    pub colors: Slice<'a, Color>,
}

/// Theme of a palette with respect to background color.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Theme {
    /// Usable with both light and dark backgrounds.
    Any,
    /// Usable with light backgrounds.
    Light,
    /// Usable with dark backgrounds.
    Dark,
}

/// RGBA color value.
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct Color {
    /// Red component.
    pub r: u8,
    /// Green component.
    pub g: u8,
    /// Blue component.
    pub b: u8,
    /// Alpha component.
    pub a: u8,
}

impl ReadData for Color {
    unsafe fn read_data_unchecked(buf: &[u8], offset: usize) -> Self {
        Self {
            r: *buf.get_unchecked(offset + 2),
            g: *buf.get_unchecked(offset + 1),
            b: *buf.get_unchecked(offset),
            a: *buf.get_unchecked(offset + 3),
        }
    }
}
