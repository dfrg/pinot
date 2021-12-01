use core::fmt;

/// Four byte tag for identifying resources and settings.
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Default)]
pub struct Tag(pub u32);

impl Tag {
    /// Creates a new tag from a sequence of four bytes.
    pub const fn new(bytes: &[u8; 4]) -> Self {
        Self(
            (bytes[0] as u32) << 24
                | (bytes[1] as u32) << 16
                | (bytes[2] as u32) << 8
                | bytes[3] as u32,
        )
    }

    /// Creates a new tag from the first four bytes of string. If the string is
    /// shorter than four bytes, the remaining slots are packed with spaces.
    pub fn from_str_lossy(s: &str) -> Self {
        let mut bytes = [b' '; 4];
        for (i, b) in s.as_bytes().iter().enumerate().take(4) {
            bytes[i] = *b;
        }
        Self::new(&bytes)
    }

    /// Returns the four bytes of the tag.
    pub fn to_bytes(self) -> [u8; 4] {
        self.0.to_be_bytes()
    }
}

impl From<&[u8; 4]> for Tag {
    fn from(bytes: &[u8; 4]) -> Self {
        Self(u32::from_be_bytes(*bytes))
    }
}

impl From<[u8; 4]> for Tag {
    fn from(bytes: [u8; 4]) -> Self {
        Self(u32::from_be_bytes(bytes))
    }
}

impl From<&str> for Tag {
    fn from(s: &str) -> Self {
        Self::from_str_lossy(s)
    }
}

impl fmt::Debug for Tag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Tag(\"{}\")", self)
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bytes = self.to_bytes();
        match core::str::from_utf8(&bytes) {
            Ok(s) => write!(f, "{}", s),
            _ => write!(f, "{:?}", bytes),
        }
    }
}
