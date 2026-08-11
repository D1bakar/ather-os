//! Minimal growable vector for `#![no_std]` environments with an allocator.

use alloc::vec::Vec as AllocVec;
use core::iter::FromIterator;
use core::ops::{Deref, DerefMut};

/// A growable contiguous buffer backed by the global allocator.
///
/// This is an intentionally small stub: it forwards to `alloc::vec::Vec` while
/// the kernel heap and collection policy mature in later milestones.
pub struct Vec<T> {
    inner: AllocVec<T>,
}

impl<T> Vec<T> {
    /// Creates an empty vector.
    pub fn new() -> Self {
        Self { inner: AllocVec::new() }
    }

    /// Creates a vector with the given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self { inner: AllocVec::with_capacity(capacity) }
    }

    /// Returns the number of elements.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if the vector contains no elements.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Appends an element and returns its index.
    pub fn push(&mut self, value: T) -> usize {
        let index = self.inner.len();
        self.inner.push(value);
        index
    }

    /// Removes and returns the last element, if any.
    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop()
    }

    /// Returns a shared reference to the element at `index`, or `None`.
    pub fn get(&self, index: usize) -> Option<&T> {
        self.inner.get(index)
    }

    /// Returns a mutable reference to the element at `index`, or `None`.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.inner.get_mut(index)
    }

    /// Clears the vector, dropping all elements.
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Extends the vector with the contents of `slice`.
    pub fn extend_from_slice(&mut self, slice: &[T])
    where
        T: Clone,
    {
        self.inner.extend_from_slice(slice);
    }

    /// Returns a slice of all elements.
    pub fn as_slice(&self) -> &[T] {
        self.inner.as_slice()
    }
}

impl<T> Default for Vec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Deref for Vec<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        self.inner.as_slice()
    }
}

impl<T> DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        self.inner.as_mut_slice()
    }
}

impl<T> FromIterator<T> for Vec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self { inner: AllocVec::from_iter(iter) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop_and_index() {
        let mut v = Vec::new();
        assert!(v.is_empty());
        assert_eq!(v.push(10), 0);
        assert_eq!(v.push(20), 1);
        assert_eq!(v.len(), 2);
        assert_eq!(v.get(0), Some(&10));
        assert_eq!(v.pop(), Some(20));
        assert_eq!(v.as_slice(), &[10]);
    }

    #[test]
    fn from_iterator() {
        let v: Vec<_> = [1, 2, 3].into_iter().collect();
        assert_eq!(v.len(), 3);
        assert_eq!(v[1], 2);
    }

    #[test]
    fn clear_drops_elements() {
        let mut v = Vec::from_iter([1, 2, 3]);
        v.clear();
        assert!(v.is_empty());
    }
}
