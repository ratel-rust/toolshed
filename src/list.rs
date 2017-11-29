//! A linked list and auxiliary types that can be used with the `Arena`.

use Arena;
use cell::CopyCell;

const PAGE_CAP: usize = 16;

/// A single-ended linked list.
#[derive(Clone)]
pub struct List<'arena, T: 'arena + Copy> {
    root: CopyCell<Option<&'arena ListPage<'arena, T>>>,
}

impl<'arena, T: 'arena + Copy> List<'arena, T> {
    /// Create a new empty `List`.
    #[inline]
    pub fn empty() -> Self {
        List {
            root: CopyCell::new(None)
        }
    }

    /// Returns an iterator over the items in the list.
    #[inline]
    pub fn iter(&self) -> ListIter<'arena, T> {
        ListIter {
            index: 0,
            page: self.root.get().unwrap_or_else(|| unsafe {
                empty_page()
            })
        }
    }

    /// Checks if the list is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.root.get().is_none()
    }

    /// Returns the first element if, and only if, the list contains
    /// just that single element.
    #[inline]
    pub fn only_element(&self) -> Option<&'arena T> {
        match self.root.get() {
            Some(list) if list.len() == 1 => Some(list.value(0).get_ref()),
            _                             => None,
        }
    }

    /// Turns the list into an empty list.
    ///
    /// Internally, all this method does is removing the reference to the
    /// first item on the list.
    #[inline]
    pub fn clear(&self) {
        self.root.set(None);
    }

    /// Returns an `UnsafeList` for the current `List`. While this function is
    /// safe itself, using `UnsafeList` might lead to undefined behavior.
    #[inline]
    pub fn into_unsafe(self) -> UnsafeList {
        UnsafeList {
            root: match self.root.get() {
                Some(ptr) => ptr as *const ListPage<'arena, T> as usize,
                None      => 0
            }
        }
    }

    /// Create a single-element list from the given value.
    #[inline]
    pub fn from(arena: &'arena Arena, value: T) -> List<'arena, T> {
        let page = ListPage::new(arena);

        unsafe { page.values.get_unchecked(0) }.set(value);
        page.length.set(1);

        List {
            root: CopyCell::new(Some(page))
        }
    }

    /// Create a list from an iterator of items.
    pub fn from_iter<I>(arena: &'arena Arena, source: I) -> List<'arena, T> where
        I: IntoIterator<Item = T>
    {
        let mut builder = ListBuilder::new(arena);

        for item in source {
            builder.push(item);
        }

        builder.into_list()
    }
}


impl<'arena, T> IntoIterator for List<'arena, T>
where
    T: 'arena + Copy,
{
    type Item = &'arena T;
    type IntoIter = ListIter<'arena, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, 'arena, T> IntoIterator for &'a List<'arena, T>
where
    T: 'arena + Copy,
{
    type Item = &'arena T;
    type IntoIter = ListIter<'arena, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'arena, T: Copy> Copy for List<'arena, T> { }

// Get a pointer to a ListPage of an arbitrary type T. This is meant to be used
// as a read-only value, and only for the 0-length.
//
// The actual allocated memory will likely not match the type T. The **only**
// assumption the user of this function can make is that the length is set,
// and that it is set to `0`. Anything else is likely a SEGFAULT or UD.
unsafe fn empty_page<'a, T>() -> &'a ListPage<'a, T>
where
    T: 'a + Copy,
{
    // This static stores a pointer, usize makes things easier
    static mut EMPTY_PAGE: Option<usize> = None;

    let ptr = match EMPTY_PAGE {
        None => {
            // Allocate 3 0-initialized words on the heap, get pointer as usize
            let ptr = Box::into_raw(Box::new([0usize; 3])) as usize;

            // Poor man's lazy_static (and no, lazy_static wouldn't work, because !Sync)
            EMPTY_PAGE = Some(ptr);
            ptr
        },
        Some(ptr) => ptr
    };

    // We don't know what the size of T is. The catch is that we only need to read length,
    // which with the #[repr(C)] should always be the first word!
    &*(ptr as *const ListPage<'a, T>)
}

#[derive(Debug, PartialEq, Clone)]
#[repr(C)]
struct ListPage<'arena, T: 'arena + Copy> {
    length: CopyCell<usize>,
    values: [CopyCell<T>; PAGE_CAP],
    next: CopyCell<&'arena ListPage<'arena, T>>,
}

impl<'arena, T: Copy> Copy for ListPage<'arena, T> {}

impl<'arena, T: Copy> ListPage<'arena, T> {
    #[inline]
    fn new(arena: &'arena Arena) -> &'arena ListPage<'arena, T> {
        let page = unsafe { arena.alloc_uninitialized::<ListPage<'arena, T>>() };
        page.length = CopyCell::new(0);
        page
    }

    #[inline]
    fn has_capacity(&self) -> bool {
        self.len() != PAGE_CAP
    }

    #[inline]
    fn push(&self, value: T) {
        debug_assert!(self.has_capacity());

        self.value(self.len()).set(value);
        self.length.set(self.len() + 1);
    }

    #[inline]
    fn value(&self, index: usize) -> &CopyCell<T> {
        unsafe { &self.values.get_unchecked(index) }
    }

    #[inline]
    fn len(&self) -> usize {
        self.length.get()
    }
}

/// A builder struct that allows to push elements onto the end of the list.
pub struct ListBuilder<'arena, T>
where
    T: 'arena + Copy,
{
    arena: &'arena Arena,
    first: &'arena ListPage<'arena, T>,
    last: &'arena ListPage<'arena, T>,
}

impl<'arena, T> ListBuilder<'arena, T>
where
    T: 'arena + Copy,
{
    /// Create a new builder.
    #[inline]
    pub fn new(arena: &'arena Arena) -> Self {
        let page = ListPage::new(arena);

        ListBuilder {
            arena,
            first: page,
            last: page,
        }
    }

    #[inline]
    fn get_page(&mut self) -> &'arena ListPage<'arena, T> {
        if !self.last.has_capacity() {
            let page = ListPage::new(self.arena);

            // Set the capacity at it's limit
            self.last.length.set(self.last.len() + 1);
            self.last.next.set(page);

            self.last = page;
        }

        self.last
    }

    /// Push a new item at the end of the `List`.
    #[inline]
    pub fn push(&mut self, item: T) {
        self.get_page().push(item)
    }

    /// Consume the builder and return a `List`.
    #[inline]
    pub fn into_list(self) -> List<'arena, T> {
        let page = match self.first.len() {
            0 => None,
            _ => Some(self.first),
        };

        List {
            root: CopyCell::new(page)
        }
    }
}

/// Unsafe variant of the `List` that erases any lifetime information.
#[derive(Debug, Clone, Copy)]
pub struct UnsafeList {
    root: usize
}

impl UnsafeList {
    /// Converts the `UnsafeList` into a regular `List`. Using this with
    /// incorrect lifetimes of after the original arena has been dropped
    /// will lead to undefined behavior. Use with extreme care.
    pub unsafe fn into_list<'arena, T>(self) -> List<'arena, T>
    where
        T: 'arena + Copy,
    {
        List {
            root: CopyCell::new(match self.root {
                0   => None,
                ptr => Some(&*(ptr as *const ListPage<'arena, T>))
            })
        }
    }
}

/// An iterator over the items in the list.
pub struct ListIter<'arena, T>
where
    T: 'arena + Copy,
{
    index: usize,
    page: &'arena ListPage<'arena, T>
}

impl<'arena, T> Iterator for ListIter<'arena, T>
where
    T: 'arena + Copy,
{
    type Item = &'arena T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Iterate on the current page
        if self.index != self.page.len() {
            let value = self.page.value(self.index).get_ref();

            self.index += 1;

            return Some(value);
        }

        // Jump to the next page
        if self.page.len() == PAGE_CAP {
            self.page = self.page.next.get();
            self.index = 1;

            return Some(self.page.value(0).get_ref())
        }

        // This is the end
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn builder() {
        let arena = Arena::new();
        let mut builder = ListBuilder::new(&arena);

        builder.push(10);
        builder.push(20);
        builder.push(30);

        let list = builder.into_list();

        assert!(list.iter().eq([10, 20, 30].iter()));
    }

    #[test]
    fn empty_iterator() {
        // Test with a big-ish type
        let list = List::<(usize, f64, u64)>::empty();
        let mut iter = list.iter();

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn unsafe_empty_page() {
        let page = unsafe { empty_page::<ListPage<(usize, f64, u64)>>() };

        assert_eq!(page.len(), 0);
    }

    #[test]
    fn from_value() {
        let arena = Arena::new();
        let list = List::from(&arena, 10);

        assert!(list.iter().eq([10].iter()));
    }

    #[test]
    fn from_iter() {
        let arena = Arena::new();
        let list = List::from_iter(&arena, [10, 20, 30].iter().cloned());

        assert!(list.iter().eq([10, 20, 30].iter()));
    }

    #[test]
    fn only_element() {
        let arena = Arena::new();
        let list = List::from(&arena, 42);

        assert_eq!(list.only_element(), Some(&42));

        let list = List::from_iter(&arena, [42, 42].iter().cloned());

        assert_eq!(list.only_element(), None);
    }

    #[test]
    fn empty_unsafe_list() {
        let list: List<usize> = List::empty();
        let raw = list.into_unsafe();

        assert_eq!(raw.root, 0);

        let list: List<usize> = unsafe { raw.into_list() };

        assert_eq!(list.is_empty(), true);
    }

    #[test]
    fn unsafe_list() {
        let arena = Arena::new();

        {
            let list = List::from(&arena, 42usize);

            drop(list);

            let raw = list.into_unsafe();

            assert_ne!(raw.root, 0);

            let list: List<usize> = unsafe { raw.into_list() };

            assert_eq!(list.only_element(), Some(&42));

            // Let's be absolutely sure...
            drop(list);
        }

        // ...that things are dropped in the right order
        drop(arena);
    }
}
