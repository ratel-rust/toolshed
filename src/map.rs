//! Maps of keys to values that can be used with the `Arena`.

use std::hash::{Hash, Hasher};
use std::fmt::{self, Debug};
use fxhash::FxHasher;

use cell::CopyCell;
use Arena;
use bloom::bloom;

#[derive(Clone, Copy)]
struct MapNode<'arena, K, V>
where
    K: 'arena,
    V: 'arena + Copy,
{
    pub key: K,
    pub hash: u64,
    pub value: CopyCell<V>,
    pub left: CopyCell<Option<&'arena MapNode<'arena, K, V>>>,
    pub right: CopyCell<Option<&'arena MapNode<'arena, K, V>>>,
}

impl<'arena, K: Debug, V: Debug + Copy> Debug for MapNode<'arena, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("MapNode")
            .field(&self.key)
            .field(&self.value.get())
            .finish()
    }
}

impl<'arena, K: Eq, V: Eq + Copy> PartialEq for MapNode<'arena, K, V> {
    #[inline]
    fn eq(&self, other: &MapNode<'arena, K, V>) -> bool {
        self.hash  == other.hash &&
        self.key   == other.key  &&
        self.value == other.value
    }
}
impl<'arena, K: Eq, V: Eq + Copy> Eq for MapNode<'arena, K, V> {}

impl<'arena, K, V: Copy> MapNode<'arena, K, V> {
    #[inline]
    pub fn new(key: K, hash: u64, value: V) -> Self {
        MapNode {
            key,
            hash,
            value: CopyCell::new(value),
            left: CopyCell::new(None),
            right: CopyCell::new(None),
        }
    }
}

/// A map of keys `K` to values `V`. The map is built as a pseudo-random
/// binary tree with hashes of keys used for ordering.
pub struct Map<'arena, K: 'arena, V: 'arena + Copy> {
    root: CopyCell<Option<&'arena MapNode<'arena, K, V>>>,
}

impl<'arena, K, V> Map<'arena, K, V>
where
    K: Eq + Hash + Copy,
    V: Copy,
{
    /// Create a new, empty `Map`.
    #[inline]
    pub fn new() -> Self {
        Map {
            root: CopyCell::new(None),
        }
    }

    #[inline]
    fn hash_key(key: &K) -> u64 {
        let mut hasher = FxHasher::default();

        key.hash(&mut hasher);

        hasher.finish()
    }

    #[inline]
    fn find_slot(&self, key: K, hash: u64) -> &CopyCell<Option<&'arena MapNode<'arena, K, V>>> {
        let mut node = &self.root;

        loop {
            match node.get() {
                None         => return node,
                Some(parent) => {
                    if hash == parent.hash && key == parent.key {
                        return node;
                    } else if hash < parent.hash {
                        node = &parent.left;
                    } else {
                        node = &parent.right;
                    }
                }
            }
        }
    }

    /// Inserts a key-value pair into the map.
    #[inline]
    pub fn insert(&self, arena: &'arena Arena, key: K, value: V) {
        let hash = Self::hash_key(&key);
        let node = self.find_slot(key, hash);

        match node.get() {
            Some(node) => node.value.set(value),
            None => {
                let new = arena.alloc(MapNode::new(key, hash, value));
                node.set(Some(new));
            }
        }
    }

    /// Returns the value corresponding to the key.
    #[inline]
    pub fn get(&self, key: K) -> Option<V> {
        let hash = Self::hash_key(&key);

        self.find_slot(key, hash).get().map(|node| node.value.get())
    }

    /// Returns true if the map contains a value for the specified key.
    #[inline]
    pub fn contains_key(&self, key: K) -> bool {
        let hash = Self::hash_key(&key);

        self.find_slot(key, hash).get().is_some()
    }

    /// Returns true if the map contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.root.get().is_none()
    }

    /// Clears the map.
    #[inline]
    pub fn clear(&self) {
        self.root.set(None);
    }
}

/// A variant of the `Map` that includes a bloom filter using the
/// `bloom` function for keys that can be represented as byte slices.
///
/// This is ideal for small maps for which querying for absent keys is
/// a common behavior. In this case it will very likely outperform a
/// `HashMap`, even one with a fast hashing algorithm.
pub struct BloomMap<'arena, K: 'arena, V: 'arena + Copy> {
    filter: CopyCell<u64>,
    inner: Map<'arena, K, V>,
}

impl<'arena, K, V> BloomMap<'arena, K, V>
where
    K: Eq + Hash + Copy + AsRef<[u8]>,
    V: Copy,
{
    /// Create a new, empty `BloomMap`.
    #[inline]
    pub fn new() -> Self {
        BloomMap {
            filter: CopyCell::new(0),
            inner: Map::new(),
        }
    }

    /// Inserts a key-value pair into the map.
    #[inline]
    pub fn insert(&self, arena: &'arena Arena, key: K, value: V) {
        self.filter.set(self.filter.get() | bloom(key.as_ref()));
        self.inner.insert(arena, key, value);
    }

    /// Returns the value corresponding to the key.
    #[inline]
    pub fn get(&self, key: K) -> Option<V> {
        let b = bloom(key.as_ref());

        if self.filter.get() & b == b {
            self.inner.get(key)
        } else {
            None
        }
    }

    /// Returns true if the map contains a value for the specified key.
    #[inline]
    pub fn contains_key(&self, key: K) -> bool {
        let b = bloom(key.as_ref());

        self.filter.get() & b == b && self.inner.contains_key(key)
    }

    /// Returns true if the map contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Clears the map.
    #[inline]
    pub fn clear(&self) {
        self.filter.set(0);
        self.inner.clear();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn map() {
        let arena = Arena::new();
        let map = Map::new();

        map.insert(&arena, "foo", 10u64);
        map.insert(&arena, "bar", 20);
        map.insert(&arena, "doge", 30);

        assert_eq!(map.contains_key("foo"), true);
        assert_eq!(map.contains_key("bar"), true);
        assert_eq!(map.contains_key("doge"), true);
        assert_eq!(map.contains_key("moon"), false);

        assert_eq!(map.get("foo"), Some(10));
        assert_eq!(map.get("bar"), Some(20));
        assert_eq!(map.get("doge"), Some(30));
        assert_eq!(map.get("moon"), None);
    }

    #[test]
    fn bloom_map() {
        let arena = Arena::new();
        let map = BloomMap::new();

        map.insert(&arena, "foo", 10u64);
        map.insert(&arena, "bar", 20);
        map.insert(&arena, "doge", 30);

        assert_eq!(map.contains_key("foo"), true);
        assert_eq!(map.contains_key("bar"), true);
        assert_eq!(map.contains_key("doge"), true);
        assert_eq!(map.contains_key("moon"), false);

        assert_eq!(map.get("foo"), Some(10));
        assert_eq!(map.get("bar"), Some(20));
        assert_eq!(map.get("doge"), Some(30));
        assert_eq!(map.get("moon"), None);
    }
}
