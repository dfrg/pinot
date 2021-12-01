//! Sequence of values in a big endian encoded buffer.

use super::ReadData;
use core::ops::Range;

/// Lazy slice wrapping a byte buffer over a sequence of values that implement
/// [`ReadData`].
#[derive(Copy, Clone)]
pub struct Slice<'a, T: ReadData> {
    data: &'a [u8],
    len: usize,
    _p: core::marker::PhantomData<T>,
}

impl<'a, T: ReadData> Slice<'a, T> {
    pub(crate) fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            len: data.len() / T::SIZE,
            _p: core::marker::PhantomData {},
        }
    }

    /// Returns the length of the slice.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the slice is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the element at the specified index.
    pub fn get(&self, index: usize) -> Option<T> {
        if index >= self.len {
            None
        } else {
            unsafe { Some(T::read_data_unchecked(self.data, index * T::SIZE)) }
        }
    }

    /// Returns the element at the specified index, or some value if the index
    /// is out of bounds.
    pub fn get_or(&self, index: usize, or: T) -> T {
        if index >= self.len {
            or
        } else {
            unsafe { T::read_data_unchecked(self.data, index * T::SIZE) }
        }
    }

    /// Returns the element at the specified index without bounds checking.
    ///
    /// # Safety
    /// This is safe if `index < self.len()`.
    pub unsafe fn get_unchecked(&self, index: usize) -> T {
        T::read_data_unchecked(self.data, index * T::SIZE)
    }

    /// Performs a binary search over the slice using the specified comparator
    /// function. Returns the index and value of the element on success, or
    /// `None` if a match was not found.
    pub fn binary_search_by<F>(&self, mut f: F) -> Option<(usize, T)>
    where
        F: FnMut(&T) -> core::cmp::Ordering,
    {
        // Taken from Rust core library.
        use core::cmp::Ordering::*;
        let mut size = self.len;
        if size == 0 {
            return None;
        }
        let mut base = 0usize;
        while size > 1 {
            let half = size / 2;
            let mid = base + half;
            // SAFETY: the call is made safe by the following inconstants:
            // - `mid >= 0`: by definition
            // - `mid < size`: `mid = size / 2 + size / 4 + size / 8 ...`
            let element = unsafe { self.get_unchecked(mid) };
            base = match f(&element) {
                Greater => base,
                Less => mid,
                Equal => return Some((mid, element)),
            };
            size -= half;
        }
        None
    }

    /// Returns an iterator over the elements of the slice.
    pub fn iter(&self) -> Iter<'a, T> {
        Iter {
            inner: *self,
            range: 0..self.len,
        }
    }
}

impl<T: ReadData> Default for Slice<'_, T> {
    fn default() -> Self {
        Slice::new(&[])
    }
}

impl<T> core::fmt::Debug for Slice<'_, T>
where
    T: core::fmt::Debug + ReadData,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T: ReadData + PartialEq> PartialEq for Slice<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        if self.len() == other.len() {
            self.iter().eq(other.iter())
        } else {
            false
        }
    }
}

impl<T: ReadData + PartialEq> Eq for Slice<'_, T> {}

impl<T: ReadData + PartialOrd> PartialOrd for Slice<'_, T> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}

impl<T: ReadData + Ord> Ord for Slice<'_, T> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.iter().cmp(other.iter())
    }
}

/// Iterator over the elements of a slice.
#[derive(Clone)]
pub struct Iter<'a, T: ReadData> {
    inner: Slice<'a, T>,
    range: Range<usize>,
}

impl<'a, T: ReadData + 'a> Iterator for Iter<'a, T> {
    type Item = T;

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.range.len();
        (remaining, Some(remaining))
    }

    fn next(&mut self) -> Option<T> {
        let index = self.range.next()?;
        unsafe { Some(self.inner.get_unchecked(index)) }
    }
}

impl<'a, T: ReadData + 'a> ExactSizeIterator for Iter<'a, T> {
    fn len(&self) -> usize {
        self.range.len()
    }
}

impl<'a, T: ReadData + 'a> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let index = self.range.next_back()?;
        unsafe { Some(self.inner.get_unchecked(index)) }
    }
}

impl<'a, T: ReadData + 'a> IntoIterator for Slice<'a, T> {
    type IntoIter = Iter<'a, T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T: ReadData + 'a> IntoIterator for &'_ Slice<'a, T> {
    type IntoIter = Iter<'a, T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
