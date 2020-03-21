//! Module containing the `Arena` and `Uninitialized` structs. For convenience the
//! `Arena` is exported at the root of the crate.

use std::mem;
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
pub struct Uninitialized<'a, T: Copy> {
    pointer: &'a mut MaybeUninit<T>,
}

/// Almost a copy of https://github.com/rust-lang/rust/issues/53491
union MaybeUninit<T: Copy> {
    value: T,
    _uninit: (),
}

impl<'a, T: Copy> Uninitialized<'a, T> {
    /// Initialize the memory at the pointer with a given value.
    #[inline]
    pub fn init(self, value: T) -> &'a mut T {
        unsafe {
            self.pointer.value = value;
            &mut self.pointer.value
        }
    }

    /// Get a reference to the pointer without writing to it.
    ///
    /// **Calling this method without calling `init` is undefined behavior.**
    #[inline]
    pub unsafe fn as_ref(&self) -> &'a T {
        &*(&self.pointer.value as *const T)
    }

    /// Convert the `Uninitialized` to a regular mutable reference.
    ///
    /// **Calling this method without calling `init` is undefined behavior.**
    #[inline]
    pub unsafe fn as_mut_ref(self) -> &'a mut T {
        &mut self.pointer.value
    }

    /// Convert a raw pointer to an `Uninitialized`. This method is unsafe since it can
    /// bind to arbitrary lifetimes.
    #[inline]
    pub unsafe fn from_raw(pointer: *mut T) -> Self {
        Uninitialized {
            pointer: &mut *(pointer as *mut MaybeUninit<T>),
        }
    }
}

impl<'a, T: Copy> From<&'a mut T> for Uninitialized<'a, T> {
    #[inline]
    fn from(pointer: &'a mut T) -> Self {
        unsafe { Self::from_raw(pointer) }
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
            offset: Cell::new(ARENA_BLOCK),
        }
    }

    /// Put the value onto the page of the arena and return a reference to it.
    #[inline]
    pub fn alloc<'a, T: Sized + Copy>(&'a self, value: T) -> &'a mut T {
        self.alloc_uninitialized().init(value)
    }

    /// Allocate enough bytes for the type `T`, then return an `Uninitialized` pointer to the memory.
    #[inline]
    pub fn alloc_uninitialized<'a, T: Sized + Copy>(&'a self) -> Uninitialized<'a, T> {
        Uninitialized {
            pointer: unsafe { &mut *(self.require::<T>(mem::size_of::<T>()) as *mut MaybeUninit<T>) },
        }
    }

    /// Allocate a slice of `T` slice onto the arena and return a reference to it.
    /// This is useful when the original slice has an undefined lifetime.
    ///
    /// Note: static slices (`&'static [T]`) can be safely used in place of arena-bound
    ///       slices without having to go through this method.
    pub fn alloc_slice<'a, T: Copy>(&'a self, val: &[T]) -> &'a [T] {
        let ptr = self.require::<T>(val.len() * mem::size_of::<T>()) as *mut T;

        unsafe {
            use std::ptr::copy_nonoverlapping;
            use std::slice::from_raw_parts;

            copy_nonoverlapping(val.as_ptr(), ptr, val.len());
            from_raw_parts(ptr, val.len())
        }
    }

    /// Allocate a statically-sized but lazily-generated slice `[T]` out of an iterator
    /// This is useful if you're going to make a slice of something and put it on the arena,
    /// but you don't want to make an allocation first just to have something to copy in.
    ///
    /// The slice will be at maximum length `n`, further elements of the iterator ignored and not evaluated.
    /// If the iterator yields less than `n` elements, a shorter slice will simply be returned.
    pub fn alloc_lazy_slice<'a, T, I: Iterator<Item=T>>(&'a self, vals: I, n: usize) -> &'a [T] {
      // Grab space for `n` elements even if it may turn out we have to walk it back
      let ptr = self.require::<T>(n * mem::size_of::<T>()) as *mut T;
      let mut i: usize = 0;

      unsafe {
        use std::slice::from_raw_parts;

        for val in vals.take(n) {
          *ptr.offset(i as isize) = val;
          i += 1;
        }
        // Now fix the slice length and arena offset
        let diff = n - i;
        self.reset_to( self.offset() - diff * mem::size_of::<T>() );
        from_raw_parts(ptr, i)
      }
    }

    /// Put a `Vec<T>` on the arena without reallocating.
    pub fn alloc_vec<'a, T: Copy>(&'a self, mut val: Vec<T>) -> &'a [T] {
        use std::slice;

        let ptr = val.as_mut_ptr();
        let cap = val.capacity();
        let len = val.len();

        mem::forget(val);

        let out = self.alloc_byte_vec(unsafe {
            Vec::from_raw_parts(ptr as _, 0, cap * mem::size_of::<T>())
        });

        unsafe { slice::from_raw_parts(out as _, len) }
    }

    /// Allocate many items at once, avoid allocation for owned values.
    #[inline]
    pub fn alloc_cow<'input, 'a, T>(&'a self, vals: Cow<'input, [T]>) -> &'a [T]
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
    pub fn alloc_str<'a>(&'a self, val: &str) -> &'a str {
        unsafe {
            use std::str::from_utf8_unchecked;

            from_utf8_unchecked(self.alloc_slice(val.as_bytes()))
        }
    }

    #[inline]
    pub fn builder<'a>(&'a mut self) -> ArenaStr<'a> {
        ArenaStr {
            len: 0,
            arena: self,
        }
    }

    /// Pushes the `String` as it's own page onto the arena and returns a reference to it.
    /// This does not copy or reallocate the original `String`.
    pub fn alloc_string<'a>(&'a self, val: String) -> &'a str {
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
    fn require<T>(&self, size: usize) -> *mut u8 {
        if size > ARENA_BLOCK {
            return self.alloc_bytes(size);
        }

        let size = match self.offset.get() % mem::align_of::<T>() {
            0 => size,
            n => size + n,
        };

        if let Some(offset) = self.offset.get().checked_sub(size) {
            self.offset.set(offset);
            unsafe { self.ptr.get().add(offset) }
        } else {
            self.grow();

            self.offset.set(ARENA_BLOCK - size);
            unsafe { self.ptr.get().add(self.offset.get()) }
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
        ARENA_BLOCK - self.offset.get()
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn reset_to(&self, offset: usize) {
        self.offset.set(ARENA_BLOCK - offset)
    }
}

pub struct ArenaStr<'a> {
    len: usize,
    arena: &'a mut Arena,
}

impl<'a> ArenaStr<'a> {
    pub fn push_str(&mut self, slice: &str) {

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

        assert_eq!(arena.offset.get(), ARENA_BLOCK - 8 * 3);

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
        assert_eq!(arena.offset.get(), ARENA_BLOCK - 8 * 2);
        assert_eq!(arena.alloc(0x8000000u64), &0x8000000u64);
        assert_eq!(arena.offset.get(), ARENA_BLOCK - 8 * 3);

        // For inspecting internals
        let mut arena = arena;

        // However second page has been added
        assert_eq!(arena.store.get_mut().len(), 2);

        // Second page is appropriately large
        assert_eq!(
            arena.store.get_mut()[1].capacity(),
            mem::size_of::<usize>() * 1024 * 1024
        );
    }

    #[test]
    fn alloc_slice() {
        let arena = Arena::new();

        assert_eq!(arena.alloc_slice(&[10u16, 20u16]), &[10u16, 20u16][..]);
        assert_eq!(arena.offset.get(), ARENA_BLOCK - 4);
    }

    #[test]
    fn alloc_lazy_slices() {
      let arena = Arena::new();
      let nums: [u32; 6] = [1, 2, 3, 4, 5, 1000];
      let big_nums: [u32; 6] = [100, 200, 300, 400, 500, 1050];

      // Put the whole array in the arena
      let all_nums = arena.alloc_lazy_slice(nums.iter().map(|x| *x), 6);
      // Truncate it using the `n` argument
      let trunc_nums = arena.alloc_lazy_slice(big_nums.iter().map(|x| *x), 3);
      // Put a whole array of half the nums in the arena
      let half_nums = arena.alloc_lazy_slice(nums[0..3].iter().map(|x| *x), 6);

      assert!(nums.iter().eq(all_nums.iter()));
      assert!(nums[0..3].iter().eq(half_nums.iter()));
      assert!(big_nums[0..3].iter().eq(trunc_nums.iter()));
    }

    #[test]
    fn aligns_slice_allocs() {
        let arena = Arena::new();

        assert_eq!(arena.alloc_slice(b"foo"), b"foo");
        assert_eq!(arena.offset.get(), ARENA_BLOCK - 3);

        assert_eq!(arena.alloc(42u64), &42);
        assert_eq!(arena.offset.get(), ARENA_BLOCK - 16);

        assert_eq!(arena.alloc_slice(b"doge to the moon!"), b"doge to the moon!");
        assert_eq!(arena.offset.get(), ARENA_BLOCK - 33);
    }

    #[test]
    fn aligns_str_allocs() {
        let arena = Arena::new();

        assert_eq!(arena.alloc_str("foo"), "foo");
        assert_eq!(arena.offset.get(), ARENA_BLOCK - 3);

        assert_eq!(arena.alloc(42u64), &42);
        assert_eq!(arena.offset.get(), ARENA_BLOCK - 16);

        assert_eq!(arena.alloc_str("doge to the moon!"), "doge to the moon!");
        assert_eq!(arena.offset.get(), ARENA_BLOCK - 33);
    }
}
