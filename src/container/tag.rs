//! Support for working with four byte tag values.

use core::fmt;

/// Four byte tag value used to identify various resources.
pub type Tag = u32;

/// Creates a tag from an array.
pub const fn from_bytes(bytes: &[u8; 4]) -> Tag {
    (bytes[0] as u32) << 24 | (bytes[1] as u32) << 16 | (bytes[2] as u32) << 8 | bytes[3] as u32
}

/// Creates a tag from a string. Encodes missing characters as spaces.
pub fn from_str_lossy(s: &str) -> Tag {
    let mut bytes = [b' '; 4];
    for (i, b) in s.as_bytes().iter().enumerate().take(4) {
        bytes[i] = *b;
    }
    from_bytes(&bytes)
}

/// Adapter for displaying a tag.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct DisplayTag(pub [u8; 4]);

impl fmt::Display for DisplayTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Ok(s) = core::str::from_utf8(&self.0) {
            write!(f, "{}", s)
        } else {
            write!(f, "{:?}", self.0)
        }
    }
}

/// Creates an adapter for displaying the specified tag.
pub const fn display(tag: Tag) -> DisplayTag {
    DisplayTag(tag.to_be_bytes())
}
