use crate::parse_prelude::*;
use core::ops::Range;

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

impl ReadData for TableRecord {
    unsafe fn read_data_unchecked(buf: &[u8], offset: usize) -> Self {
        Self {
            tag: Tag(u32::read_data_unchecked(buf, offset)),
            checksum: u32::read_data_unchecked(buf, offset + 4),
            offset: u32::read_data_unchecked(buf, offset + 8),
            len: u32::read_data_unchecked(buf, offset + 12),
        }
    }
}

impl TableRecord {
    /// Returns the byte range of the table in the font data.
    pub fn data_range(&self) -> Range<usize> {
        let start = self.offset as usize;
        start..start + self.len as usize
    }
}
