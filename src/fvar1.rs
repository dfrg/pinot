use crate::binary::read::read_n;
use crate::binary::*;
use core::mem::size_of;
use zerocopy::{AsBytes, FromBytes, Unaligned};

/// Variation axis record.
#[derive(Copy, Clone, FromBytes, AsBytes, Unaligned, Default, Debug)]
#[repr(C)]
pub struct AxisRecord {
    pub tag: U32,
    pub min_value: RawFixed,
    pub default_value: RawFixed,
    pub max_value: RawFixed,
    pub flags: U16,
    pub name_id: U16,
}

/// Fixed size header portion of a variation instance record.
#[derive(Copy, Clone, FromBytes, AsBytes, Unaligned, Default, Debug)]
#[repr(C)]
pub struct InstanceHeader {
    pub subfamily_name_id: U16,
    pub flags: U16,
}

/// Variation instance record.
#[derive(Copy, Clone)]
pub struct InstanceRecord<'a> {
    pub header: InstanceHeader,
    pub coords: &'a [RawFixed],
    pub postscript_name_id: Option<U16>,
}

/// Header for the font variations table.
#[derive(Copy, Clone, AsBytes, FromBytes, Unaligned, Default, Debug)]
#[repr(C)]
pub struct Header {
    pub major_version: U16,
    pub minor_version: U16,
    pub axes_offset: Offset16,
    pub reserved: U16,
    pub axis_count: U16,
    pub axis_size: U16,
    pub instance_count: U16,
    pub instance_size: U16,
}

/// Font variations table.
#[derive(Copy, Clone, Default)]
pub struct FvarTable<'a> {
    data: &'a [u8],
}

impl<'a> FvarTable<'a> {
    /// Creates a new font variations table from a byte slice containing
    /// the table data.
    pub fn new(data: &'a [u8]) -> Option<Self> {
        if data.len() >= size_of::<Header>() {
            Some(Self { data })
        } else {
            None
        }
    }

    /// Returns the header of the table.
    pub fn header(&self) -> &'a Header {
        const DEFAULT_HEADER: Header = Header {
            major_version: U16::ZERO,
            minor_version: U16::ZERO,
            axes_offset: Offset16(U16::ZERO),
            reserved: U16::ZERO,
            axis_count: U16::ZERO,
            axis_size: U16::ZERO,
            instance_count: U16::ZERO,
            instance_size: U16::ZERO,
        };
        read_n(self.data, 0, 1)
            .map(|slice| &slice[0])
            .unwrap_or(&DEFAULT_HEADER)
    }

    /// Returns the array of variation axis records.
    pub fn axis_records(&self) -> &'a [AxisRecord] {
        let header = self.header();
        read_n(
            self.data,
            header.axes_offset.0.get() as usize,
            header.axis_count.get() as usize,
        )
        .unwrap_or_default()
    }

    /// Returns an iterator over the variation axis records.
    pub fn axes(&self) -> impl Iterator<Item = AxisRecord> + 'a + Clone {
        self.axis_records().iter().copied()
    }

    /// Returns the variation instance record at the specified index.
    pub fn instance_record(&self, index: u16) -> Option<InstanceRecord<'a>> {
        let header = self.header();
        if index >= header.instance_count.get() {
            return None;
        }
        let axis_count = header.axis_count.get() as usize;
        let axis_size = header.axis_size.get() as usize;
        let instance_size = header.instance_size.get() as usize;
        let instance_base = axis_count * axis_size + header.axes_offset.0.get() as usize;
        let instance_offset = instance_base + instance_size * index as usize;
        let header = read_n(self.data, instance_offset, 1)?[0];
        let coords = read_n(self.data, instance_offset + 4, axis_count)?;
        let postscript_name_id = if instance_size == axis_count * 4 + 6 {
            read_n(self.data, instance_offset + 4 + axis_count * 4, 1)
                .map(|slice| slice.get(0))
                .flatten()
                .copied()
        } else {
            None
        };
        Some(InstanceRecord {
            header,
            coords,
            postscript_name_id,
        })
    }

    /// Returns an iterator over the variation instance records.
    pub fn instances(&self) -> impl Iterator<Item = InstanceRecord<'a>> + 'a + Clone {
        let copy = *self;
        (0..self.header().instance_count.get()).filter_map(move |i| copy.instance_record(i))
    }
}

#[cfg(feature = "write")]
mod write {
    use super::*;
    use alloc::vec::Vec;
    use core::ops::Range;

    struct InstanceData {
        header: InstanceHeader,
        coords: Range<usize>,
        psnid: Option<U16>,
    }

    /// Builder for constructing a font variations table.
    #[derive(Default)]
    pub struct FvarTableBuilder {
        axes: Vec<AxisRecord>,
        coords: Vec<RawFixed>,
        instances: Vec<InstanceData>,
        psnid_count: usize,
    }

    impl FvarTableBuilder {
        /// Creates a new font variations table builder.
        pub fn new() -> Self {
            Self::default()
        }

        /// Adds the specified variation axis to the table.
        pub fn axis(&mut self, axis: AxisRecord) -> &mut Self {
            self.axes.push(axis);
            self
        }

        /// Adds the specified variation instance to the table.
        pub fn instance<C>(
            &mut self,
            header: InstanceHeader,
            coords: C,
            postscript_name_id: Option<u16>,
        ) -> &mut Self
        where
            C: IntoIterator,
            C::Item: Into<RawFixed>,
        {
            let start = self.coords.len();
            self.coords.extend(coords.into_iter().map(|x| x.into()));
            let end = self.coords.len();
            self.psnid_count += postscript_name_id.is_some() as usize;
            self.instances.push(InstanceData {
                header,
                coords: start..end,
                psnid: postscript_name_id.map(|x| x.into()),
            });
            self
        }

        /// Builds a font variations table.
        pub fn build(&self) -> Vec<u8> {
            let mut buf = Vec::new();
            self.build_into(&mut buf);
            buf
        }

        /// Builds a font variations table into the specified buffer.
        pub fn build_into(&self, buf: &mut Vec<u8>) {
            let axis_count = self.axes.len();
            let has_ps_names = self.psnid_count == self.instances.len();
            let instance_size = U16::from((axis_count * 4 + 4 + has_ps_names as usize * 2) as u16);
            let header = Header {
                major_version: U16::from(1),
                minor_version: U16::from(0),
                axes_offset: Offset16(U16::from(core::mem::size_of::<Header>() as u16)),
                reserved: U16::ZERO,
                axis_count: U16::from(axis_count as u16),
                axis_size: U16::from(core::mem::size_of::<AxisRecord>() as u16),
                instance_count: U16::from(self.instances.len() as u16),
                instance_size,
            };
            buf.extend(header.as_bytes());
            for axis in &self.axes {
                buf.extend(axis.as_bytes());
            }
            for instance in &self.instances {
                buf.extend(instance.header.as_bytes());
                for coord in self
                    .coords
                    .get(instance.coords.clone())
                    .unwrap()
                    .iter()
                    .copied()
                    .chain(core::iter::repeat(RawFixed(I32::ZERO)))
                    .take(axis_count)
                {
                    buf.extend(coord.0.as_bytes());
                }
                if has_ps_names {
                    buf.extend(instance.psnid.unwrap().as_bytes())
                }
            }
        }
    }
}

#[cfg(feature = "write")]
pub use write::*;
