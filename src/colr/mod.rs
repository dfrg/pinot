//! Color table.

mod paint;

use super::parse_prelude::*;
use super::var::item::*;
use core::ops::Range;

pub use paint::*;

/// Tag for the `COLR` table.
pub const COLR: Tag = Tag::new(b"COLR");

/// Color table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/colr>
#[derive(Copy, Clone)]
pub struct Colr<'a> {
    data: Buffer<'a>,
    version: u16,
}

impl<'a> Colr<'a> {
    /// Creates a new color table from a byte slice containing the
    /// table data.
    pub fn new(data: &'a [u8]) -> Self {
        let data = Buffer::new(data);
        let version = data.read_or_default(0);
        Self { data, version }
    }

    /// Returns the version of the table.
    pub fn version(&self) -> u16 {
        self.version
    }

    /// Returns the number of glyphs in the table.
    pub fn num_glyphs(&self) -> u16 {
        self.data.read_u16(2).unwrap_or_default()
    }

    /// Returns the glyph at the specified index.
    pub fn glyph(&self, index: u16) -> Option<Glyph<'a>> {
        if index >= self.num_glyphs() {
            return None;
        }
        let d = &self.data;
        let offset = d.read_u32(4)? as usize + index as usize * 6;
        let gid = d.read_u16(offset)?;
        let first = d.read_u16(offset + 2)? as usize;
        let offset = d.read_u32(8)? as usize + first * 4;
        let len = d.read_u16(offset + 4)? as usize;
        let layers = d.read_slice(offset, len)?;
        Some(Glyph { gid, layers })
    }

    /// Returns the glyph for the specified identifier.
    pub fn find_glyph(&self, gid: GlyphId) -> Option<Glyph<'a>> {
        let d = &self.data;
        let base_offset = d.read_u32(4)? as usize;
        let mut lo = 0;
        let mut hi = self.num_glyphs() as usize;
        while lo < hi {
            use core::cmp::Ordering::*;
            let i = (lo + hi) / 2;
            let offset = base_offset + i * 6;
            let id = d.read_u16(offset)?;
            match gid.cmp(&id) {
                Less => hi = i,
                Greater => lo = i + 1,
                Equal => {
                    let first = d.read_u16(offset + 2)? as usize;
                    let offset = d.read_u32(8)? as usize + first * 4;
                    let len = d.read_u16(offset + 4)? as usize;
                    let layers = d.read_slice(offset, len)?;
                    return Some(Glyph { gid, layers });
                }
            }
        }
        None
    }

    /// Returns an iterator over the glyphs in the table.
    pub fn glyphs(&self) -> impl Iterator<Item = Glyph<'a>> + 'a + Clone {
        let copy = *self;
        (0..self.num_glyphs()).filter_map(move |i| copy.glyph(i))
    }

    /// Returns the number of base paints.
    pub fn num_base_paints(&self) -> u32 {
        if self.version < 1 {
            return 0;
        }
        let base = self.data.read_u32(14).unwrap_or_default() as usize;
        if base == 0 {
            return 0;
        }
        self.data.read_u32(base).unwrap_or_default()
    }

    /// Returns the glyph identifier and base paint at the specified index.
    pub fn base_paint(&self, index: u32) -> Option<(GlyphId, Paint<'a>)> {
        if self.version < 1 {
            return None;
        }
        let index = index as usize;
        let base = self.data.read_u32(14)? as usize;
        let len = self.data.read_u32(base)? as usize;
        if index >= len {
            return None;
        }
        let record_base = base + 4 + index * 6;
        let id = self.data.read_u16(record_base)?;
        let paint_offset = base + self.data.read_u32(record_base + 2)? as usize;
        Some((id, PaintRef::new(self.data, paint_offset as u32)?.get()?))
    }

    /// Returns the base paint for the specified glyph identifier.
    pub fn find_base_paint(&self, gid: GlyphId) -> Option<Paint<'a>> {
        if self.version < 1 {
            return None;
        }
        let base = self.data.read_u32(14)? as usize;
        let len = self.data.read_u32(base)? as usize;
        let mut lo = 0;
        let mut hi = len;
        while lo < hi {
            use core::cmp::Ordering::*;
            let i = (lo + hi) / 2;
            let record_base = base + 4 + i * 6;
            let id = self.data.read_u16(record_base)?;
            match gid.cmp(&id) {
                Less => hi = i,
                Greater => lo = i + 1,
                Equal => {
                    let paint_offset = base + self.data.read_u32(record_base + 2)? as usize;
                    return PaintRef::new(self.data, paint_offset as u32)?.get();
                }
            }
        }
        None
    }

    /// Returns an iterator over the collection of base paints in the table.
    pub fn base_paints(&self) -> impl Iterator<Item = (GlyphId, Paint<'a>)> + 'a + Clone {
        let copy = *self;
        (0..self.num_base_paints()).filter_map(move |i| copy.base_paint(i))
    }

    /// Returns the number of paint layers in the table.
    pub fn num_paint_layers(&self) -> u32 {
        if self.version < 1 {
            return 0;
        }
        let base = self.data.read_u32(18).unwrap_or_default() as usize;
        if base == 0 {
            return 0;
        }
        self.data.read_u32(base).unwrap_or_default()
    }

    /// Returns the paint layer at the specified index.
    pub fn paint_layer(&self, index: u32) -> Option<Paint<'a>> {
        if self.version < 1 {
            return None;
        }
        let index = index as usize;
        let base = self.data.read_u32(18)? as usize;
        let len = self.data.read_u32(base)? as usize;
        if index >= len {
            return None;
        }
        let record_base = base + 4 + index * 4;
        let paint_offset = base + self.data.read_u32(record_base)? as usize;
        PaintRef::new(self.data, paint_offset as u32)?.get()
    }

    /// Returns an iterator over the paint layers in the table.
    pub fn paint_layers(&self) -> impl Iterator<Item = Paint<'a>> + 'a + Clone {
        let copy = *self;
        (0..self.num_paint_layers()).filter_map(move |i| copy.paint_layer(i))
    }

    /// Returns the number of clip boxes in the table.
    pub fn num_clip_boxes(&self) -> u32 {
        if self.version < 1 {
            return 0;
        }
        let base = self.data.read_u32(22).unwrap_or_default() as usize;
        if base == 0 {
            return 0;
        }
        self.data.read_u32(base + 1).unwrap_or_default()
    }

    /// Returns the glyph identifier range and clip box for the specified
    /// index.
    pub fn clip_box(&self, index: u32) -> Option<(Range<GlyphId>, ClipBox)> {
        if self.version < 1 {
            return None;
        }
        let d = &self.data;
        let base = d.read_u32(22)? as usize;
        if base == 0 {
            return None;
        }
        let len = d.read_u32(base + 1)? as usize;
        let index = index as usize;
        if index >= len {
            return None;
        }
        let record_base = base + 5 + index * 7;
        let start = d.read_u16(record_base)?;
        let end = d.read_u16(record_base + 2)? + 1;
        let offset = d.read_u24(record_base + 4)?;
        if offset == 0 {
            return None;
        }
        let clip_base = record_base + offset as usize;
        let format = d.read_u8(clip_base)?;
        let x_min = f2dot14_to_f32(d.read_i16(clip_base + 1)?);
        let y_min = f2dot14_to_f32(d.read_i16(clip_base + 5)?);
        let x_max = f2dot14_to_f32(d.read_i16(clip_base + 9)?);
        let y_max = f2dot14_to_f32(d.read_i16(clip_base + 13)?);
        let var_index = if format == 2 {
            Some(d.read_u32(clip_base + 17)?)
        } else {
            None
        };
        Some((
            start..end,
            ClipBox {
                x_min,
                y_min,
                x_max,
                y_max,
                var_index,
            },
        ))
    }

    /// Returns the clip box for the specified glyph identifier.
    pub fn find_clip_box(&self, gid: GlyphId) -> Option<ClipBox> {
        if self.version < 1 {
            return None;
        }
        let d = &self.data;
        let base = d.read_u32(22)? as usize;
        if base == 0 {
            return None;
        }
        let len = d.read_u32(base + 1)? as usize;
        let mut lo = 0;
        let mut hi = len;
        while lo < hi {
            let i = (lo + hi) / 2;
            let record_base = base + 5 + i * 7;
            let start = d.read_u16(record_base)?;
            if gid < start {
                lo = i + 1;
            } else if gid > d.read_u16(record_base + 2)? {
                hi = i;
            } else {
                let offset = d.read_u24(record_base + 4)?;
                if offset == 0 {
                    return None;
                }
                let clip_base = record_base + offset as usize;
                let format = d.read_u8(clip_base)?;
                let x_min = f2dot14_to_f32(d.read_i16(clip_base + 1)?);
                let y_min = f2dot14_to_f32(d.read_i16(clip_base + 5)?);
                let x_max = f2dot14_to_f32(d.read_i16(clip_base + 9)?);
                let y_max = f2dot14_to_f32(d.read_i16(clip_base + 13)?);
                let var_index = if format == 2 {
                    Some(d.read_u32(clip_base + 17)?)
                } else {
                    None
                };
                return Some(ClipBox {
                    x_min,
                    y_min,
                    x_max,
                    y_max,
                    var_index,
                });
            }
        }
        None
    }

    /// Returns an iterator over the clip boxes in the table.
    pub fn clip_boxes(&self) -> impl Iterator<Item = (Range<GlyphId>, ClipBox)> + 'a + Clone {
        let copy = *self;
        (0..self.num_clip_boxes()).filter_map(move |i| copy.clip_box(i))
    }

    /// Returns the mapping for variation indices.
    pub fn var_mapping(&self) -> Option<DeltaSetIndexMap<'a>> {
        if self.version < 1 {
            return None;
        }
        DeltaSetIndexMap::new(self.data, self.data.read_offset32(26, 0)?)
    }

    /// Returns the item variation store.
    pub fn ivs(&self) -> Option<ItemVariationStore<'a>> {
        if self.version < 1 {
            return None;
        }
        ItemVariationStore::new(self.data, self.data.read_offset32(30, 0)?)
    }
}

/// Sequence of layers that define a color outline.
#[derive(Copy, Clone)]
pub struct Glyph<'a> {
    /// Glyph identifier for the outline.
    pub gid: GlyphId,
    /// Color layers that define the outline.
    pub layers: Slice<'a, Layer>,
}

/// Single layer in a color outline.
#[derive(Copy, Clone)]
pub struct Layer {
    /// Glyph that contains the outline for the layer.
    pub gid: GlyphId,
    /// Index value to use with the selected color palette.
    pub palette_index: Option<u16>,
}

impl ReadData for Layer {
    const SIZE: usize = 4;

    unsafe fn read_data_unchecked(buf: &[u8], offset: usize) -> Self {
        let gid = u16::read_data_unchecked(buf, offset);
        let index = u16::read_data_unchecked(buf, offset + 2);
        Self {
            gid,
            palette_index: if index == 0xFFFF { None } else { Some(index) },
        }
    }
}

fn fixed_to_f32(x: i32) -> f32 {
    const SCALE: f32 = 1. / 65536.;
    x as f32 * SCALE
}

fn f2dot14_to_f32(x: i16) -> f32 {
    const SCALE: f32 = 1. / 65536.;
    (x as i32 * 4) as f32 * SCALE
}
