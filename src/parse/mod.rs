//! Parsing primitives.

#![allow(dead_code)]

use core::ops::Range;

pub mod array;
pub mod slice;

pub use slice::Slice;

use super::types::Tag;

/// Parser for for reading from arbitrary offsets.
#[derive(Copy, Clone)]
pub struct Buffer<'a>(pub &'a [u8]);

impl<'a> Buffer<'a> {
    /// Creates a new data instance for the specified bytes.
    pub fn new(bytes: &'a [u8]) -> Self {
        Self(bytes)
    }

    /// Creates a new data instance for the specified bytes and offset.
    pub fn with_offset(bytes: &'a [u8], offset: usize) -> Option<Self> {
        Some(Self(bytes.get(offset..)?))
    }

    /// Creates a new data instance with the specified range of bytes.
    pub fn with_range(bytes: &'a [u8], range: Range<usize>) -> Option<Self> {
        Some(Self(bytes.get(range)?))
    }

    /// Returns the underlying data.
    pub fn data(&self) -> &'a [u8] {
        self.0
    }

    /// Returns the length of the underlying data.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the underlying data is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns true if the specified range is within the bounds of the
    /// underlying data.
    pub fn check_range(&self, offset: usize, len: usize) -> bool {
        let end = self.0.len();
        (offset < end) && (end - offset >= len)
    }

    /// Returns an error if the specified range is not within the bounds of
    /// the underlying data.
    pub fn ensure_range(&self, offset: usize, len: usize) -> Option<()> {
        if self.check_range(offset, len) {
            Some(())
        } else {
            None
        }
    }

    /// Reads a value of the specified type at the specified offset.
    #[inline(always)]
    pub fn read<T: ReadData>(&self, offset: usize) -> Option<T> {
        T::read_data(self.0, offset)
    }

    /// Reads a value of the specified type at some offset, or returns the
    /// default value on bounds check failure.
    pub fn read_or_default<T: ReadData + Default>(&self, offset: usize) -> T {
        T::read_data(self.0, offset).unwrap_or_default()
    }

    /// Returns a value of the specified type at some offset without bounds
    /// checking.
    ///
    /// # Safety
    /// This is safe if `offset + T::SIZE <= self.len()`.
    #[inline(always)]
    pub unsafe fn read_unchecked<T: ReadData>(&self, offset: usize) -> T {
        T::read_data_unchecked(self.0, offset)
    }

    /// Reads a u8 value at the specified offset.
    #[inline(always)]
    pub fn read_u8(&self, offset: usize) -> Option<u8> {
        u8::read_data(self.0, offset)
    }

    /// Reads a u16 value at the specified offset.
    #[inline(always)]
    pub fn read_u16(&self, offset: usize) -> Option<u16> {
        u16::read_data(self.0, offset)
    }

    /// Reads a u24 value at the specified offset.
    #[inline(always)]
    pub fn read_u24(&self, offset: usize) -> Option<u32> {
        U24::read_data(self.0, offset).map(|x| x.0)
    }

    /// Reads a u32 value at the specified offset.
    #[inline(always)]
    pub fn read_u32(&self, offset: usize) -> Option<u32> {
        u32::read_data(self.0, offset)
    }

    /// Reads an i8 value at the specified offset.
    #[inline(always)]
    pub fn read_i8(&self, offset: usize) -> Option<i8> {
        i8::read_data(self.0, offset)
    }

    /// Reads an i16 value at the specified offset.
    #[inline(always)]
    pub fn read_i16(&self, offset: usize) -> Option<i16> {
        i16::read_data(self.0, offset)
    }

    /// Reads an i16 value at the specified offset.
    #[inline(always)]
    pub fn read_i32(&self, offset: usize) -> Option<i32> {
        i32::read_data(self.0, offset)
    }

    /// Reads a tag at the specified offset.
    #[inline(always)]
    pub fn read_tag(&self, offset: usize) -> Option<Tag> {
        Some(Tag(self.read_u32(offset)?))
    }

    /// Reads a 16-bit value at the specified offset and if non-zero, returns
    /// the result added to the specified base.
    pub fn read_offset16(&self, offset: usize, base: u32) -> Option<u32> {
        let value = self.read_u16(offset)? as u32;
        if value != 0 {
            Some(value + base)
        } else {
            None
        }
    }

    /// Reads a 32-bit value at the specified offset and if non-zero, returns
    /// the result added to the specified base.
    pub fn read_offset32(&self, offset: usize, base: u32) -> Option<u32> {
        let value = self.read_u32(offset)?;
        if value != 0 {
            Some(value + base)
        } else {
            None
        }
    }

    /// Reads a slice of values of the specified type and length at some
    /// offset.
    pub fn read_slice<T: ReadData>(&self, offset: usize, len: usize) -> Option<Slice<'a, T>> {
        let len = len * T::SIZE;
        if !self.check_range(offset, len) {
            return None;
        }
        Some(Slice::new(&self.0[offset..offset + len]))
    }

    /// Reads a 16-bit length followed by slice of values of the specified type at some
    /// offset.
    pub fn read_slice16<T: ReadData>(&self, offset: usize) -> Option<Slice<'a, T>> {
        let len = self.read_u16(offset)? as usize;
        self.read_slice(offset + 2, len)
    }

    /// Reads a 32-bit length followed by slice of values of the specified type at some
    /// offset.
    pub fn read_slice32<T: ReadData>(&self, offset: usize) -> Option<Slice<'a, T>> {
        let len = self.read_u32(offset)? as usize;
        self.read_slice(offset + 4, len)
    }

    /// Reads a sequence of bytes at the specified offset and length.
    pub fn read_bytes(&self, offset: usize, len: usize) -> Option<&'a [u8]> {
        if !self.check_range(offset, len) {
            return None;
        }
        Some(&self.0[offset..offset + len])
    }

    /// Creates a new cursor at the specified offset.
    pub fn cursor_at(&self, offset: usize) -> Option<Cursor<'a>> {
        Cursor::with_offset(self.0, offset)
    }
}

impl<'a> core::ops::Deref for Buffer<'a> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl Default for Buffer<'_> {
    fn default() -> Self {
        Self::new(&[])
    }
}

/// Parser for sequential reading.
#[derive(Copy, Clone)]
pub struct Cursor<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    /// Creates a new cursor wrapping the specified bytes.
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    /// Creates a new cursor with the specified data and offset.
    pub fn with_offset(data: &'a [u8], offset: usize) -> Option<Self> {
        let data = data.get(offset..)?;
        Some(Self { data, offset: 0 })
    }

    /// Creates a new cursor with the specified range of data.
    pub fn with_range(data: &'a [u8], range: Range<usize>) -> Option<Self> {
        let data = data.get(range)?;
        Some(Self { data, offset: 0 })
    }

    /// Returns the underlying buffer for the cursor.
    pub fn data(&self) -> &'a [u8] {
        self.data
    }

    /// Returns the length of the underlying buffer.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if the underlying buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the current offset.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Returns the number of bytes available for reading.
    pub fn remaining(&self) -> usize {
        self.data.len() - self.offset
    }

    /// Sets the offset.
    pub fn set_offset(&mut self, offset: usize) -> Option<()> {
        if offset > self.data.len() {
            return None;
        }
        self.offset = offset;
        Some(())
    }

    /// Returns true if the specified number of bytes can be read.
    pub fn check_range(&self, len: usize) -> bool {
        self.data.len() - self.offset >= len
    }

    /// Returns `None` if the specified number of bytes cannot be read.
    pub fn ensure_range(&self, len: usize) -> Option<()> {
        if self.check_range(len) {
            Some(())
        } else {
            None
        }
    }

    /// Skips the specified number of bytes.
    pub fn skip(&mut self, bytes: usize) -> Option<()> {
        self.set_offset(self.offset.checked_add(bytes)?)
    }

    /// Reads a value of the specified type.
    pub fn read<T: ReadData>(&mut self) -> Option<T> {
        if self.data.len() - self.offset < T::SIZE {
            None
        } else {
            let v = unsafe { T::read_data_unchecked(self.data, self.offset) };
            self.offset += T::SIZE;
            Some(v)
        }
    }

    /// Reads a u8 value.
    #[inline(always)]
    pub fn read_u8(&mut self) -> Option<u8> {
        self.read::<u8>()
    }

    /// Reads a u16 value.
    #[inline(always)]
    pub fn read_u16(&mut self) -> Option<u16> {
        self.read::<u16>()
    }

    /// Reads a u24 value.
    #[inline(always)]
    pub fn read_u24(&mut self) -> Option<u32> {
        self.read::<U24>().map(|x| x.0)
    }

    /// Reads a u32 value.
    #[inline(always)]
    pub fn read_u32(&mut self) -> Option<u32> {
        self.read::<u32>()
    }

    /// Reads an i8 value.
    #[inline(always)]
    pub fn read_i8(&mut self) -> Option<i8> {
        self.read::<i8>()
    }

    /// Reads an i16 value.
    #[inline(always)]
    pub fn read_i16(&mut self) -> Option<i16> {
        self.read::<i16>()
    }

    /// Reads an i32 value.
    #[inline(always)]
    pub fn read_i32(&mut self) -> Option<i32> {
        self.read::<i32>()
    }

    /// Reads a tag value.
    pub fn read_tag(&mut self) -> Option<Tag> {
        Some(Tag(self.read_u32()?))
    }

    /// Reads a 16-bit value and if non-zero, returns the result added to the
    /// specified base.
    pub fn read_offset16(&mut self, base: u32) -> Option<u32> {
        let value = self.read_u16()? as u32;
        if value != 0 {
            Some(value + base)
        } else {
            None
        }
    }

    /// Reads a 32-bit value and if non-zero, returns the result added to the
    /// specified base.
    pub fn read_offset32(&mut self, base: u32) -> Option<u32> {
        let value = self.read_u32()?;
        if value != 0 {
            Some(value + base)
        } else {
            None
        }
    }

    /// Reads a slice of values of the specified type and length.
    pub fn read_slice<T: ReadData>(&mut self, len: usize) -> Option<Slice<'a, T>> {
        let len = len * T::SIZE;
        if !self.check_range(len) {
            return None;
        }
        let arr = Slice::new(&self.data[self.offset..self.offset + len]);
        self.offset += len;
        Some(arr)
    }

    /// Reads a 16-bit length followed by slice of values of the specified type.
    pub fn read_slice16<T: ReadData>(&mut self) -> Option<Slice<'a, T>> {
        let len = self.read_u16()? as usize;
        self.read_slice(len)
    }

    /// Reads a 32-bit length followed by slice of values of the specified type.
    pub fn read_slice32<T: ReadData>(&mut self) -> Option<Slice<'a, T>> {
        let len = self.read_u32()? as usize;
        self.read_slice(len)
    }

    /// Reads a sequence of bytes of the specified length.
    pub fn read_bytes(&mut self, len: usize) -> Option<&'a [u8]> {
        if !self.check_range(len) {
            return None;
        }
        let bytes = &self.data[self.offset..self.offset + len];
        self.offset += len;
        Some(bytes)
    }
}

impl Default for Cursor<'_> {
    fn default() -> Self {
        Self {
            data: &[],
            offset: 0,
        }
    }
}

/// Trait for types that can be read from a big endian encoded buffer.
pub trait ReadData: Sized + Copy + Clone {
    /// The encoded size of `Self`.
    const SIZE: usize = core::mem::size_of::<Self>();

    /// Reads `Self` from the buffer at the specified offset. Returns `None`
    /// if the read would be out of bounds.
    #[inline(always)]
    fn read_data(buf: &[u8], offset: usize) -> Option<Self> {
        let len = buf.len();
        if (offset < len) && ((len - offset) >= Self::SIZE) {
            unsafe { Some(Self::read_data_unchecked(buf, offset)) }
        } else {
            None
        }
    }

    /// Reads `Self` from the buffer at the specified offset without
    /// bounds checking.
    ///
    /// # Safety
    ///
    /// This is safe if `offset + T::SIZE <= buf.len()`.    
    unsafe fn read_data_unchecked(buf: &[u8], offset: usize) -> Self;
}

pub(crate) const USE_UNALIGNED_READS_LE: bool =
    cfg!(any(target_arch = "x86", target_arch = "x86_64"));

impl ReadData for u8 {
    unsafe fn read_data_unchecked(buf: &[u8], offset: usize) -> Self {
        *buf.get_unchecked(offset)
    }
}

impl ReadData for i8 {
    unsafe fn read_data_unchecked(buf: &[u8], offset: usize) -> Self {
        *buf.get_unchecked(offset) as i8
    }
}

impl ReadData for u16 {
    #[inline(always)]
    unsafe fn read_data_unchecked(buf: &[u8], offset: usize) -> Self {
        if USE_UNALIGNED_READS_LE {
            (*(buf.as_ptr().add(offset) as *const u16)).swap_bytes()
        } else {
            (*buf.get_unchecked(offset) as u16) << 8 | *buf.get_unchecked(offset + 1) as u16
        }
    }
}

impl ReadData for i16 {
    unsafe fn read_data_unchecked(buf: &[u8], offset: usize) -> Self {
        u16::read_data_unchecked(buf, offset) as i16
    }
}

impl ReadData for u32 {
    unsafe fn read_data_unchecked(buf: &[u8], offset: usize) -> Self {
        if USE_UNALIGNED_READS_LE {
            (*(buf.as_ptr().add(offset) as *const u32)).swap_bytes()
        } else {
            (*buf.get_unchecked(offset) as u32) << 24
                | (*buf.get_unchecked(offset + 1) as u32) << 16
                | (*buf.get_unchecked(offset + 2) as u32) << 8
                | *buf.get_unchecked(offset + 3) as u32
        }
    }
}

impl ReadData for i32 {
    unsafe fn read_data_unchecked(buf: &[u8], offset: usize) -> Self {
        u32::read_data_unchecked(buf, offset) as i32
    }
}

impl ReadData for u64 {
    unsafe fn read_data_unchecked(buf: &[u8], offset: usize) -> Self {
        if USE_UNALIGNED_READS_LE {
            (*(buf.as_ptr().add(offset) as *const u64)).swap_bytes()
        } else {
            (*buf.get_unchecked(offset) as u64) << 56
                | (*buf.get_unchecked(offset + 1) as u64) << 48
                | (*buf.get_unchecked(offset + 2) as u64) << 40
                | (*buf.get_unchecked(offset + 3) as u64) << 32
                | (*buf.get_unchecked(offset + 4) as u64) << 24
                | (*buf.get_unchecked(offset + 5) as u64) << 16
                | (*buf.get_unchecked(offset + 6) as u64) << 8
                | *buf.get_unchecked(offset + 7) as u64
        }
    }
}

/// Unsigned 24-bit integer.
#[doc(hidden)]
#[derive(Copy, Clone, Debug)]
pub struct U24(pub u32);

impl ReadData for U24 {
    const SIZE: usize = 3;

    unsafe fn read_data_unchecked(buf: &[u8], offset: usize) -> Self {
        Self(
            (*buf.get_unchecked(offset) as u32) << 16
                | (*buf.get_unchecked(offset + 1) as u32) << 8
                | *buf.get_unchecked(offset + 2) as u32,
        )
    }
}

impl ReadData for () {
    unsafe fn read_data_unchecked(_buf: &[u8], _offset: usize) -> Self {}
}

impl ReadData for [u8; 4] {
    unsafe fn read_data_unchecked(buf: &[u8], offset: usize) -> Self {
        [
            *buf.get_unchecked(offset),
            *buf.get_unchecked(offset + 1),
            *buf.get_unchecked(offset + 2),
            *buf.get_unchecked(offset + 3),
        ]
    }
}
