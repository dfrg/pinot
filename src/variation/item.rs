//! Variation deltas for individual values.

use crate::container::prelude::*;

/// Item variation store.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/otvarcommonformats#item-variation-store>
#[derive(Copy, Clone)]
pub struct Store<'a> {
    data: Buffer<'a>,
    offset: u32,
    regions_offset: u32,
    num_axes: u16,
    num_regions: u16,
    num_outer_sets: u16,
}

impl<'a> Store<'a> {
    /// Creates a new variation store for the specified data and offset.
    pub fn new(data: &'a [u8], offset: u32) -> Self {
        let data = Buffer::new(data);
        let (regions_offset, num_axes, num_regions) =
            if let Some(regions_offset) = data.read_offset32(offset as usize + 2, offset) {
                let num_axes = data.read_u16(regions_offset as usize).unwrap_or(0);
                let num_regions = data.read_u16(regions_offset as usize + 2).unwrap_or(0);
                (regions_offset, num_axes, num_regions)
            } else {
                (0, 0, 0)
            };
        let num_outer_sets = data.read_u16(offset as usize + 6).unwrap_or(0);
        Self {
            data,
            offset,
            regions_offset,
            num_axes,
            num_regions,
            num_outer_sets,
        }
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
        outer: u16,
        inner: u16,
    ) -> impl Iterator<Item = (Region<'a>, FWord)> + 'a + Clone {
        self.outer_set(outer)
            .map(|set| set.get(self, inner))
            .unwrap_or_default()
    }

    /// Returns a delta value for the specified delta set indices and normalized
    /// variation coordinates.
    pub fn delta(&self, outer: u16, inner: u16, coords: &[NormalizedCoord]) -> Fixed {
        let mut delta = Fixed::ZERO;
        for (region, value) in self.deltas(outer, inner) {
            let scalar = region.compute_scalar(coords);
            delta += Fixed::from_i32(value as i32) * scalar;
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
    num_short_deltas: u16,
    row_size: u32,
    region_indices: Slice<'a, u16>,
}

impl<'a> OuterSet<'a> {
    fn new(data: Buffer<'a>, offset: usize) -> Option<Self> {
        let len = data.read_u16(offset)?;
        let num_short_deltas = data.read_u16(offset + 2)?;
        let region_indices = data.read_slice16(offset + 4)?;
        let num_regions = region_indices.len() as u32;
        let row_size = num_short_deltas as u32 * 2 + (num_regions - num_short_deltas as u32);
        Some(Self {
            data,
            offset,
            len,
            num_short_deltas,
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
    pub fn get(&self, store: &Store<'a>, index: u16) -> Deltas<'a> {
        if index >= self.len {
            return Deltas::default();
        }
        let offset = self.offset + self.row_size as usize * index as usize;
        if let Some(cursor) = self.data.cursor_at(offset) {
            Deltas {
                store: *store,
                cursor,
                region_indices: self.region_indices(),
                num_short_deltas: self.num_short_deltas,
                len: self.region_indices.len() as u16,
                pos: 0,
            }
        } else {
            Deltas::default()
        }
    }
}

/// Iterator over the set of per-region deltas for an item.
#[derive(Copy, Clone)]
struct Deltas<'a> {
    store: Store<'a>,
    cursor: Cursor<'a>,
    region_indices: Slice<'a, u16>,
    num_short_deltas: u16,
    len: u16,
    pos: u16,
}

impl<'a> Iterator for Deltas<'a> {
    type Item = (Region<'a>, i16);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.len {
            return None;
        }
        let pos = self.pos;
        self.pos += 1;
        let value = if pos >= self.num_short_deltas {
            self.cursor.read_i8()? as i16
        } else {
            self.cursor.read_i16()?
        };
        let index = self.region_indices.get(pos as usize)?;
        let region = self.store.region(index)?;
        Some((region, value))
    }
}

impl Default for Deltas<'_> {
    fn default() -> Self {
        Self {
            store: Store::new(&[], 0),
            cursor: Cursor::default(),
            region_indices: Slice::default(),
            num_short_deltas: 0,
            len: 0,
            pos: 0,
        }
    }
}
