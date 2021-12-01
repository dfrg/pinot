//! Font variations table.

use super::parse_prelude::*;

/// Tag for the `fvar` table.
pub const FVAR: Tag = Tag::new(b"fvar");

/// Font variations table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/fvar>
#[derive(Copy, Clone)]
pub struct Fvar<'a> {
    data: Buffer<'a>,
    axis_offset: u16,
    num_axes: u16,
    axis_size: u16,
    num_instances: u16,
    instance_size: u16,
}

impl<'a> Fvar<'a> {
    /// Creates a new font variations table from a byte slice containing the
    /// table data.
    pub fn new(data: &'a [u8]) -> Self {
        let data = Buffer::new(data);
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
        }
    }

    /// Returns the major version.
    pub fn major_version(&self) -> u16 {
        self.data.read(0).unwrap_or(0)
    }

    /// Returns the minor version.
    pub fn minor_version(&self) -> u16 {
        self.data.read(2).unwrap_or(0)
    }

    /// Returns the number of available variation axes.
    pub fn num_axes(&self) -> u16 {
        self.num_axes
    }

    /// Returns the variation axis at the specified index.
    pub fn axis(&self, index: u16) -> Option<Axis> {
        if index >= self.num_axes {
            return None;
        }
        let b = &self.data;
        let base = self.axis_offset as usize;
        let offset = base + index as usize * self.axis_size as usize;
        let tag = b.read_tag(offset)?;
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
        })
    }

    /// Returns an iterator over the available variation axes.
    pub fn axes(&'a self) -> impl Iterator<Item = Axis> + 'a + Clone {
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
        let subfamily_name_id = b.read_u16(offset)?;
        let flags = b.read_u16(offset + 2)?;
        let coords = b.read_slice::<Fixed>(offset + 4, self.num_axes as usize)?;
        let ps_name_offset = 4 + self.num_axes as usize * 4;
        let postscript_name_id = if ps_name_offset == self.instance_size as usize - 2 {
            b.read_u16(ps_name_offset)
        } else {
            None
        };
        Some(Instance {
            index,
            subfamily_name_id,
            flags,
            coords,
            postscript_name_id,
        })
    }

    /// Returns an iterator over the available named instances.
    pub fn instances(&'a self) -> impl Iterator<Item = Instance<'a>> + 'a + Clone {
        (0..self.num_instances()).filter_map(move |index| self.instance(index))
    }
}

/// Axis of variation in a variable font.
#[derive(Copy, Clone, Debug, Default)]
pub struct Axis {
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
}

impl Axis {
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
}

/// Named instance in a variable font.
#[derive(Copy, Clone)]
pub struct Instance<'a> {
    /// Index of the instance.
    pub index: u16,
    /// Subfamily name identifier.
    pub subfamily_name_id: u16,
    /// Instance flags.
    pub flags: u16,
    /// Coordinates in 16.16 fixed point format.
    pub coords: Slice<'a, Fixed>,
    /// PostScript name identifier.
    pub postscript_name_id: Option<u16>,
}
