use super::parse_prelude::*;

use super::{cmap::*, fvar::*, head::*, hhea::*, maxp::*, os2::*, post::*, vhea::*};

const TTCF: Tag = Tag::new(b"ttcf");
const OTTO: Tag = Tag::new(b"OTTO");
const FONT: Tag = Tag(0x10000);
const DFNT: Tag = Tag::new(&[0, 0, 1, 0]);
const TRUE: Tag = Tag::new(b"true");
const SFNT: Tag = Tag::new(b"sfnt");

/// Kind of a font.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum FontKind {
    /// Font with TrueType outlines.
    TrueType,
    /// Font with PostScript outlines.
    OpenType,
}

impl FontKind {
    /// Returns the font kind from the specified data and offset.
    pub fn parse(data: &[u8], offset: u32) -> Option<Self> {
        Some(match Buffer::new(data).read_tag(offset as usize)? {
            FONT | TRUE => Self::TrueType,
            OTTO => Self::OpenType,
            _ => return None,
        })
    }
}

/// Kind of a font file.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum FontDataKind {
    /// Single font.
    Font(FontKind),
    /// Font collection.
    Collection,
    /// Resource fork.
    ResourceFork,
}

impl FontDataKind {
    /// Returns the font data kind from the specified data and offset.
    pub fn parse(data: &[u8], offset: u32) -> Option<Self> {
        Some(match Buffer::new(data).read_tag(offset as usize)? {
            TTCF => Self::Collection,
            DFNT => Self::ResourceFork,
            _ => Self::Font(FontKind::parse(data, offset)?),
        })
    }
}

/// Borrowed reference to a font.
#[derive(Copy, Clone)]
pub struct FontRef<'a> {
    /// Reference to the full content of a font file.
    pub data: &'a [u8],
    /// Offset to the table directory.
    pub offset: u32,
}

impl<'a> FontRef<'a> {
    /// Creates a new font reference from the specified data and index.
    pub fn from_index(data: &'a [u8], index: u32) -> Option<Self> {
        FontDataRef::new(data)?.get(index)
    }

    /// Creates a new font reference from the specified data and offset to the
    /// table directory.
    pub fn from_offset(data: &'a [u8], offset: u32) -> Option<Self> {
        let _ = FontKind::parse(data, offset)?;
        Some(Self { data, offset })
    }

    /// Returns the kind of the font.
    pub fn kind(&self) -> Option<FontKind> {
        FontKind::parse(self.data, self.offset)
    }
}

/// Borrowed reference to the content of a font file.
#[derive(Copy, Clone)]
pub struct FontDataRef<'a> {
    /// Reference to the full content of a font file.
    pub data: &'a [u8],
}

impl<'a> FontDataRef<'a> {
    /// Creates a font data reference for the specified data.
    pub fn new(data: &'a [u8]) -> Option<Self> {
        let _ = FontDataKind::parse(data, 0)?;
        Some(Self { data })
    }

    /// Returns the kind of the font data.
    pub fn kind(&self) -> Option<FontDataKind> {
        FontDataKind::parse(self.data, 0)
    }

    /// Returns the number of available fonts.
    pub fn len(&self) -> u32 {
        font_count(self.data)
    }

    /// Returns true there are no available fonts.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the font at the specified index.
    pub fn get(&self, index: u32) -> Option<FontRef<'a>> {
        let (data, offset) = match font_location(self.data, index)? {
            Location::Offset(offset) => (self.data, offset),
            Location::Embedded(offset) => (self.data.get(offset as usize..)?, 0),
        };
        Some(FontRef { data, offset })
    }

    /// Returns an iterator over the available fonts.
    pub fn iter(self) -> impl Iterator<Item = FontRef<'a>> + 'a + Clone {
        (0..self.len()).filter_map(move |index| self.get(index))
    }
}

/// Abstract interface for a collection of font tables.
pub trait TableProvider<'a> {
    /// Returns the data for the table with the specified tag.
    fn table_data(&self, tag: Tag) -> Option<&'a [u8]>;

    /// Returns the font header table.
    fn head(&self) -> Option<Head<'a>> {
        Some(Head::new(self.table_data(HEAD)?))
    }

    /// Returns the PostScript table.
    fn post(&self) -> Option<Post<'a>> {
        Some(Post::new(self.table_data(POST)?))
    }

    /// Returns the OS/2 and Windows metrics table.
    fn os2(&self) -> Option<Os2<'a>> {
        Some(Os2::new(self.table_data(OS2)?))
    }

    /// Returns the horizontal header table.
    fn hhea(&self) -> Option<Hhea<'a>> {
        Some(Hhea::new(self.table_data(HHEA)?))
    }

    /// Returns the vertical header table.
    fn vhea(&self) -> Option<Vhea<'a>> {
        Some(Vhea::new(self.table_data(VHEA)?))
    }

    /// Returns the maximum profile table.
    fn maxp(&self) -> Option<Maxp<'a>> {
        Some(Maxp::new(self.table_data(MAXP)?))
    }

    /// Returns the character mapping table.
    fn cmap(&self) -> Option<Cmap<'a>> {
        Some(Cmap::new(self.table_data(CMAP)?))
    }

    /// Returns the font variations table.
    fn fvar(&self) -> Option<Fvar<'a>> {
        Some(Fvar::new(self.table_data(FVAR)?))
    }
}

impl<'a> TableProvider<'a> for FontRef<'a> {
    fn table_data(&self, _tag: Tag) -> Option<&'a [u8]> {
        None
    }
}

impl<'a> TableProvider<'a> for &'_ FontRef<'a> {
    fn table_data(&self, _tag: Tag) -> Option<&'a [u8]> {
        None
    }
}

#[derive(Copy, Clone)]
enum Location {
    Offset(u32),
    Embedded(u32),
}

fn font_count(data: &[u8]) -> u32 {
    if let Some(kind) = FontDataKind::parse(data, 0) {
        match kind {
            FontDataKind::Collection => Buffer::new(data).read_u32(8).unwrap_or(0),
            FontDataKind::ResourceFork => dfont_count(data).unwrap_or(0),
            _ => 1,
        }
    } else {
        0
    }
}

fn font_location(data: &[u8], index: u32) -> Option<Location> {
    if index >= font_count(data) {
        return None;
    }
    Some(match FontDataKind::parse(data, 0)? {
        FontDataKind::Collection => {
            Location::Offset(Buffer::new(data).read_u32(12 + index as usize * 4)?)
        }
        FontDataKind::ResourceFork => Location::Embedded(dfont_range(data, index)?.0),
        _ => Location::Offset(0),
    })
}

fn dfont_count(data: &[u8]) -> Option<u32> {
    let d = Buffer::new(data);
    let resource_map_base = d.read_u32(4)? as usize;
    let type_list_base = resource_map_base + d.read_u16(resource_map_base + 24)? as usize;
    let type_count = d.read_u16(type_list_base)? as usize + 1;
    for i in 0..type_count {
        let offset = type_list_base + 2 + i * 8;
        let tag = d.read_tag(offset)?;
        if tag == SFNT {
            return Some(d.read_u16(offset + 4)? as u32 + 1);
        }
    }
    None
}

fn dfont_range(data: &[u8], index: u32) -> Option<(u32, u32)> {
    let d = Buffer::new(data);
    let data_base = d.read_u32(0)?;
    let resource_map_base = d.read_u32(4)? as usize;
    let type_list_base = resource_map_base + d.read_u16(resource_map_base + 24)? as usize;
    let type_count = d.read_u16(type_list_base)? as usize + 1;
    for i in 0..type_count {
        let offset = type_list_base + 2 + i * 8;
        let tag = d.read_tag(offset)?;
        if tag == SFNT {
            let count = d.read_u16(offset + 4)? as u32 + 1;
            if index >= count {
                return None;
            }
            let resources_base = type_list_base + d.read_u16(offset + 6)? as usize;
            let resource_base = resources_base + index as usize * 12;
            let data_offset = data_base + d.read::<U24>(resource_base + 5)?.0;
            let data_len = d.read_u32(data_offset as usize)?;
            return Some((data_offset + 4, data_offset + 4 + data_len));
        }
    }
    None
}
