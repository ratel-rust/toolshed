//! Sets of values that can be used with the `Arena`.

use std::hash::Hash;

use crate::map::{Map, BloomMap, MapIter};
use crate::Arena;

/// A set of values. This structure is using a `Map` with value
/// type set to `()` internally.
#[derive(Clone, Copy)]
pub struct Set<'arena, I> {
    map: Map<'arena, I, ()>,
}

impl<I> Default for Set<'_, I> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'arena, I> Set<'arena, I> {
    /// Creates a new, empty `Set`.
    pub const fn new() -> Self {
        Set {
            map: Map::new(),
        }
    }

    /// Get an iterator over the elements in the set
    #[inline]
    pub fn iter(&self) -> SetIter<'arena, I> {
        SetIter {
            inner: self.map.iter()
        }
    }

    /// Returns `true` if the set contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Clears the map.
    #[inline]
    pub fn clear(&self) {
        self.map.clear()
    }
}

impl<'arena, I> Set<'arena, I>
where
    I: Eq + Hash + Copy,
{
    /// Inserts a value into the set.
    #[inline]
    pub fn insert(&self, arena: &'arena Arena, item: I) {
        self.map.insert(arena, item, ());
    }

    /// Gets a reference to the existing value in the set, if it exists
    #[inline]
    pub fn get(&self, key: I) -> Option<&I> {
        self.map.get_key(key)
    }

    /// Returns `true` if the set contains a value.
    #[inline]
    pub fn contains(&self, item: I) -> bool {
        self.map.contains_key(item)
    }
}

/// A set of values with a bloom filter. This structure is
/// using a `BloomMap` with value type set to `()` internally.
#[derive(Clone, Copy)]
pub struct BloomSet<'arena, I> {
    map: BloomMap<'arena, I, ()>,
}

impl<'arena, I> BloomSet<'arena, I> {
    /// Creates a new, empty `BloomSet`.
    pub const fn new() -> Self {
        BloomSet {
            map: BloomMap::new(),
        }
    }

    /// Get an iterator over the elements in the set
    #[inline]
    pub fn iter(&self) -> SetIter<'arena, I> {
        SetIter {
            inner: self.map.iter()
        }
    }

    /// Returns `true` if the set contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Clears the map.
    #[inline]
    pub fn clear(&self) {
        self.map.clear()
    }
}

impl<'arena, I> BloomSet<'arena, I>
where
    I: Eq + Hash + Copy + AsRef<[u8]>,
{
    /// Inserts a value into the set.
    #[inline]
    pub fn insert(&self, arena: &'arena Arena, item: I) {
        self.map.insert(arena, item, ());
    }

    /// Returns `true` if the set contains a value.
    #[inline]
    pub fn contains(&self, item: I) -> bool {
        self.map.contains_key(item)
    }
}

/// An iterator over the elements in the set.
pub struct SetIter<'arena, I> {
    inner: MapIter<'arena, I, ()>
}

impl<'arena, I> Iterator for SetIter<'arena, I> {
    type Item = &'arena I;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(key, _)| key)
    }
}

impl<'arena, I> IntoIterator for Set<'arena, I> {
    type Item = &'arena I;
    type IntoIter = SetIter<'arena, I>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'arena, I> IntoIterator for BloomSet<'arena, I> {
    type Item = &'arena I;
    type IntoIter = SetIter<'arena, I>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'arena, I> From<Set<'arena, I>> for BloomSet<'arena, I>
where
    I: Eq + Hash + Copy + AsRef<[u8]>,
{
    #[inline]
    fn from(set: Set<'arena, I>) -> BloomSet<'arena, I> {
        BloomSet {
            map: set.map.into()
        }
    }
}

impl<'arena, I> From<BloomSet<'arena, I>> for Set<'arena, I> {
    #[inline]
    fn from(bloom_set: BloomSet<'arena, I>) -> Set<'arena, I> {
        Set {
            map: bloom_set.map.into()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn set() {
        let arena = Arena::new();
        let set = Set::new();

        set.insert(&arena, "foo");
        set.insert(&arena, "bar");
        set.insert(&arena, "doge");

        assert_eq!(set.contains("foo"), true);
        assert_eq!(set.contains("bar"), true);
        assert_eq!(set.contains("doge"), true);
        assert_eq!(set.contains("moon"), false);
    }

    #[test]
    fn bloom_set() {
        let arena = Arena::new();
        let set = BloomSet::new();

        set.insert(&arena, "foo");
        set.insert(&arena, "bar");
        set.insert(&arena, "doge");

        assert_eq!(set.contains("foo"), true);
        assert_eq!(set.contains("bar"), true);
        assert_eq!(set.contains("doge"), true);
        assert_eq!(set.contains("moon"), false);
    }

    #[test]
    fn set_iter() {
        let arena = Arena::new();
        let set = Set::new();

        set.insert(&arena, "foo");
        set.insert(&arena, "bar");
        set.insert(&arena, "doge");

        let mut iter = set.iter();

        assert_eq!(iter.next(), Some(&"foo"));
        assert_eq!(iter.next(), Some(&"bar"));
        assert_eq!(iter.next(), Some(&"doge"));
    }

    #[test]
    fn bloom_set_iter() {
        let arena = Arena::new();
        let set = BloomSet::new();

        set.insert(&arena, "foo");
        set.insert(&arena, "bar");
        set.insert(&arena, "doge");

        let mut iter = set.iter();

        assert_eq!(iter.next(), Some(&"foo"));
        assert_eq!(iter.next(), Some(&"bar"));
        assert_eq!(iter.next(), Some(&"doge"));
    }

    #[test]
    fn from_eq() {
        let arena = Arena::new();
        let set = Set::new();

        set.insert(&arena, "foo");
        set.insert(&arena, "bar");
        set.insert(&arena, "doge");

        let bloom_set = BloomSet::new();

        bloom_set.insert(&arena, "foo");
        bloom_set.insert(&arena, "bar");
        bloom_set.insert(&arena, "doge");

        assert_eq!(set, Set::from(bloom_set));
        assert_eq!(BloomSet::from(set), bloom_set);
    }
}
