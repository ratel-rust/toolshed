//! A linked list and auxiliary types that can be used with the `Arena`.

use arena::Arena;
use cell::CopyCell;

#[derive(Debug, PartialEq, Clone)]
struct ListNode<'arena, T: 'arena> {
    value: T,
    next: CopyCell<Option<&'arena ListNode<'arena, T>>>,
}

impl<'arena, T: Copy> Copy for ListNode<'arena, T> {}

/// A single-ended linked list.
#[derive(Clone)]
pub struct List<'arena, T: 'arena> {
    root: CopyCell<Option<&'arena ListNode<'arena, T>>>,
}

impl<'arena, T: Copy> Copy for List<'arena, T> { }

impl<'arena, T: 'arena> List<'arena, T> {
    /// Create a new empty `List`.
    #[inline]
    pub fn empty() -> Self {
        List {
            root: CopyCell::new(None)
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

    /// Returns an iterator over the items in the list.
    #[inline]
    pub fn iter(&self) -> ListIter<'arena, T> {
        ListIter {
            next: self.root.get()
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
            Some(&ListNode {
                ref value,
                ref next,
                ..
            }) if next.get().is_none() => Some(value),
            _                          => None
        }
    }

    /// Returns the reference to the first element.
    #[inline]
    pub fn first_element(&self) -> Option<&'arena T> {
        self.root.get().map(|li| &li.value)
    }

    /// Returns an `UnsafeList` for the current `List`. While this function is
    /// safe itself, using `UnsafeList` might lead to undefined behavior.
    #[inline]
    pub fn into_unsafe(self) -> UnsafeList {
        UnsafeList {
            root: match self.root.get() {
                Some(ptr) => ptr as *const ListNode<'arena, T> as usize,
                None      => 0
            }
        }
    }
}

impl<'arena, T: 'arena + Copy> List<'arena, T> {
    /// Create a single-element list from the given value.
    #[inline]
    pub fn from(arena: &'arena Arena, value: T) -> List<'arena, T> {
        List {
            root: CopyCell::new(Some(arena.alloc(ListNode {
                value,
                next: CopyCell::new(None)
            })))
        }
    }

    /// Create a list from an iterator of items.
    pub fn from_iter<I>(arena: &'arena Arena, source: I) -> List<'arena, T> where
        I: IntoIterator<Item = T>
    {
        let mut iter = source.into_iter();

        let builder = match iter.next() {
            Some(item) => ListBuilder::new(arena, item),
            None       => return List::empty(),
        };

        for item in iter {
            builder.push(arena, item);
        }

        builder.as_list()
    }

    /// Adds a new element to the beginning of the list.
    #[inline]
    pub fn prepend(&self, arena: &'arena Arena, value: T) -> &'arena T {
        let root = arena.alloc(
            ListNode {
                value,
                next: self.root
            }
        );

        self.root.set(Some(root));

        &root.value
    }

    /// Removes the first element from the list and returns it.
    #[inline]
    pub fn shift(&self) -> Option<&'arena T> {
        let list_item = match self.root.get() {
            None => return None,
            Some(list_item) => list_item
        };

        self.root.set(list_item.next.get());

        Some(&list_item.value)
    }

    /// Get the first element of the `List`, if any, then create a
    /// new `List` starting from the second element at the reference to
    /// the old list.
    ///
    /// Note: This does not modify the internal state of the `List`.
    ///       If you wish to modify the list use `shift` instead.
    #[inline]
    pub fn shift_ref(&mut self) -> Option<&'arena T> {
        let list_item = match self.root.get() {
            None => return None,
            Some(list_item) => list_item
        };

        *self = List {
            root: list_item.next
        };

        Some(&list_item.value)
    }
}

impl<'arena, T: 'arena> IntoIterator for List<'arena, T> {
    type Item = &'arena T;
    type IntoIter = ListIter<'arena, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, 'arena, T: 'arena> IntoIterator for &'a List<'arena, T> {
    type Item = &'arena T;
    type IntoIter = ListIter<'arena, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// A variant of the `List` that keeps track of the last element and thus
/// allows user to push to the end of the list.
#[derive(Clone, Copy)]
pub struct GrowableList<'arena, T>
where
    T: 'arena,
{
    last: CopyCell<Option<&'arena ListNode<'arena, T>>>,
    first: CopyCell<Option<&'arena ListNode<'arena, T>>>,
}

impl<'arena, T> GrowableList<'arena, T>
where
    T: 'arena + Copy,
{
    /// Push a new item at the end of the `List`.
    #[inline]
    pub fn push(&self, arena: &'arena Arena, item: T) {
        let next = Some(&*arena.alloc(ListNode {
            value: item,
            next: CopyCell::new(None)
        }));

        match self.last.get() {
            Some(ref last) => last.next.set(next),
            None           => self.first.set(next),
        }

        self.last.set(next);
    }
}

impl<'arena, T> GrowableList<'arena, T>
where
    T: 'arena,
{
    /// Create a new builder.
    #[inline]
    pub fn new() -> Self {
        GrowableList {
            first: CopyCell::new(None),
            last: CopyCell::new(None),
        }
    }

    /// Get a `List` from the builder.
    #[inline]
    pub fn as_list(&self) -> List<'arena, T> {
        List {
            root: self.first
        }
    }
}

/// A builder that allows one to push elements onto the end of the list.
///
/// This is in principle identical to `GrowableList`, however it skips
/// some checks on pushing given that it always has to have at least one
/// element, and thus might be ever so slightly faster.
#[derive(Clone, Copy)]
pub struct ListBuilder<'arena, T: 'arena>
where
    T: 'arena,
{
    first: &'arena ListNode<'arena, T>,
    last: CopyCell<&'arena ListNode<'arena, T>>,
}

impl<'arena, T> ListBuilder<'arena, T>
where
    T: 'arena + Copy,
{
    /// Create a new builder with the first element.
    #[inline]
    pub fn new(arena: &'arena Arena, first: T) -> Self {
        let first = arena.alloc(ListNode {
            value: first,
            next: CopyCell::new(None)
        });

        ListBuilder {
            first,
            last: CopyCell::new(first),
        }
    }

    /// Push a new item at the end of the `List`.
    #[inline]
    pub fn push(&self, arena: &'arena Arena, item: T) {
        let next = arena.alloc(ListNode {
            value: item,
            next: CopyCell::new(None)
        });

        self.last.get().next.set(Some(next));
        self.last.set(next);
    }
}

impl<'arena, T> ListBuilder<'arena, T>
where
    T: 'arena,
{
    /// Get a `List` from the builder.
    #[inline]
    pub fn as_list(&self) -> List<'arena, T> {
        List {
            root: CopyCell::new(Some(self.first))
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
    pub unsafe fn into_list<'arena, T: 'arena>(self) -> List<'arena, T> {
        List {
            root: CopyCell::new(match self.root {
                0   => None,
                ptr => Some(&*(ptr as *const ListNode<'arena, T>))
            })
        }
    }
}

/// An iterator over the items in the list.
pub struct ListIter<'arena, T: 'arena> {
    next: Option<&'arena ListNode<'arena, T>>
}

impl<'arena, T: 'arena> Iterator for ListIter<'arena, T> {
    type Item = &'arena T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next;

        next.map(|list_item| {
            let value = &list_item.value;
            self.next = list_item.next.get();
            value
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn builder() {
        let arena = Arena::new();
        let builder = ListBuilder::new(&arena, 10);

        builder.push(&arena, 20);
        builder.push(&arena, 30);

        let list = builder.as_list();

        assert!(list.iter().eq([10, 20, 30].iter()));
    }

    #[test]
    fn empty_builder() {
        let arena = Arena::new();
        let builder = GrowableList::new();

        builder.push(&arena, 10);
        builder.push(&arena, 20);
        builder.push(&arena, 30);

        let list = builder.as_list();

        assert!(list.iter().eq([10, 20, 30].iter()));
    }

    #[test]
    fn from_iter() {
        let arena = Arena::new();
        let list = List::from_iter(&arena, [10, 20, 30].iter().cloned());

        assert!(list.iter().eq([10, 20, 30].iter()));
    }

    #[test]
    fn prepend() {
        let arena = Arena::new();
        let list = List::from(&arena, 30);

        list.prepend(&arena, 20);
        list.prepend(&arena, 10);

        assert!(list.iter().eq([10, 20, 30].iter()));
    }

    #[test]
    fn only_element() {
        let arena = Arena::new();
        let list = List::from(&arena, 42);

        assert_eq!(list.only_element(), Some(&42));

        list.prepend(&arena, 10);

        assert_eq!(list.only_element(), None);
    }

    #[test]
    fn shift() {
        let arena = Arena::new();
        let builder = GrowableList::new();

        builder.push(&arena, 10);
        builder.push(&arena, 20);
        builder.push(&arena, 30);

        let list = builder.as_list();

        assert_eq!(list.shift(), Some(&10));

        assert!(list.iter().eq([20, 30].iter()));
    }

    #[test]
    fn shift_ref() {
        let arena = Arena::new();
        let builder = GrowableList::new();

        builder.push(&arena, 10);
        builder.push(&arena, 20);
        builder.push(&arena, 30);

        let list_a = builder.as_list();
        let mut list_b = list_a;

        assert_eq!(list_b.shift_ref(), Some(&10));

        assert!(list_a.iter().eq([10, 20, 30].iter()));
        assert!(list_b.iter().eq([20, 30].iter()));
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
