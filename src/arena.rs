//! Module containing the `Arena` and `Uninitialized` structs. For convenience the
//! `Arena` is exported at the root of the crate.

use std::mem::size_of;
use std::ops::Deref;
use std::cell::Cell;
use std::borrow::Cow;

const ARENA_BLOCK: usize = 64 * 1024;

/// An arena implementation that uses preallocated 64KiB pages for all allocations.
/// If a new allocation were to be pushed over the the boundaries of the page, a
/// new page is internally allocated first, thus this version of the arena can never
/// run out of memory unless the process runs out of heap altogether.
///
/// Allocating a type larger than the page size will result in a new heap allocation
/// just for that type separate from the page mechanism.
pub struct Arena {
    store: Cell<Vec<Vec<u8>>>,
    ptr: Cell<*mut u8>,
    offset: Cell<usize>,
}

/// A pointer to an uninitialized region of memory.
pub struct Uninitialized<'arena, T: 'arena> {
    pointer: &'arena mut T,
}

impl<'arena, T: 'arena> Uninitialized<'arena, T> {
    /// Initialize the memory at the pointer with a given value.
    #[inline]
    pub fn init(self, value: T) -> &'arena mut T {
        *self.pointer = value;

        self.pointer
    }

    /// Get a reference to the pointer without writing to it.
    ///
    /// **Reading from this reference without calling `init` is undefined behavior.**
    #[inline]
    pub unsafe fn as_ref(&self) -> &'arena T {
        &*(self.pointer as *const T)
    }

    /// Convert the `Uninitialized` to a regular mutable reference.
    ///
    /// **Reading from this reference without calling `init` is undefined behavior.**
    #[inline]
    pub unsafe fn as_mut_ref(self) -> &'arena mut T {
        self.pointer
    }

    /// Convert a raw pointer to an `Uninitialized`. This method is unsafe since it can
    /// bind to arbitrary lifetimes.
    #[inline]
    pub unsafe fn from_raw(pointer: *mut T) -> Self {
        Uninitialized {
            pointer: &mut *pointer,
        }
    }
}

impl<'arena, T: 'arena> From<&'arena mut T> for Uninitialized<'arena, T> {
    #[inline]
    fn from(pointer: &'arena mut T) -> Self {
        Uninitialized {
            pointer
        }
    }
}

/// A wrapper around a `str` slice that has an extra `0` byte allocated following
/// its contents.
pub struct NulTermStr<'arena>(&'arena str);

impl<'arena> NulTermStr<'arena> {
    /// Read byte at a given `index`. This does not check for length boundaries,
    /// but is guaranteed to return `0` for `index` equal to the length.
    ///
    /// This can be a very useful optimization when reading a long string one
    /// byte at a time until termination, if checking for `0` can replace what
    /// would otherwise have to be length checks.
    ///
    /// ```rust
    /// # extern crate toolshed;
    /// # use toolshed::Arena;
    /// # fn main() {
    /// let arena = Arena::new();
    /// let str = arena.alloc_nul_term_str("foo");
    ///
    /// // We can safely get the underlying `&str` at any time.
    /// assert_eq!(str.as_ref(), "foo");
    ///
    /// unsafe {
    ///     // First 3 bytes are known to us
    ///     assert_eq!(str.byte_unchecked(0), b'f');
    ///     assert_eq!(str.byte_unchecked(1), b'o');
    ///     assert_eq!(str.byte_unchecked(2), b'o');
    ///
    ///     // Following is safe and guaranteed to be '0'
    ///     assert_eq!(str.byte_unchecked(3), 0);
    ///
    ///     // Reading index 4 would be undefined behavior!
    /// }
    /// # }
    /// ```
    pub unsafe fn byte_unchecked(&self, index: usize) -> u8 {
        *self.0.as_ptr().add(index)
    }
}

impl<'arena> AsRef<str> for NulTermStr<'arena> {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl<'arena> Deref for NulTermStr<'arena> {
    type Target = str;

    fn deref(&self) -> &str {
        self.0
    }
}

impl Arena {
    /// Create a new arena with a single preallocated 64KiB page.
    pub fn new() -> Self {
        let mut store = vec![Vec::with_capacity(ARENA_BLOCK)];
        let ptr = store[0].as_mut_ptr();

        Arena {
            store: Cell::new(store),
            ptr: Cell::new(ptr),
            offset: Cell::new(0),
        }
    }

    /// Put the value onto the page of the arena and return a reference to it.
    #[inline]
    pub fn alloc<'arena, T: Sized + Copy>(&'arena self, value: T) -> &'arena mut T {
        self.alloc_uninitialized().init(value)
    }

    /// Allocate enough bytes for the type `T`, then return an `Uninitialized` pointer to the memory.
    #[inline]
    pub fn alloc_uninitialized<'arena, T: Sized + Copy>(&'arena self) -> Uninitialized<'arena, T> {
        Uninitialized {
            pointer: unsafe { &mut *(self.require(size_of::<T>()) as *mut T) }
        }
    }

    /// Allocate a slice of `T` slice onto the arena and return a reference to it.
    /// This is useful when the original slice has an undefined lifetime.
    ///
    /// Note: static slices (`&'static [T]`) can be safely used in place of arena-bound
    ///       slices without having to go through this method.
    pub fn alloc_slice<'arena, T: Copy>(&'arena self, val: &[T]) -> &'arena [T] {
        let ptr = self.require(val.len() * size_of::<T>()) as *mut T;

        unsafe {
            use std::ptr::copy_nonoverlapping;
            use std::slice::from_raw_parts;

            copy_nonoverlapping(val.as_ptr(), ptr, val.len());
            from_raw_parts(ptr, val.len())
        }
    }

    /// Put a `Vec<T>` on the arena without reallocating.
    pub fn alloc_vec<'arena, T: Copy>(&'arena self, mut val: Vec<T>) -> &'arena [T] {
        use std::{mem, slice};

        let ptr = val.as_mut_ptr();
        let cap = val.capacity();
        let len = val.len();

        mem::forget(val);

        let out = self.alloc_byte_vec(unsafe {
            Vec::from_raw_parts(ptr as _, 0, cap * size_of::<T>())
        });

        unsafe { slice::from_raw_parts(out as _, len) }
    }

    /// Allocate many items at once, avoid allocation for owned values.
    #[inline]
    pub fn alloc_cow<'input, 'arena, T>(&'arena self, vals: Cow<'input, [T]>) -> &'arena [T]
    where
        T: Sized + Copy + 'input,
    {
        match vals {
            Cow::Owned(vec)      => self.alloc_vec(vec),
            Cow::Borrowed(slice) => self.alloc_slice(slice),
        }
    }

    /// Allocate an `&str` slice onto the arena and return a reference to it. This is
    /// useful when the original slice has an undefined lifetime.
    ///
    /// Note: static slices (`&'static str`) can be safely used in place of arena-bound
    ///       slices without having to go through this method.
    pub fn alloc_str<'arena>(&'arena self, val: &str) -> &'arena str {
        unsafe {
            use std::str::from_utf8_unchecked;

            from_utf8_unchecked(self.alloc_slice(val.as_bytes()))
        }
    }

    /// Allocate an `&str` slice onto the arena as null terminated C-style string.
    /// No checks are performed on the source and whether or not it already contains
    /// any nul bytes. While this does not create any memory issues, it assumes that
    /// the reader of the source can deal with malformed source.
    pub fn alloc_nul_term_str<'arena>(&'arena self, val: &str) -> NulTermStr {
        let len_with_zero = val.len() + 1;
        let ptr = self.require(len_with_zero);

        unsafe {
            use std::ptr::copy_nonoverlapping;
            use std::slice::from_raw_parts;
            use std::str::from_utf8_unchecked;

            copy_nonoverlapping(val.as_ptr(), ptr, val.len());
            *ptr.add(val.len()) = 0;

            NulTermStr(from_utf8_unchecked(from_raw_parts(ptr, val.len())))
        }
    }

    /// Pushes the `String` as it's own page onto the arena and returns a reference to it.
    /// This does not copy or reallocate the original `String`.
    pub fn alloc_string<'arena>(&'arena self, val: String) -> &'arena str {
        let len = val.len();
        let ptr = self.alloc_byte_vec(val.into_bytes());

        unsafe {
            use std::str::from_utf8_unchecked;
            use std::slice::from_raw_parts;

            from_utf8_unchecked(from_raw_parts(ptr, len))
        }
    }

    #[inline]
    fn alloc_byte_vec(&self, mut val: Vec<u8>) -> *mut u8 {
        let ptr = val.as_mut_ptr();

        let mut temp = self.store.replace(Vec::new());
        temp.push(val);
        self.store.replace(temp);

        ptr
    }

    fn alloc_bytes(&self, size: usize) -> *mut u8 {
        self.alloc_byte_vec(Vec::with_capacity(size))
    }

    #[inline]
    fn require(&self, size: usize) -> *mut u8 {
        // This should be optimized away for size known at compile time.
        if size > ARENA_BLOCK {
            return self.alloc_bytes(size);
        }

        let size = match size % size_of::<usize>() {
            0 => size,
            n => size + (size_of::<usize>() - n),
        };

        let offset = self.offset.get();
        let cap = offset + size;

        if cap > ARENA_BLOCK {
            self.grow();

            self.offset.set(size);
            self.ptr.get()
        } else {
            self.offset.set(cap);
            unsafe { self.ptr.get().add(offset) }
        }
    }

    fn grow(&self) {
        let ptr = self.alloc_byte_vec(Vec::with_capacity(ARENA_BLOCK));
        self.ptr.set(ptr);
    }

    /// Resets the pointer to the current page of the arena.
    ///
    /// **Using this method is an extremely bad idea!**
    ///
    /// The only case where the use of this method would be justified is
    /// in benchmarks where creation of a structure on the arena is to be
    /// tested without the cost of re-creating the arena itself on every iteration.
    #[doc(hidden)]
    #[inline]
    pub unsafe fn clear(&self) {
        self.reset_to(0)
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn offset(&self) -> usize {
        self.offset.get()
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn reset_to(&self, offset: usize) {
        self.offset.set(offset)
    }
}

/// Akin to `CopyCell`: `Sync` is unsafe but `Send` is totally fine!
unsafe impl Send for Arena {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn allocate_some_stuff() {
        let arena = Arena::new();

        assert_eq!(arena.alloc(0u64), &0);
        assert_eq!(arena.alloc(42u64), &42);
        assert_eq!(arena.alloc(0x8000000u64), &0x8000000u64);

        assert_eq!(arena.offset.get(), 8 * 3);

        // For inspecting internals
        let mut arena = arena;

        assert_eq!(arena.store.get_mut().len(), 1);
    }

    #[test]
    fn allocate_some_vecs() {
        let arena = Arena::new();

        let vecs = vec![vec![1u64, 2, 3, 4], vec![7; ARENA_BLOCK * 2], vec![]];

        for vec in vecs {
            assert_eq!(arena.alloc_vec(vec.clone()), &vec[..]);
        }
    }

    #[test]
    fn allocate_some_cows() {
        let arena = Arena::new();

        let vecs = vec![vec![1u64, 2, 3, 4], vec![7; ARENA_BLOCK * 2], vec![]];

        for vec in vecs {
            assert_eq!(arena.alloc_cow(vec.clone().into()), &vec[..]);
        }
    }

    #[test]
    fn allocate_huge_heap() {
        let arena = Arena::new();

        assert_eq!(arena.alloc(0u64), &0);
        assert_eq!(arena.alloc(42u64), &42);

        arena.alloc_uninitialized::<[usize; 1024 * 1024]>();

        // Still writes to the first page
        assert_eq!(arena.offset.get(), 8 * 2);
        assert_eq!(arena.alloc(0x8000000u64), &0x8000000u64);
        assert_eq!(arena.offset.get(), 8 * 3);

        // For inspecting internals
        let mut arena = arena;

        // However second page has been added
        assert_eq!(arena.store.get_mut().len(), 2);

        // Second page is appropriately large
        assert_eq!(
            arena.store.get_mut()[1].capacity(),
            size_of::<usize>() * 1024 * 1024
        );
    }

    #[test]
    fn alloc_slice() {
        let arena = Arena::new();

        assert_eq!(arena.alloc_slice(&[10u16, 20u16]), &[10u16, 20u16][..]);
        assert_eq!(arena.offset.get(), 8);
    }

    #[test]
    fn aligns_slice_allocs() {
        let arena = Arena::new();

        assert_eq!(arena.alloc_slice(b"foo"), b"foo");
        assert_eq!(arena.offset.get(), 8);

        assert_eq!(arena.alloc_slice(b"doge to the moon!"), b"doge to the moon!");
        assert_eq!(arena.offset.get(), 32);
    }

    #[test]
    fn aligns_str_allocs() {
        let arena = Arena::new();

        assert_eq!(arena.alloc_str("foo"), "foo");
        assert_eq!(arena.offset.get(), 8);

        assert_eq!(arena.alloc_str("doge to the moon!"), "doge to the moon!");
        assert_eq!(arena.offset.get(), 32);
    }

    #[test]
    fn alloc_nul_term_str() {
        let arena = Arena::new();
        let nts = arena.alloc_nul_term_str("abcdefghijk");
        let allocated = unsafe { ::std::slice::from_raw_parts(nts.as_ptr(), 12) };

        assert_eq!(arena.offset.get(), 16);
        assert_eq!(
            allocated,
            &[
                b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', 0
            ]
        );
    }
}
