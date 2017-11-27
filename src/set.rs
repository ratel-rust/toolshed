//! Sets of values that can be used with the `Arena`.

use std::hash::Hash;

use map::{Map, BloomMap};
use Arena;

/// A set of values. This structure is using a `Map` with value
/// type set to `()` internally.
pub struct Set<'arena, I: 'arena> {
    map: Map<'arena, I, ()>,
}

impl<'arena, I> Set<'arena, I>
where
    I: Eq + Hash + Copy,
{
    /// Creates a new, empty `Set`.
    #[inline]
    pub fn new() -> Self {
        Set {
            map: Map::new(),
        }
    }

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

/// A set of values with a bloom filter. This structure is
/// using a `BloomMap` with value type set to `()` internally.
pub struct BloomSet<'arena, I: 'arena> {
    map: BloomMap<'arena, I, ()>,
}

impl<'arena, I> BloomSet<'arena, I>
where
    I: Eq + Hash + Copy + AsRef<[u8]>,
{
    /// Creates a new, empty `BloomSet`.
    #[inline]
    pub fn new() -> Self {
        BloomSet {
            map: BloomMap::new(),
        }
    }

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
}
