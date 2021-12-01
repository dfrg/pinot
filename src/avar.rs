//! Axis variations table.

use super::parse_prelude::*;

/// Tag for the `avar` table.
pub const AVAR: Tag = Tag::new(b"avar");

/// Axis variations table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/avar>
#[derive(Copy, Clone)]
pub struct Avar<'a>(Buffer<'a>);

impl<'a> Avar<'a> {
    /// Creates a new axis variations table from a byte slice containing the
    /// table data.
    pub fn new(avar: &'a [u8]) -> Self {
        Self(Buffer::new(avar))
    }

    /// Returns the major version.
    pub fn major_version(&self) -> u16 {
        self.0.read_u16(0).unwrap_or_default()
    }

    /// Returns the minor version.
    pub fn minor_version(&self) -> u16 {
        self.0.read_u16(2).unwrap_or_default()
    }

    /// Returns the number of axes.
    pub fn num_axes(&self) -> u16 {
        self.0.read_u16(6).unwrap_or_default()
    }

    /// Returns the segment map for the specified axis.
    pub fn segment_map(&self, axis: u16) -> Option<SegmentMap<'a>> {
        let mut c = Cursor::new(self.0.data());
        c.skip(8)?;
        for _ in 0..axis {
            let count = c.read_u16()? as usize;
            c.skip(count * 4)?;
        }
        Some(SegmentMap {
            values: c.read_slice16()?,
        })
    }
}

/// Collection of value maps for a single axis.
#[derive(Copy, Clone, Debug)]
pub struct SegmentMap<'a> {
    /// Collection of value maps.
    pub values: Slice<'a, ValueMap>,
}

impl<'a> SegmentMap<'a> {
    /// Returns a modified copy of the coordinate according to the value maps.
    pub fn apply(&self, coord: Fixed) -> Fixed {
        let mut prev = ValueMap::default();
        for (i, value_map) in self.values.iter().enumerate() {
            use core::cmp::Ordering::*;
            let from = Fixed::from_f2dot14(value_map.from_coord);
            match from.cmp(&coord) {
                Equal => return Fixed::from_f2dot14(value_map.to_coord),
                Greater => {
                    if i == 0 {
                        return coord;
                    }
                    let to = Fixed::from_f2dot14(value_map.to_coord);
                    let prev_from = Fixed::from_f2dot14(prev.from_coord);
                    let prev_to = Fixed::from_f2dot14(prev.to_coord);
                    return prev_to + ((to - prev_to) * (coord - prev_from) / (from - prev_from));
                }
                _ => {}
            }
            prev = value_map;
        }
        coord
    }
}

/// Axis value mapping correspondence.
#[derive(Copy, Clone, Default, Debug)]
pub struct ValueMap {
    /// Normalized coordinate value obtained using default normalization.
    pub from_coord: F2dot14,
    /// Modified normalized coordinate value.
    pub to_coord: F2dot14,
}

impl ReadData for ValueMap {
    unsafe fn read_data_unchecked(buf: &[u8], offset: usize) -> Self {
        Self {
            from_coord: i16::read_data_unchecked(buf, offset),
            to_coord: i16::read_data_unchecked(buf, offset + 2),
        }
    }
}
