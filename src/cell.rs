//! A mutable memory location for `Copy` types.

use std::fmt::{self, Debug};

/// This should be identical to the `Cell` implementation in the standard
/// library, but always require that the internal type implements `Copy`
/// and implements `Copy` itself.
#[derive(PartialEq)]
pub struct CopyCell<T: Copy> {
    value: T
}

impl<T: Copy + Eq> Eq for CopyCell<T> {}

impl<T: Copy> CopyCell<T> {
    /// Creates a new `CopyCell` containing the given value.
    #[inline]
    pub fn new(value: T) -> Self {
        CopyCell {
            value
        }
    }

    #[inline]
    fn mut_ptr(&self) -> *mut T {
        &self.value as *const T as *mut T
    }

    /// Returns a copy of the contained value.
    #[inline]
    pub fn get(&self) -> T {
        unsafe {
            *self.mut_ptr()
        }
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// This call borrows `CopyCell` mutably (at compile-time) which guarantees that we possess the only reference.
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        unsafe {
            &mut *self.mut_ptr()
        }
    }

    /// Sets the contained value.
    #[inline]
    pub fn set(&self, value: T) {
        let ptr = unsafe { &mut *self.mut_ptr() };
        *ptr = value;
    }
}

impl<T: Copy> Clone for CopyCell<T> {
    #[inline]
    fn clone(&self) -> CopyCell<T> {
        CopyCell::new(self.get())
    }
}

impl<T: Copy> Copy for CopyCell<T> { }

impl<T: Debug + Copy> Debug for CopyCell<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.get(), f)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cell() {
        let cell_a = CopyCell::new(42u64);
        let mut cell_b = cell_a; // copy
        let cell_c = &cell_a;    // reference

        assert_eq!(cell_a.get(), 42);
        assert_eq!(cell_b.get(), 42);
        assert_eq!(cell_c.get(), 42);

        // Only affects the copy
        cell_b.set(100);

        assert_eq!(cell_a.get(), 42);
        assert_eq!(cell_b.get(), 100);
        assert_eq!(cell_c.get(), 42);

        // Affects a since c is a ref
        cell_c.set(200);

        assert_eq!(cell_a.get(), 200);
        assert_eq!(cell_b.get(), 100);
        assert_eq!(cell_c.get(), 200);

        // Again, only affects the copy
        *cell_b.get_mut() = 300;

        assert_eq!(cell_a.get(), 200);
        assert_eq!(cell_b.get(), 300);
        assert_eq!(cell_c.get(), 200);
    }
}
