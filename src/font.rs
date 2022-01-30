use super::parse_prelude::*;

use super::{
    avar::*,
    cmap::*,
    colr::{Colr, COLR},
    cpal::*,
    fvar::*,
    gdef::*,
    gpos::*,
    gsub::*,
    head::*,
    hhea::*,
    hmtx::*,
    hvar::*,
    math::*,
    maxp::*,
    name::*,
    os2::*,
    post::*,
    vhea::*,
    vmtx::*,
    vorg::*,
    vvar::*,
};

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

    /// Returns the list of table records in the font.
    pub fn records(&self) -> Slice<'a, TableRecord> {
        let base = self.offset as usize;
        let d = Buffer::new(self.data);
        let len = self.len() as usize;
        d.read_slice(base + 12, len as usize).unwrap_or_default()
    }

    /// Returns the table record with the specified tag.
    pub fn find_record(&self, tag: Tag) -> Option<TableRecord> {
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
            let table_tag = s.read_tag()?;
            match tag.cmp(&table_tag) {
                Less => hi = i,
                Greater => lo = i + 1,
                Equal => {
                    let checksum = s.read_u32()?;
                    let offset = s.read_u32()?;
                    let len = s.read_u32()?;
                    return Some(TableRecord {
                        tag,
                        checksum,
                        offset,
                        len,
                    });
                }
            }
        }
        None
    }

    /// Returns an iterator over the tables in the font.
    pub fn tables(&self) -> impl Iterator<Item = Table<'a>> + 'a + Clone {
        let data = self.data;
        self.records().iter().filter_map(move |record| {
            Some(Table {
                data: data.get(record.data_range())?,
                record,
            })
        })
    }

    /// Returns the table for the specified tag.
    pub fn find_table(&self, tag: Tag) -> Option<Table<'a>> {
        let record = self.find_record(tag)?;
        Some(Table {
            data: self.data.get(record.data_range())?,
            record,
        })
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
    pub fn fonts(self) -> impl Iterator<Item = FontRef<'a>> + 'a + Clone {
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

    /// Returns the maximum profile table.
    fn maxp(&self) -> Option<Maxp<'a>> {
        Some(Maxp::new(self.table_data(MAXP)?))
    }

    /// Returns the OS/2 and Windows metrics table.
    fn os2(&self) -> Option<Os2<'a>> {
        Some(Os2::new(self.table_data(OS2)?))
    }

    /// Returns the horizontal header table.
    fn hhea(&self) -> Option<Hhea<'a>> {
        Some(Hhea::new(self.table_data(HHEA)?))
    }

    /// Returns the horizontal metrics table.
    fn hmtx(&self) -> Option<Hmtx<'a>> {
        let num_glyphs = self.maxp()?.num_glyphs();
        let num_metrics = self.hhea()?.num_long_metrics();
        Some(Hmtx::new(self.table_data(HMTX)?, num_glyphs, num_metrics))
    }

    /// Returns the horizontal metrics variation table.
    fn hvar(&self) -> Option<Hvar<'a>> {
        Some(Hvar::new(self.table_data(HVAR)?))
    }

    /// Returns the vertical header table.
    fn vhea(&self) -> Option<Vhea<'a>> {
        Some(Vhea::new(self.table_data(VHEA)?))
    }

    /// Returns the vertical metrics table.
    fn vmtx(&self) -> Option<Vmtx<'a>> {
        let num_glyphs = self.maxp()?.num_glyphs();
        let num_metrics = self.vhea()?.num_long_metrics();
        Some(Vmtx::new(self.table_data(VMTX)?, num_glyphs, num_metrics))
    }

    /// Returns the vertical origin table.
    fn vorg(&self) -> Option<Vorg<'a>> {
        Some(Vorg::new(self.table_data(VORG)?))
    }

    /// Returns the vertical metrics variation table.
    fn vvar(&self) -> Option<Vvar<'a>> {
        Some(Vvar::new(self.table_data(VVAR)?))
    }

    /// Returns the naming table.
    fn name(&self) -> Option<Name<'a>> {
        Some(Name::new(self.table_data(NAME)?))
    }

    /// Returns the character mapping table.
    fn cmap(&self) -> Option<Cmap<'a>> {
        Some(Cmap::new(self.table_data(CMAP)?))
    }

    /// Returns the font variations table.
    fn fvar(&self) -> Option<Fvar<'a>> {
        Some(Fvar::new(self.table_data(FVAR)?))
    }

    /// Returns the axis variations table.
    fn avar(&self) -> Option<Avar<'a>> {
        Some(Avar::new(self.table_data(AVAR)?))
    }

    /// Returns the color palette table.
    fn cpal(&self) -> Option<Cpal<'a>> {
        Some(Cpal::new(self.table_data(CPAL)?))
    }

    /// Returns the color table.
    fn colr(&self) -> Option<Colr<'a>> {
        Some(Colr::new(self.table_data(COLR)?))
    }

    /// Returns the glyph definition table.
    fn gdef(&self) -> Option<Gdef<'a>> {
        Gdef::new(self.table_data(GDEF)?)
    }

    /// Returns the glyph substitution table.
    fn gsub(&self) -> Option<Gsub<'a>> {
        Some(Gsub::new(self.table_data(GSUB)?, self.gdef()))
    }

    /// Returns the glyph positioning table.
    fn gpos(&self) -> Option<Gpos<'a>> {
        Some(Gpos::new(self.table_data(GPOS)?, self.gdef()))
    }

    /// Returns the mathemetical typesetting table.
    fn math(&self) -> Option<Math<'a>> {
        Some(Math::new(self.table_data(MATH)?))
    }
}

impl<'a> TableProvider<'a> for FontRef<'a> {
    fn table_data(&self, tag: Tag) -> Option<&'a [u8]> {
        self.data.get(self.find_record(tag)?.data_range())
    }
}

impl<'a> TableProvider<'a> for &'_ FontRef<'a> {
    fn table_data(&self, tag: Tag) -> Option<&'a [u8]> {
        self.data.get(self.find_record(tag)?.data_range())
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
