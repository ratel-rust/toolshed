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
    pub unsafe fn alloc_uninitialized<'a, T: Sized + Copy>(&'a self) -> *mut T {
        let offset = self.require(size_of::<T>());

        self.ptr.get().offset(offset as isize) as *mut T
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

    /// Allocate an `&str` slice onto the arena with an extra null byte at the end.
    /// Can be useful for C-style byte parsers where a null byte is not expected in
    /// a well formatted string.
    pub fn alloc_str_zero_end<'a>(&'a self, val: &str) -> *const u8 {
        let len_with_zero = val.len() + 1;
        let offset = self.offset.get();
        let alignment = size_of::<usize>() - (len_with_zero % size_of::<usize>());
        let cap = offset + len_with_zero + alignment;

        if cap > ARENA_BLOCK {
            let mut vec = Vec::with_capacity(len_with_zero);
            vec.extend_from_slice(val.as_bytes());
            vec.push(0);
            return self.alloc_bytes(vec);
        }

        self.offset.set(cap);

        unsafe {
            use std::ptr::copy_nonoverlapping;

            let ptr = self.ptr.get().offset(offset as isize);
            copy_nonoverlapping(val.as_ptr(), ptr, val.len());
            *ptr.offset(val.len() as isize) = 0;
            ptr
        }
    }

    /// Pushes the `String` as it's own page onto the arena and returns a reference to it.
    /// This does not copy or reallocate the original `String`.
    pub fn alloc_string<'a>(&'a self, val: String) -> &'a str {
        let len = val.len();
        let ptr = self.alloc_bytes(val.into_bytes());

        unsafe {
            use std::str::from_utf8_unchecked;
            use std::slice::from_raw_parts;

            from_utf8_unchecked(from_raw_parts(ptr, len))
        }
    }

    #[inline]
    fn alloc_bytes(&self, val: Vec<u8>) -> *const u8 {
        let ptr = val.as_ptr();

        let mut temp = self.store.replace(Vec::new());
        temp.push(val);
        self.store.replace(temp);

        ptr
    }

    #[inline]
    fn require(&self, size: usize) -> usize {
        let offset = self.offset.get();
        let cap = offset + size;

        if cap > ARENA_BLOCK {
            self.grow();

            self.offset.set(size);
            0
        } else {
            self.offset.set(cap);
            offset
        }
    }

    fn grow(&self) {
        let mut temp = self.store.replace(Vec::new());
        let mut block = Vec::with_capacity(ARENA_BLOCK);
        self.ptr.set(block.as_mut_ptr());
        temp.push(block);
        self.store.replace(temp);
    }

    /// Resets the pointer to the current page of the arena.
    ///
    /// **Using this method is an extremely bad idea!**
    ///
    /// The only case where the use of this method would be justified is
    /// in benchmarks where creation of a structure on the arena is to be
    /// tested without the cost of re-creating the arena itself on every iteration.
    #[inline]
    pub unsafe fn clear(&self) {
        self.offset.set(0)
    }
}
