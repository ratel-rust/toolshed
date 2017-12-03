use std::mem::size_of;
use std::cell::Cell;

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
    offset: Cell<usize>
}

impl Arena {
    /// Create a new arena with a single preallocated 64KiB page
    pub fn new() -> Self {
        let mut store = vec![Vec::with_capacity(ARENA_BLOCK)];
        let ptr = store[0].as_mut_ptr();

        Arena {
            store: Cell::new(store),
            ptr: Cell::new(ptr),
            offset: Cell::new(0)
        }
    }

    /// Put the value onto the page of the arena and return a reference to it.
    #[inline]
    pub fn alloc<'a, T: Sized + Copy>(&'a self, val: T) -> &'a T {
        unsafe {
            let ptr = self.alloc_uninitialized();
            *ptr = val;
            &*ptr
        }
    }

    /// Allocate enough bytes for the type `T`, then return a pointer to the memory.
    /// Memory behind the pointer is uninitialized, can contain garbage and reading
    /// from it is undefined behavior.
    #[inline]
    pub unsafe fn alloc_uninitialized<'a, T: Sized + Copy>(&'a self) -> &'a mut T {
        &mut *(self.require(size_of::<T>()) as *mut T)
    }

    /// Allocate an `&str` slice onto the arena and return a reference to it. This is
    /// useful when the original slice has an undefined lifetime.
    ///
    /// Note: static slices (`&'static str`) can be safely used in place of arena-bound
    ///       slices without having to go through this method.
    pub fn alloc_str<'a>(&'a self, val: &str) -> &'a str {
        let offset = self.offset.get();
        let alignment = size_of::<usize>() - (val.len() % size_of::<usize>());
        let cap = offset + val.len() + alignment;

        if cap > ARENA_BLOCK {
            return self.alloc_string(val.into());
        }

        self.offset.set(cap);

        unsafe {
            use std::ptr::copy_nonoverlapping;
            use std::str::from_utf8_unchecked;
            use std::slice::from_raw_parts;

            let ptr = self.ptr.get().offset(offset as isize);
            copy_nonoverlapping(val.as_ptr(), ptr, val.len());

            from_utf8_unchecked(from_raw_parts(ptr, val.len()))
        }
    }

    /// Allocate an `&str` slice onto the arena as null terminated C-style string.
    /// No checks are performed on the source and whether or not it already contains
    /// any nul bytes. While this does not create any memory issues, it assumes that
    /// the reader of the source can deal with malformed source.
    pub fn alloc_str_with_nul<'a>(&'a self, val: &str) -> *const u8 {
        let len_with_zero = val.len() + 1;
        let ptr = self.require(len_with_zero);

        unsafe {
            use std::ptr::copy_nonoverlapping;

            copy_nonoverlapping(val.as_ptr(), ptr, val.len());
            *ptr.offset(val.len() as isize) = 0;
            ptr
        }
    }

    /// Pushes the `String` as it's own page onto the arena and returns a reference to it.
    /// This does not copy or reallocate the original `String`.
    pub fn alloc_string<'a>(&'a self, val: String) -> &'a str {
        let len = val.len();
        let ptr = self.alloc_vec(val.into_bytes());

        unsafe {
            use std::str::from_utf8_unchecked;
            use std::slice::from_raw_parts;

            from_utf8_unchecked(from_raw_parts(ptr, len))
        }
    }

    #[inline]
    fn alloc_vec(&self, mut val: Vec<u8>) -> *mut u8 {
        let ptr = val.as_mut_ptr();

        let mut temp = self.store.replace(Vec::new());
        temp.push(val);
        self.store.replace(temp);

        ptr
    }

    fn alloc_bytes(&self, size: usize) -> *mut u8 {
        self.alloc_vec(Vec::with_capacity(size))
    }

    #[inline]
    fn require(&self, size: usize) -> *mut u8 {
        // This should be optimized away for size known at compile time.
        if size > ARENA_BLOCK {
            return self.alloc_bytes(size);
        }

        // This should also be optimized away.
        let size = match size % size_of::<usize>() {
            0 => size,
            n => size + n
        };

        let offset = self.offset.get();
        let cap = offset + size;

        if cap > ARENA_BLOCK {
            self.grow();

            self.offset.set(size);
            self.ptr.get()
        } else {
            self.offset.set(cap);
            unsafe { self.ptr.get().offset(offset as isize) }
        }
    }

    fn grow(&self) {
        let ptr = self.alloc_vec(Vec::with_capacity(ARENA_BLOCK));
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
    fn allocate_huge_heap() {
        let arena = Arena::new();

        assert_eq!(arena.alloc(0u64), &0);
        assert_eq!(arena.alloc(42u64), &42);

        unsafe { arena.alloc_uninitialized::<[usize; 1024 * 1024]>() };

        // Still writes to the first page
        assert_eq!(arena.offset.get(), 8 * 2);
        assert_eq!(arena.alloc(0x8000000u64), &0x8000000u64);
        assert_eq!(arena.offset.get(), 8 * 3);

        // For inspecting internals
        let mut arena = arena;

        // However second page has been added
        assert_eq!(arena.store.get_mut().len(), 2);

        // Second page is appropriately large
        assert_eq!(arena.store.get_mut()[1].capacity(), size_of::<usize>() * 1024 * 1024);
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
    fn alloc_str_with_nul() {
        let arena = Arena::new();
        let ptr = arena.alloc_str_with_nul("abcdefghijk");
        let allocated = unsafe { ::std::slice::from_raw_parts(ptr, 12) };

        assert_eq!(arena.offset.get(), 16);
        assert_eq!(allocated, &[b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', 0]);
    }
}
