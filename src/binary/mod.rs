pub use zerocopy;

pub(crate) use zerocopy::{AsBytes, FromBytes, Unaligned};

use crate::types::Fixed;
use zerocopy::byteorder::{self, BigEndian};

pub mod read;
pub mod write;

pub type U16 = byteorder::U16<BigEndian>;
pub type I32 = byteorder::I32<BigEndian>;
pub type U32 = byteorder::U32<BigEndian>;

#[derive(Copy, Clone, AsBytes, FromBytes, Unaligned, Default, Debug)]
#[repr(C)]
pub struct Offset16(pub U16);

impl Offset16 {
    pub fn resolve<'a, B, T>(self, base: &B, buf: &'a [u8]) -> Option<&'a T> {
        let base_address = base as *const B as usize;
        let start = base_address
            .checked_sub(buf.as_ptr() as usize)?
            .checked_add(self.0.get() as usize)?;
        let end = start.checked_add(core::mem::size_of::<T>())?;
        let bytes = buf.get(start..end)?;
        Some(unsafe { core::mem::transmute(bytes.as_ptr()) })
    }

    pub fn resolve_slice<'a, B, T>(self, base: &B, buf: &'a [u8], len: usize) -> Option<&'a [T]> {
        let base_address = base as *const B as usize;
        let start = base_address
            .checked_sub(buf.as_ptr() as usize)?
            .checked_add(self.0.get() as usize)?;
        let end = start.checked_add(len.checked_mul(core::mem::size_of::<T>())?)?;
        let bytes = buf.get(start..end)?;
        Some(unsafe { core::slice::from_raw_parts(bytes.as_ptr() as _, len) })
    }
}

#[derive(Copy, Clone, AsBytes, FromBytes, Unaligned, Default, Debug)]
#[repr(C)]
pub struct RawFixed(pub I32);

impl From<Fixed> for RawFixed {
    fn from(x: Fixed) -> Self {
        Self(I32::from(x.0))
    }
}

impl From<RawFixed> for Fixed {
    fn from(x: RawFixed) -> Self {
        Self(x.0.get())
    }
}
