//! PostScript table.

use crate::parse_prelude::*;

/// Tag for the `post` table.
pub const POST: Tag = Tag::new(b"post");

/// PostScript table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/post>
#[derive(Copy, Clone)]
pub struct Post<'a>(Buffer<'a>);

impl<'a> Post<'a> {
    /// Creates a new PostScript table from a byte slice containing the table
    /// data.
    pub fn new(data: &'a [u8]) -> Self {
        Self(Buffer::new(data))
    }

    /// Returns the version of the PostScript table.
    pub fn version(&self) -> Fixed {
        self.0.read(0).unwrap_or(Fixed::ZERO)
    }

    /// Returns the italic angle in counter-clockwise degrees from the vertical.
    pub fn italic_angle(&self) -> Fixed {
        self.0.read(4).unwrap_or(Fixed::ZERO)
    }

    /// Returns the suggested offset of the top of the underline stroke from
    /// the baseline.
    pub fn underline_position(&self) -> FWord {
        self.0.read(8).unwrap_or(0)
    }

    /// Returns the suggested thickness for the underline stroke.
    pub fn underline_thickness(&self) -> FWord {
        self.0.read(10).unwrap_or(0)
    }

    /// Returns true if the font is not proportionally spaced (i.e. monospaced).
    pub fn is_fixed_pitch(&self) -> bool {
        self.0.read_u32(12).unwrap_or(0) != 0
    }

    /// Returns true if the table can provide glyph names. Only versions 1.0
    /// (0x00010000) and 2.0 (0x00020000).
    pub fn has_glyph_names(&self) -> bool {
        let v = self.version().0;
        v == 0x10000 || v == 0x20000
    }

    /// Returns the name of the specified glyph if available.
    pub fn glyph_name(&self, glyph_id: GlyphId) -> Option<&'a str> {
        if !self.has_glyph_names() {
            return None;
        }
        let v = self.version().0;
        if v == 0x10000 {
            if glyph_id >= 258 {
                return None;
            }
            return Some(DEFAULT_GLYPH_NAMES[glyph_id as usize]);
        } else if v == 0x20000 {
            let b = &self.0;
            let count = b.read_u16(32)?;
            if glyph_id >= count {
                return None;
            }
            let mut index = b.read_u16(34 + glyph_id as usize * 2)? as usize;
            if index < 258 {
                return Some(DEFAULT_GLYPH_NAMES[index]);
            }
            index -= 258;
            let mut base = 34 + count as usize * 2;
            for _ in 0..index {
                let len = b.read::<u8>(base)? as usize;
                base += len + 1;
            }
            let len = b.read::<u8>(base)? as usize;
            base += 1;
            let bytes = b.read_bytes(base, len)?;
            return core::str::from_utf8(bytes).ok();
        }
        None
    }

    /// Returns an iterator over the available glyph names.
    pub fn glyph_names(&self) -> impl Iterator<Item = Option<&'a str>> + 'a + Clone {
        let len = match self.version().0 {
            0x10000 => 258,
            0x20000 => self.0.read_u16(32).unwrap_or(0),
            _ => 0,
        };
        Names {
            post: *self,
            cur: 0,
            len,
        }
    }
}

#[derive(Clone)]
struct Names<'a> {
    post: Post<'a>,
    cur: u16,
    len: u16,
}

impl<'a> Iterator for Names<'a> {
    type Item = Option<&'a str>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur >= self.len {
            None
        } else {
            let cur = self.cur;
            self.cur += 1;
            Some(self.post.glyph_name(cur))
        }
    }
}

#[rustfmt::skip]
const DEFAULT_GLYPH_NAMES: [&str; 258] = [
    ".notdef", ".null", "nonmarkingreturn", "space", "exclam", "quotedbl", "numbersign", "dollar", 
    "percent", "ampersand", "quotesingle", "parenleft", "parenright", "asterisk", "plus", "comma", 
    "hyphen", "period", "slash", "zero", "one", "two", "three", "four", "five", "six", "seven", 
    "eight", "nine", "colon", "semicolon", "less", "equal", "greater", "question", "at", "A", "B", 
    "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", 
    "V", "W", "X", "Y", "Z", "bracketleft", "backslash", "bracketright", "asciicircum", 
    "underscore", "grave", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", 
    "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", "braceleft", "bar", "braceright", 
    "asciitilde", "Adieresis", "Aring", "Ccedilla", "Eacute", "Ntilde", "Odieresis", "Udieresis", 
    "aacute", "agrave", "acircumflex", "adieresis", "atilde", "aring", "ccedilla", "eacute", 
    "egrave", "ecircumflex", "edieresis", "iacute", "igrave", "icircumflex", "idieresis", "ntilde", 
    "oacute", "ograve", "ocircumflex", "odieresis", "otilde", "uacute", "ugrave", "ucircumflex", 
    "udieresis", "dagger", "degree", "cent", "sterling", "section", "bullet", "paragraph", 
    "germandbls", "registered", "copyright", "trademark", "acute", "dieresis", "notequal", "AE", 
    "Oslash", "infinity", "plusminus", "lessequal", "greaterequal", "yen", "mu", "partialdiff", 
    "summation", "product", "pi", "integral", "ordfeminine", "ordmasculine", "Omega", "ae", 
    "oslash", "questiondown", "exclamdown", "logicalnot", "radical", "florin", "approxequal", 
    "Delta", "guillemotleft", "guillemotright", "ellipsis", "nonbreakingspace", "Agrave", "Atilde", 
    "Otilde", "OE", "oe", "endash", "emdash", "quotedblleft", "quotedblright", "quoteleft", 
    "quoteright", "divide", "lozenge", "ydieresis", "Ydieresis", "fraction", "currency", 
    "guilsinglleft", "guilsinglright", "fi", "fl", "daggerdbl", "periodcentered", "quotesinglbase", 
    "quotedblbase", "perthousand", "Acircumflex", "Ecircumflex", "Aacute", "Edieresis", "Egrave", 
    "Iacute", "Icircumflex", "Idieresis", "Igrave", "Oacute", "Ocircumflex", "apple", "Ograve", 
    "Uacute", "Ucircumflex", "Ugrave", "dotlessi", "circumflex", "tilde", "macron", "breve", 
    "dotaccent", "ring", "cedilla", "hungarumlaut", "ogonek", "caron", "Lslash", "lslash", 
    "Scaron", "scaron", "Zcaron", "zcaron", "brokenbar", "Eth", "eth", "Yacute", "yacute", "Thorn", 
    "thorn", "minus", "multiply", "onesuperior", "twosuperior", "threesuperior", "onehalf", 
    "onequarter", "threequarters", "franc", "Gbreve", "gbreve", "Idotaccent", "Scedilla", 
    "scedilla", "Cacute", "cacute", "Ccaron", "ccaron", "dcroat",     
];
