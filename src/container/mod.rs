//! Fundamental structure of a font file.

pub mod parse;
pub mod prelude;
pub mod tag;
pub mod types;

use core::ops::Range;
use prelude::*;

const TTCF: Tag = tag::from_bytes(b"ttcf");
const OTTO: Tag = tag::from_bytes(b"OTTO");
const FONT: Tag = 0x10000;
const DFNT: Tag = tag::from_bytes(&[0, 0, 1, 0]);
const TRUE: Tag = tag::from_bytes(b"true");
const SFNT: Tag = tag::from_bytes(b"sfnt");

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
    pub fn from_data(data: &[u8], offset: u32) -> Option<Self> {
        Some(match Buffer::new(data).read_u32(offset as usize)? {
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
    pub fn from_data(data: &[u8], offset: u32) -> Option<Self> {
        Some(match Buffer::new(data).read_u32(offset as usize)? {
            TTCF => Self::Collection,
            DFNT => Self::ResourceFork,
            _ => Self::Font(FontKind::from_data(data, offset)?),
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
        FontDataRef::from_data(data)?.get(index)
    }

    /// Creates a new font reference from the specified data and offset to the
    /// table directory.
    pub fn from_offset(data: &'a [u8], offset: u32) -> Option<Self> {
        let _ = FontKind::from_data(data, offset)?;
        Some(Self { data, offset })
    }

    /// Returns the kind of the font.
    pub fn kind(&self) -> Option<FontKind> {
        FontKind::from_data(self.data, self.offset)
    }

    /// Returns the number of available tables in the font.
    pub fn len(&self) -> u16 {
        let base = self.offset as usize;
        let b = Buffer::new(self.data);
        base.checked_add(4)
            .map(|offset| b.read_u16(offset))
            .flatten()
            .unwrap_or(0)
    }

    /// Returns true if the font does not contain any tables.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the table at the specified index.
    pub fn get(&self, index: u16) -> Option<Table<'a>> {
        let index = index as usize;
        let len = self.len() as usize;
        if index >= len {
            return None;
        }
        let base = self.offset as usize;
        let b = Buffer::new(self.data);
        let record_base = base.checked_add(12)?;
        let reclen = 16usize;
        let recbase = reclen.checked_mul(index)?.checked_add(record_base)?;
        let mut s = b.cursor_at(recbase)?;
        let tag = s.read_u32()?;
        let checksum = s.read_u32()?;
        let offset = s.read_u32()?;
        let len = s.read_u32()?;
        TableRecord {
            tag,
            checksum,
            offset,
            len,
        }
        .materialize(self.data)
    }

    /// Returns the table for the specified tag.
    pub fn get_by_tag(&self, tag: Tag) -> Option<Table<'a>> {
        let base = self.offset as usize;
        let b = Buffer::new(self.data);
        let len = b.read_u16(base.checked_add(4)?)? as usize;
        let record_base = base.checked_add(12)?;
        let reclen = 16usize;
        let mut lo = 0;
        let mut hi = len;
        while lo < hi {
            use core::cmp::Ordering::*;
            let i = (lo + hi) / 2;
            let recbase = reclen.checked_mul(i)?.checked_add(record_base)?;
            let mut s = b.cursor_at(recbase)?;
            let table_tag = s.read_u32()?;
            match tag.cmp(&table_tag) {
                Less => hi = i,
                Greater => lo = i + 1,
                Equal => {
                    let checksum = s.read_u32()?;
                    let offset = s.read_u32()?;
                    let len = s.read_u32()?;
                    TableRecord {
                        tag,
                        checksum,
                        offset,
                        len,
                    }
                    .materialize(self.data);
                }
            }
        }
        None
    }

    /// Returns an iterator over the table records in the font.
    pub fn iter(self) -> impl Iterator<Item = Table<'a>> + 'a + Clone {
        (0..self.len()).filter_map(move |index| self.get(index))
    }

    /// Returns the data for the table with the specified tag.
    pub fn data_by_tag(&self, tag: Tag) -> Option<&'a [u8]> {
        let range = self.table_range(tag)?;
        self.data.get(range.0 as usize..range.1 as usize)
    }

    /// Returns the byte offset for the table with the specified tag.
    pub fn offset_by_tag(&self, tag: Tag) -> Option<u32> {
        self.table_range(tag).map(|range| range.0)
    }

    /// Returns the byte range for the table with the specified tag.
    pub fn range_by_tag(&self, tag: Tag) -> Option<Range<usize>> {
        self.table_range(tag)
            .map(|range| range.0 as usize..range.0 as usize + range.1 as usize)
    }

    fn table_range(&self, tag: Tag) -> Option<(u32, u32)> {
        let base = self.offset as usize;
        let b = Buffer::new(self.data);
        let len = b.read_u16(base.checked_add(4)?)? as usize;
        let record_base = base.checked_add(12)?;
        let reclen = 16usize;
        let mut lo = 0;
        let mut hi = len;
        while lo < hi {
            use core::cmp::Ordering::*;
            let i = (lo + hi) / 2;
            let recbase = reclen.checked_mul(i)?.checked_add(record_base)?;
            let mut s = b.cursor_at(recbase)?;
            let table_tag = s.read_u32()?;
            match tag.cmp(&table_tag) {
                Less => hi = i,
                Greater => lo = i + 1,
                Equal => {
                    s.skip(4)?;
                    let start = s.read_u32()?;
                    let len = s.read_u32()?;
                    let end = start.checked_add(len)?;
                    return Some((start, end));
                }
            }
        }
        None
    }
}

/// Borrowed reference to the content of a font file.
#[derive(Copy, Clone)]
pub struct FontDataRef<'a> {
    /// Reference to the full content of a font file.
    pub data: &'a [u8],
}

impl<'a> FontDataRef<'a> {
    /// Creates a font data reference from the specified data.
    pub fn from_data(data: &'a [u8]) -> Option<Self> {
        let _ = FontDataKind::from_data(data, 0)?;
        Some(Self { data })
    }

    /// Returns the kind of the font data.
    pub fn kind(&self) -> Option<FontDataKind> {
        FontDataKind::from_data(self.data, 0)
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

/// Record for a table in a font.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct TableRecord {
    /// Table identifier.
    pub tag: Tag,
    /// Checksum for the table.
    pub checksum: u32,
    /// Offset from the beginning of the font data.
    pub offset: u32,
    /// Length of the table.
    pub len: u32,
}

impl TableRecord {
    /// Returns the byte range of the table in the font data.
    pub fn range(&self) -> Range<usize> {
        let start = self.offset as usize;
        start..start + self.len as usize
    }

    /// Returns a slice containing the data for the table in the specified
    /// font data.
    pub fn materialize<'a>(&self, data: &'a [u8]) -> Option<Table<'a>> {
        Some(Table {
            record: *self,
            data: data.get(self.range())?,
        })
    }
}

/// Record and associated data for a table in a font.
#[derive(Copy, Clone)]
pub struct Table<'a> {
    /// Record for the table.
    pub record: TableRecord,
    /// Content of the table.
    pub data: &'a [u8],
}

#[derive(Copy, Clone)]
enum Location {
    Offset(u32),
    Embedded(u32),
}

fn font_count(data: &[u8]) -> u32 {
    if let Some(kind) = FontDataKind::from_data(data, 0) {
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
    Some(match FontDataKind::from_data(data, 0)? {
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
        let tag = d.read_u32(offset)?;
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
        let tag = d.read_u32(offset)?;
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
