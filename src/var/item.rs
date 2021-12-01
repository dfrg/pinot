//! Variation deltas for single values.

use crate::parse_prelude::*;

/// Two level index for accessing data in an item variation store.
#[derive(Copy, Clone, Debug)]
pub struct Index {
    /// Outer-level index.
    pub outer: u16,
    /// Inner-level index.
    pub inner: u16,
}

impl Index {
    /// Creates a new index.
    pub fn new(outer: u16, inner: u16) -> Self {
        Self { outer, inner }
    }
}

/// Mapping from a glyph identifier to a pair of indices for an item
/// variation store.
#[derive(Copy, Clone)]
pub struct DeltaSetIndexMap<'a> {
    data: Buffer<'a>,
    offset: usize,
    format: u8,
    entry_format: u16,
    count: u32,
}

impl<'a> DeltaSetIndexMap<'a> {
    /// Creates a new delta set index map from the specified buffer and offset.
    pub fn new(data: Buffer<'a>, offset: u32) -> Option<Self> {
        if offset == 0 {
            return None;
        }
        let offset = offset as usize;
        let format = data.read_u8(offset)?;
        let entry_format = data.read_u8(offset + 1)? as u16;
        let count = match format {
            0 => data.read_u16(offset + 2)? as u32,
            1 => data.read_u32(offset + 2)?,
            _ => return None,
        };
        Some(Self {
            data,
            offset,
            format,
            entry_format,
            count,
        })
    }

    /// Returns the outer and inner indices for an item variation store.
    pub fn get(&self, index: u32) -> Option<Index> {
        let d = &self.data;
        let format = self.entry_format as u32;
        let bit_count = (format & 0xF) + 1;
        let entry_size = ((format & 0x30) >> 4) + 1;
        let base = self.offset + if self.format == 0 { 4 } else { 6 };
        let index = index.min(self.count - 1) as usize;
        let entry = match entry_size {
            1 => d.read_u8(base + index)? as u32,
            2 => d.read_u16(base + index * 2)? as u32,
            3 => d.read_u24(base + index * 3)?,
            4 => d.read_u32(base + index * 4)?,
            _ => return None,
        };
        let outer = entry >> bit_count;
        let inner = entry & ((1 << bit_count) - 1);
        Some(Index {
            outer: outer as u16,
            inner: inner as u16,
        })
    }
}

/// Item variation store.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/otvarcommonformats#item-variation-store>
#[derive(Copy, Clone, Default)]
pub struct ItemVariationStore<'a> {
    data: Buffer<'a>,
    pub(crate) offset: u32,
    regions_offset: u32,
    num_axes: u16,
    num_regions: u16,
    num_outer_sets: u16,
}

impl<'a> ItemVariationStore<'a> {
    /// Creates a new variation store for the specified data and offset.
    pub fn new(data: Buffer<'a>, offset: u32) -> Option<Self> {
        let regions_offset = data.read_offset32(offset as usize + 2, offset)?;
        let num_axes = data.read_u16(regions_offset as usize)?;
        let num_regions = data.read_u16(regions_offset as usize + 2)?;
        let num_outer_sets = data.read_u16(offset as usize + 6)?;
        Some(Self {
            data,
            offset,
            regions_offset,
            num_axes,
            num_regions,
            num_outer_sets,
        })
    }

    /// Returns the number of variation axes.
    pub fn num_axes(&self) -> u16 {
        self.num_axes
    }

    /// Returns the number of variation regions.
    pub fn num_regions(&self) -> u16 {
        self.num_regions
    }

    /// Returns the variation region at the specified index.
    pub fn region(&self, index: u16) -> Option<Region<'a>> {
        if index >= self.num_regions {
            return None;
        }
        let offset =
            self.regions_offset as usize + 4 + (self.num_axes as usize * index as usize) * 6;
        Some(Region {
            index,
            axes: self.data.read_slice(offset, self.num_axes as usize)?,
        })
    }

    /// Returns an iterator over the variation regions.
    pub fn regions(&'a self) -> impl Iterator<Item = Region<'a>> + 'a + Clone {
        (0..self.num_regions).filter_map(move |index| self.region(index))
    }

    /// Returns the number of "outer" sets.
    pub fn num_outer_sets(&self) -> u16 {
        self.num_outer_sets
    }

    /// Returns the number of "inner" delta sets for the specified outer set.
    pub fn num_inner_sets(&self, outer: u16) -> u16 {
        if outer >= self.num_outer_sets {
            return 0;
        }
        (|| {
            let offset = self
                .data
                .read_offset32(self.offset as usize + 8 + outer as usize * 4, self.offset)?;
            self.data.read_u16(offset as usize)
        })()
        .unwrap_or(0)
    }

    /// Returns an iterator over the per-region delta values for the specified
    /// outer and inner indices.
    pub fn deltas(
        &'a self,
        index: Index,
    ) -> impl Iterator<Item = (Region<'a>, Fixed)> + 'a + Clone {
        self.outer_set(index.outer)
            .map(|set| set.get(self, index.inner))
            .unwrap_or_default()
    }

    /// Returns a delta value for the specified delta set indices and normalized
    /// variation coordinates.
    pub fn delta(&self, index: Index, coords: &[NormalizedCoord]) -> Fixed {
        let mut delta = Fixed::ZERO;
        for (region, value) in self.deltas(index) {
            let scalar = region.compute_scalar(coords);
            delta += value * scalar;
        }
        delta
    }

    fn outer_set(&self, index: u16) -> Option<OuterSet<'a>> {
        if index >= self.num_outer_sets {
            return None;
        }
        let offset = self
            .data
            .read_offset32(self.offset as usize + 8 + index as usize * 4, self.offset)?;
        OuterSet::new(self.data, offset as usize)
    }
}

/// Region along a single axis.
#[derive(Copy, Clone)]
pub struct Coords {
    /// Region start coordinate.
    pub start: NormalizedCoord,
    /// Region peak coordinate.
    pub peak: NormalizedCoord,
    /// Region end coordinate.
    pub end: NormalizedCoord,
}

impl ReadData for Coords {
    unsafe fn read_data_unchecked(buf: &[u8], offset: usize) -> Self {
        Self {
            start: i16::read_data_unchecked(buf, offset),
            peak: i16::read_data_unchecked(buf, offset + 2),
            end: i16::read_data_unchecked(buf, offset + 4),
        }
    }
}

/// Per-axis regions in the variation space.
#[derive(Copy, Clone)]
pub struct Region<'a> {
    /// Index of the region.
    pub index: u16,
    /// Regions in the variation space for each axis.
    pub axes: Slice<'a, Coords>,
}

impl<'a> Region<'a> {
    /// Returns a scalar value for this region and the specified normalized
    /// variation coordinates.
    pub fn compute_scalar(&self, coords: &[NormalizedCoord]) -> Fixed {
        let mut scalar = Fixed::ONE;
        for (i, axis_coords) in self.axes.iter().enumerate() {
            let coord = coords.get(i).copied().unwrap_or(0);
            let start = axis_coords.start;
            let end = axis_coords.end;
            let peak = axis_coords.peak;
            if start > peak || peak > end || peak == 0 || start < 0 && end > 0 {
                continue;
            } else if coord < start || coord > end {
                return Fixed::ZERO;
            } else if coord == peak {
                continue;
            } else {
                let coord = Fixed::from_f2dot14(coord);
                let start = Fixed::from_f2dot14(start);
                let end = Fixed::from_f2dot14(end);
                let peak = Fixed::from_f2dot14(peak);
                if coord < peak {
                    scalar = scalar * (coord - start) / (peak - start)
                } else {
                    scalar = scalar * (end - coord) / (end - peak)
                }
            };
        }
        scalar
    }
}

#[derive(Copy, Clone)]
struct OuterSet<'a> {
    data: Buffer<'a>,
    offset: usize,
    len: u16,
    num_word_deltas: u16,
    long_words: bool,
    row_size: u32,
    region_indices: Slice<'a, u16>,
}

impl<'a> OuterSet<'a> {
    fn new(data: Buffer<'a>, offset: usize) -> Option<Self> {
        let len = data.read_u16(offset)?;
        let num_word_deltas = data.read_u16(offset + 2)?;
        let long_words = num_word_deltas & 0x8000 != 0;
        let (word_size, small_size) = if long_words { (4, 2) } else { (2, 1) };
        let num_word_deltas = num_word_deltas & 0x7FFF;
        let region_indices = data.read_slice16(offset + 4)?;
        let num_regions = region_indices.len() as u32;
        let row_size = num_word_deltas as u32 * word_size
            + (num_regions - num_word_deltas as u32) * small_size;
        Some(Self {
            data,
            offset,
            len,
            num_word_deltas,
            long_words,
            row_size,
            region_indices,
        })
    }

    /// Returns the indices for the regions used by deltas in this set.
    pub fn region_indices(&self) -> Slice<'a, u16> {
        self.region_indices
    }

    /// Returns an iterator over the deltas for the specified index. This is the
    /// `inner` index in an item delta reference.
    pub fn get(&self, store: &ItemVariationStore<'a>, index: u16) -> Deltas<'a> {
        if index >= self.len {
            return Deltas::default();
        }
        let offset = self.offset + self.row_size as usize * index as usize;
        if let Some(cursor) = self.data.cursor_at(offset) {
            Deltas {
                store: *store,
                cursor,
                region_indices: self.region_indices(),
                num_word_deltas: self.num_word_deltas,
                long_words: self.long_words,
                len: self.region_indices.len() as u16,
                pos: 0,
            }
        } else {
            Deltas::default()
        }
    }
}

/// Iterator over the set of per-region deltas for an item.
#[derive(Copy, Clone, Default)]
struct Deltas<'a> {
    store: ItemVariationStore<'a>,
    cursor: Cursor<'a>,
    region_indices: Slice<'a, u16>,
    num_word_deltas: u16,
    long_words: bool,
    len: u16,
    pos: u16,
}

impl<'a> Iterator for Deltas<'a> {
    type Item = (Region<'a>, Fixed);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.len {
            return None;
        }
        let pos = self.pos;
        self.pos += 1;
        let value = Fixed::from_i32(if pos >= self.num_word_deltas {
            if self.long_words {
                self.cursor.read_i16()? as i32
            } else {
                self.cursor.read_i8()? as i32
            }
        } else if self.long_words {
            self.cursor.read_i32()?
        } else {
            self.cursor.read_i16()? as i32
        });
        let index = self.region_indices.get(pos as usize)?;
        let region = self.store.region(index)?;
        Some((region, value))
    }
}
