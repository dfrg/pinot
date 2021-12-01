//! Naming table.

use super::parse_prelude::*;
use core::ops::Range;

/// Tag for the `name` table.
pub const NAME: Tag = Tag::new(b"name");

/// Naming table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/name>
#[derive(Copy, Clone)]
pub struct Name<'a>(Buffer<'a>);

impl<'a> Name<'a> {
    /// Creates a new naming table from a byte slice containing the table
    /// data.
    pub fn new(data: &'a [u8]) -> Self {
        Self(Buffer::new(data))
    }

    /// Returns the version.
    pub fn version(&self) -> u16 {
        self.0.read(0).unwrap_or(0)
    }

    /// Returns the list of name records.
    pub fn records(&self) -> Slice<'a, NameRecord> {
        let len = self.0.read_u16(2).unwrap_or_default() as usize;
        self.0.read_slice(6, len).unwrap_or_default()
    }

    /// Returns an iterator over the entries in the table.
    pub fn entries(&self) -> impl Iterator<Item = Entry<'a>> + 'a + Clone {
        let copy = *self;
        self.records()
            .iter()
            .map(move |record| Entry { name: copy, record })
    }

    /// Returns the storage area for the string data.
    pub fn storage(&self) -> &'a [u8] {
        if let Some(offset) = self.0.read_offset16(4, 0) {
            self.0.data().get(offset as usize..).unwrap_or(&[])
        } else {
            &[]
        }
    }
}

/// Record for an entry in the naming table.
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct NameRecord {
    /// Platform identifier.
    pub platform_id: u16,
    /// Encoding identifier.
    pub encoding_id: u16,
    /// Language identifier,
    pub language_id: u16,
    /// Name identifier.
    pub name_id: u16,
    /// Length of the string in the storage area.
    pub len: u16,
    /// Offset to the string in the storage area.
    pub offset: u16,
}

impl NameRecord {
    /// Returns true if the string data can be decoded.
    pub fn is_decodable(&self) -> bool {
        encoding(self.platform_id, self.encoding_id) < 2
    }

    /// Returns the byte range for the string data in the storage area.
    pub fn storage_range(&self) -> Range<usize> {
        let start = self.offset as usize;
        start..start + self.len as usize
    }
}

impl ReadData for NameRecord {
    unsafe fn read_data_unchecked(buf: &[u8], offset: usize) -> Self {
        Self {
            platform_id: u16::read_data_unchecked(buf, offset),
            encoding_id: u16::read_data_unchecked(buf, offset + 2),
            language_id: u16::read_data_unchecked(buf, offset + 4),
            name_id: u16::read_data_unchecked(buf, offset + 6),
            len: u16::read_data_unchecked(buf, offset + 8),
            offset: u16::read_data_unchecked(buf, offset + 10),
        }
    }
}

/// Entry for a name in the naming table.
#[derive(Copy, Clone)]
pub struct Entry<'a> {
    /// Parent table.
    pub name: Name<'a>,
    /// Record for the name.
    pub record: NameRecord,
}

impl<'a> Entry<'a> {
    /// Returns the raw string data for the name entry.
    pub fn data(&self) -> Option<&'a [u8]> {
        self.name.storage().get(self.record.storage_range())
    }

    /// Returns an iterator over the characters in the name entry.
    pub fn decode(&self) -> Decode<'a> {
        let data = Buffer::new(self.data().unwrap_or(&[]));
        let encoding = encoding(self.record.platform_id, self.record.encoding_id);
        let len = data.len();
        Decode {
            data,
            encoding,
            len,
            pos: 0,
        }
    }
}

/// Iterator over the chars of a name record.
#[derive(Copy, Clone)]
pub struct Decode<'a> {
    data: Buffer<'a>,
    encoding: u32,
    len: usize,
    pos: usize,
}

impl<'a> Iterator for Decode<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.len {
            return None;
        }
        use core::char::from_u32;
        let rep = core::char::REPLACEMENT_CHARACTER;
        let d = &self.data;
        match self.encoding {
            0 => {
                let mut c = d.read::<u16>(self.pos)? as u32;
                self.pos += 2;
                if (0xD800..0xDC00).contains(&c) {
                    let c2 = d.read::<u16>(self.pos)? as u32;
                    self.pos += 2;
                    c = ((c & 0x3FF) << 10) + (c2 & 0x3FF) + 0x10000;
                }
                Some(from_u32(c).unwrap_or(rep))
            }
            1 => {
                let c = self.data.0[self.pos] as u32;
                self.pos += 1;
                if c > 127 {
                    let idx = c as usize - 128;
                    Some(from_u32(MAC_ROMAN[idx] as u32).unwrap_or(rep))
                } else {
                    Some(from_u32(c).unwrap_or(rep))
                }
            }
            _ => None,
        }
    }
}

/// Copyright notice.
pub const COPYRIGHT_NOTICE: u16 = 0;
/// Family name.
pub const FAMILY_NAME: u16 = 1;
/// Subfamily name.
pub const SUBFAMILY_NAME: u16 = 2;
/// Unique identifier.
pub const UNIQUE_ID: u16 = 3;
/// Full name.
pub const FULL_NAME: u16 = 4;
/// Version string.
pub const VERSION_STRING: u16 = 5;
/// PostScript name.
pub const POSTSCRIPT_NAME: u16 = 6;
/// Trademark.
pub const TRADEMARK: u16 = 7;
/// Manufacturer name.
pub const MANUFACTURER: u16 = 8;
/// Designer name.
pub const DESIGNER: u16 = 9;
/// Description of the typeface.
pub const DESCRIPTION: u16 = 10;
/// URL of the font vendor.
pub const VENDOR_URL: u16 = 11;
/// URL of the font designer.
pub const DESIGNER_URL: u16 = 12;
/// License description.
pub const LICENSE_DESCRIPTION: u16 = 13;
/// URL where additional licensing information can be found.
pub const LICENSE_URL: u16 = 14;
/// Typographic family name.
pub const TYPOGRAPHIC_FAMILY_NAME: u16 = 16;
/// Typographic subfamily name.
pub const TYPOGRAPHIC_SUBFAMILY_NAME: u16 = 17;
/// Compatible full name (Macintosh only).
pub const COMPATIBLE_FULL_NAME: u16 = 18;
/// Sample text.
pub const SAMPLE_TEXT: u16 = 19;
/// PostScript CID findfont name.
pub const POSTSCRIPT_CID_NAME: u16 = 20;
/// WWS family name.
pub const WWS_FAMILY_NAME: u16 = 21;
/// WWS subfamily name.
pub const WWS_SUBFAMILY_NAME: u16 = 22;
/// Light background palette name.
pub const LIGHT_BACKGROUND_PALETTE: u16 = 23;
/// Dark background palette name.
pub const DARK_BACKGROUND_PALETTE: u16 = 24;
/// Variations PostScript name prefix.
pub const VARIATIONS_POSTSCRIPT_NAME_PREFIX: u16 = 25;

fn encoding(platform_id: u16, encoding_id: u16) -> u32 {
    match (platform_id, encoding_id) {
        (0, _) => 0,
        (1, 0) => 1,
        (3, 0) => 0,
        (3, 1) => 0,
        (3, 10) => 0,
        _ => 2,
    }
}

#[rustfmt::skip]
const MAC_ROMAN: [u16; 128] = [
    196, 197, 199, 201, 209, 214, 220, 225, 224, 226, 228, 227, 229, 231, 233,
    232, 234, 235, 237, 236, 238, 239, 241, 243, 242, 244, 246, 245, 250, 249,
    251, 252, 8224, 176, 162, 163, 167, 8226, 182, 223, 174, 169, 8482, 180,
    168, 8800, 198, 216, 8734, 177, 8804, 8805, 165, 181, 8706, 8721, 8719,
    960, 8747, 170, 186, 937, 230, 248, 191, 161, 172, 8730, 402, 8776, 8710,
    171, 187, 8230, 160, 192, 195, 213, 338, 339, 8211, 8212, 8220, 8221, 8216,
    8217, 247, 9674, 255, 376, 8260, 8364, 8249, 8250, 64257, 64258, 8225, 183,
    8218, 8222, 8240, 194, 202, 193, 203, 200, 205, 206, 207, 204, 211, 212,
    63743, 210, 218, 219, 217, 305, 710, 732, 175, 728, 729, 730, 184, 733,
    731, 711,
];
