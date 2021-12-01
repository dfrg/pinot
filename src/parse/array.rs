//! Support for types that represent arrays.

/// Trait for types that represent arrays.
pub trait Array: Clone {
    /// The type of items in the list.
    type Item;

    /// Returns the number of items in the list.
    fn len(&self) -> usize;

    /// Returns true if the list is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the item at the specified index.
    fn get(&self, index: usize) -> Option<Self::Item>;
}

/// Iterator over the items in an array.
#[derive(Clone)]
pub struct Iter<'a, T: Array> {
    array: T,
    len: usize,
    pos: usize,
    _phantom: core::marker::PhantomData<&'a ()>,
}

impl<'a, T: Array> Iter<'a, T> {
    pub(crate) fn new(array: T, len: usize) -> Self {
        Self {
            array,
            len,
            pos: 0,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<'a, T: Array> Iterator for Iter<'a, T> {
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.len {
            return None;
        }
        let pos = self.pos;
        self.pos = pos + 1;
        self.array.get(pos)
    }
}
