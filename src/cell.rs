//! A mutable memory location for `Copy` types.

use std::fmt::{self, Debug};
use std::marker::PhantomData;

/// This should be identical to the `Cell` implementation in the standard
/// library, but always require that the internal type implements `Copy`
/// and implements `Copy` itself.
#[derive(PartialEq, Eq, Copy, Clone)]
#[repr(transparent)]
pub struct CopyCell<T> {
    /// Internal value
    value: T,

    /// We trick the compiler to think that `CopyCell` contains a raw pointer,
    /// this way we make sure the `Sync` marker is not implemented and `CopyCell`
    /// cannot be shared across threads!
    _no_sync: PhantomData<*mut T>
}

/// `Sync` is unsafe due to mutability, however `Send` is totally fine!
unsafe impl<T> Send for CopyCell<T> {}

impl<T> CopyCell<T> {
    /// Creates a new `CopyCell` containing the given value.
    pub const fn new(value: T) -> Self {
        CopyCell {
            value,
            _no_sync: PhantomData
        }
    }
}

impl<T: Copy> CopyCell<T> {
    /// Returns a copy of the contained value.
    #[inline]
    pub fn get(&self) -> T {
        self.value
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// This call borrows `CopyCell` mutably, which gives us a compile time
    /// memory safety guarantee.
    #[inline]
    pub fn get_mut<'a>(&'a mut self) -> &'a mut T {
        &mut self.value
    }

    /// Sets the contained value.
    #[inline]
    pub fn set(&self, value: T) {
        use std::ptr::write_volatile;

        // Regular write produces abnormal behavior when running tests in
        // `--release` mode. Reordering writes when the compiler assumes
        // things are immutable is dangerous.
        //
        // We can just cast the pointer from `CopyCell<T>` to `T` because of
        // #[repr(transparent)]
        //
        // This behavior is copied over from the std implementation of
        // the `UnsafeCell`, and it's the best we can do right now in terms
        // of soundness till we get a stable `UnsafeCell` that implements `Copy`.
        unsafe { write_volatile(self as *const CopyCell<T> as *const T as *mut T, value) };
    }
}

impl<T: Debug> Debug for CopyCell<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.value, f)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cell() {
        let cell_a = CopyCell::new(42u64);
        let cell_b = cell_a;  // copy
        let cell_c = &cell_a; // reference

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
        cell_b.set(300);

        assert_eq!(cell_a.get(), 200);
        assert_eq!(cell_b.get(), 300);
        assert_eq!(cell_c.get(), 200);
    }

    #[test]
    fn contain_static_ref() {
        static REF: &(&u64, u64) = &(&0, 0);

        let cell = CopyCell::new(REF);

        assert_eq!(cell.get(), REF);
    }
}
