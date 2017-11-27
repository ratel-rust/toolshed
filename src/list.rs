use arena::Arena;
use cell::CopyCell;
use std::fmt::{self, Debug};

#[derive(Debug, PartialEq, Clone)]
struct ListItem<'arena, T: 'arena> {
    value: T,
    next: CopyCell<Option<&'arena ListItem<'arena, T>>>,
}

impl<'arena, T: Copy> Copy for ListItem<'arena, T> {}

pub struct ListBuilder<'arena, T: 'arena + Copy> {
    arena: &'arena Arena,
    first: &'arena ListItem<'arena, T>,
    last: &'arena ListItem<'arena, T>,
}

impl<'arena, T: 'arena + Copy> ListBuilder<'arena, T> {
    #[inline]
    pub fn new(arena: &'arena Arena, first: T) -> Self {
        let first = arena.alloc(ListItem {
            value: first,
            next: CopyCell::new(None)
        });

        ListBuilder {
            arena,
            first,
            last: first
        }
    }

    #[inline]
    pub fn push(&mut self, item: T) {
        let next = self.arena.alloc(ListItem {
            value: item,
            next: CopyCell::new(None)
        });

        self.last.next.set(Some(next));
        self.last = next;
    }

    #[inline]
    pub fn into_list(self) -> List<'arena, T> {
        List {
            root: CopyCell::new(Some(self.first))
        }
    }
}

pub struct EmptyListBuilder<'arena, T: 'arena + Copy> {
    arena: &'arena Arena,
    first: Option<&'arena ListItem<'arena, T>>,
    last: Option<&'arena ListItem<'arena, T>>,
}

impl<'arena, T: 'arena + Copy> EmptyListBuilder<'arena, T> {
    #[inline]
    pub fn new(arena: &'arena Arena) -> Self {
        EmptyListBuilder {
            arena,
            first: None,
            last: None,
        }
    }

    #[inline]
    pub fn push(&mut self, item: T) {
        match self.last {
            None => {
                self.first = Some(self.arena.alloc(ListItem {
                    value: item,
                    next: CopyCell::new(None)
                }));
                self.last = self.first;
            },
            Some(ref mut last) => {
                let next = self.arena.alloc(ListItem {
                    value: item,
                    next: CopyCell::new(None)
                });

                last.next.set(Some(next));
                *last = next;
            }
        }
    }

    #[inline]
    pub fn into_list(self) -> List<'arena, T> {
        List {
            root: CopyCell::new(self.first)
        }
    }
}

#[derive(Clone)]
pub struct List<'arena, T: 'arena> {
    root: CopyCell<Option<&'arena ListItem<'arena, T>>>,
}

impl<'arena, T: Copy> Copy for List<'arena, T> { }

#[derive(Debug, Clone, Copy)]
pub struct RawList {
    root: usize
}

impl RawList {
    pub unsafe fn into_list<'arena, T: 'arena>(self) -> List<'arena, T> {
        List {
            root: CopyCell::new(match self.root {
                0   => None,
                ptr => Some(&*(ptr as *const ListItem<'arena, T>))
            })
        }
    }
}

impl<'arena, T: 'arena + PartialEq> PartialEq for List<'arena, T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<'arena, T: 'arena + Debug> Debug for List<'arena, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<'arena, T: 'arena> List<'arena, T> {
    #[inline]
    pub fn empty() -> Self {
        List {
            root: CopyCell::new(None)
        }
    }

    #[inline]
    pub fn clear(&self) {
        self.root.set(None);
    }

    #[inline]
    pub fn into_raw(self) -> RawList {
        RawList {
            root: match self.root.get() {
                Some(ptr) => ptr as *const ListItem<'arena, T> as usize,
                None      => 0
            }
        }
    }

    #[inline]
    pub fn iter(&self) -> ListIter<'arena, T> {
        ListIter {
            next: self.root.get()
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.root.get().is_none()
    }

    /// Returns the first element if, and only if, the list contains
    /// just that single element.
    #[inline]
    pub fn only_element(&self) -> Option<&'arena T> {
        match self.root.get() {
            Some(&ListItem { ref value, ref next, .. }) => {
                match next.get() {
                    None => Some(value),
                    _    => None,
                }
            },
            None => None
        }
    }
}

impl<'arena, T: 'arena + Copy> List<'arena, T> {
    #[inline]
    pub fn from(arena: &'arena Arena, value: T) -> List<'arena, T> {
        List {
            root: CopyCell::new(Some(arena.alloc(ListItem {
                value,
                next: CopyCell::new(None)
            })))
        }
    }

    pub fn from_iter<I>(arena: &'arena Arena, source: I) -> List<'arena, T> where
        I: IntoIterator<Item = T>
    {
        let mut iter = source.into_iter();

        let mut builder = match iter.next() {
            Some(item) => ListBuilder::new(arena, item),
            None       => return List::empty(),
        };

        for item in iter {
            builder.push(item);
        }

        builder.into_list()
    }

    /// Adds a new element to the beginning of the list.
    #[inline]
    pub fn unshift(&self, arena: &'arena Arena, value: T) {
        self.root.set(Some(arena.alloc(
            ListItem {
                value,
                next: self.root
            }
        )));
    }

    /// Removes the first element from the list and returns it.
    #[inline]
    pub fn shift(&self) -> Option<T> {
        let list_item = match self.root.get() {
            None => return None,
            Some(list_item) => list_item
        };

        self.root.set(list_item.next.get());

        Some(list_item.value)
    }

    /// Get the first element of the `List`, if any, then create a
    /// new `List` starting from the second element at the reference to
    /// the old list.
    ///
    /// Note: This does not modify the internal state of the `List`.
    ///       If you wish to modify the list use `shift` instead.
    #[inline]
    pub fn shift_ref(&mut self) -> Option<T> {
        let list_item = match self.root.get() {
            None => return None,
            Some(list_item) => list_item
        };

        *self = List {
            root: list_item.next
        };

        Some(list_item.value)
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

pub struct ListIter<'arena, T: 'arena> {
    next: Option<&'arena ListItem<'arena, T>>
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
