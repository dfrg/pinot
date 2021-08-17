//! Support for variable fonts.

pub mod item;
// pub mod tuple;

use crate::container::prelude::*;

/// Tag for the `fvar` table.
pub const FVAR: Tag = tag::from_bytes(b"fvar");

/// Tag for the `avar` table.
pub const AVAR: Tag = tag::from_bytes(b"avar");

/// Font variations table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/fvar>
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/avar>
#[derive(Copy, Clone)]
pub struct Variations<'a> {
    data: Buffer<'a>,
    axis_offset: u16,
    num_axes: u16,
    axis_size: u16,
    num_instances: u16,
    instance_size: u16,
    avar: Option<&'a [u8]>,
}

impl<'a> Variations<'a> {
    /// Creates a new font variations table from the `fvar` and optional
    /// `avar` tables.
    pub fn new(fvar: &'a [u8], avar: Option<&'a [u8]>) -> Self {
        let data = Buffer::new(fvar);
        let axis_offset = data.read_or_default::<u16>(4);
        let num_axes = data.read_or_default::<u16>(8);
        let axis_size = data.read_or_default::<u16>(10);
        let num_instances = data.read_or_default::<u16>(12);
        let instance_size = data.read_or_default::<u16>(14);
        Self {
            data,
            axis_offset,
            num_axes,
            axis_size,
            num_instances,
            instance_size,
            avar,
        }
    }

    /// Returns the number of available variation axes.
    pub fn num_axes(&self) -> u16 {
        self.num_axes
    }

    /// Returns the variation axis at the specified index.
    pub fn axis(&self, index: u16) -> Option<Axis<'a>> {
        if index >= self.num_axes {
            return None;
        }
        let b = &self.data;
        let base = self.axis_offset as usize;
        let offset = base + index as usize * self.axis_size as usize;
        let tag = b.read::<u32>(offset)?;
        let min_value = Fixed(b.read::<i32>(offset + 4)?);
        let default_value = Fixed(b.read::<i32>(offset + 8)?);
        let max_value = Fixed(b.read::<i32>(offset + 12)?);
        let flags = b.read::<u16>(offset + 16)?;
        let name_id = b.read::<u16>(offset + 18)?;
        Some(Axis {
            index,
            tag,
            name_id,
            flags,
            min_value,
            default_value,
            max_value,
            avar: self.avar,
        })
    }

    /// Returns an iterator over the available variation axes.
    pub fn axes(&'a self) -> impl Iterator<Item = Axis<'a>> + 'a + Clone {
        (0..self.num_axes()).filter_map(move |index| self.axis(index))
    }

    /// Returns the number of available named instances.
    pub fn num_instances(&self) -> u16 {
        self.num_instances
    }

    /// Returns the named instance at the specified index.
    pub fn instance(&self, index: u16) -> Option<Instance<'a>> {
        if index >= self.num_instances {
            return None;
        }
        let b = &self.data;
        let base = self.axis_offset as usize + (self.num_axes as usize * self.axis_size as usize);
        let offset = base + index as usize * self.instance_size as usize;
        let name_id = b.read::<u16>(offset)?;
        let values = b.read_slice::<Fixed>(offset + 4, self.num_axes as usize)?;
        let ps_name_offset = 4 + self.num_axes as usize * 4;
        let postscript_name_id = if ps_name_offset == self.instance_size as usize - 2 {
            b.read::<u16>(ps_name_offset)
        } else {
            None
        };
        Some(Instance {
            index,
            name_id,
            postscript_name_id,
            values,
        })
    }

    /// Returns an iterator over the available named instances.
    pub fn instances(&'a self) -> impl Iterator<Item = Instance<'a>> + 'a + Clone {
        (0..self.num_instances()).filter_map(move |index| self.instance(index))
    }
}

/// Axis of variation in a variable font.
#[derive(Copy, Clone, Debug, Default)]
pub struct Axis<'a> {
    /// Index of the axis.
    pub index: u16,
    /// Tag that identifies the axis.
    pub tag: Tag,
    /// Name identifier.
    pub name_id: u16,
    /// Axis flags.
    pub flags: u16,
    /// Minimum value of the axis.
    pub min_value: Fixed,
    /// Default value of the axis.
    pub default_value: Fixed,
    /// Maximum value of the axis.
    pub max_value: Fixed,
    /// Axis variations table.
    avar: Option<&'a [u8]>,
}

impl<'a> Axis<'a> {
    /// Returns true if the axis should be hidden in a user interface.
    pub fn is_hidden(&self) -> bool {
        self.flags & 1 != 0
    }

    /// Returns a normalized coordinate for the specified value in 16.16
    /// fixed point format.
    pub fn normalize(&self, mut value: Fixed) -> Fixed {
        use core::cmp::Ordering::*;
        if value < self.min_value {
            value = self.min_value;
        } else if value > self.max_value {
            value = self.max_value;
        }
        value = match value.cmp(&self.default_value) {
            Less => -((self.default_value - value) / (self.default_value - self.min_value)),
            Greater => (value - self.default_value) / (self.max_value - self.default_value),
            Equal => Fixed(0),
        };
        value.min(Fixed::ONE).max(-Fixed::ONE)
    }

    /// Returns the sequence of axis value maps the define a modification
    /// within the variation space.
    pub fn segment_map(&self) -> Option<SegmentMap<'a>> {
        let mut c = Cursor::new(self.avar?);
        c.skip(8)?;
        for _ in 0..self.index {
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
    pub values: Slice<'a, AxisValueMap>,
}

impl<'a> SegmentMap<'a> {
    /// Applies the piecewise linear mapping to the normalized coordinate.
    pub fn apply(&self, coord: Fixed) -> Fixed {
        let mut prev = AxisValueMap::default();
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
pub struct AxisValueMap {
    /// Normalized coordinate value obtained using default normalization.
    pub from_coord: F2dot14,
    /// Modified normalized coordinate value.
    pub to_coord: F2dot14,
}

impl ReadData for AxisValueMap {
    unsafe fn read_data_unchecked(buf: &[u8], offset: usize) -> Self {
        Self {
            from_coord: i16::read_data_unchecked(buf, offset),
            to_coord: i16::read_data_unchecked(buf, offset + 2),
        }
    }
}

/// Named instance in a variable font.
#[derive(Copy, Clone)]
pub struct Instance<'a> {
    /// Index of the instance.
    pub index: u16,
    /// Name identifier.
    pub name_id: u16,
    /// PostScript name identifier.
    pub postscript_name_id: Option<u16>,
    /// Axis values in 16.16 fixed point format.
    pub values: Slice<'a, Fixed>,
}
