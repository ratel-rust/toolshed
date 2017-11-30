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
    pub next: CopyCell<Option<&'arena MapNode<'arena, K, V>>>,
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
            next: CopyCell::new(None),
        }
    }
}

/// A map of keys `K` to values `V`. The map is built as a pseudo-random
/// binary tree with hashes of keys used for balancing the tree nodes.
///
/// All the nodes of the map are also linked to allow iteration in
/// insertion order.
#[derive(Clone, Copy)]
pub struct Map<'arena, K: 'arena, V: 'arena + Copy> {
    root: CopyCell<Option<&'arena MapNode<'arena, K, V>>>,
    last: CopyCell<Option<&'arena MapNode<'arena, K, V>>>,
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
            last: CopyCell::new(None),
        }
    }

    /// Get an iterator over key value pairs.
    #[inline]
    pub fn iter(&self) -> MapIter<'arena, K, V> {
        MapIter {
            next: self.root.get()
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

    /// Inserts a key-value pair into the map. If the key was previously set,
    /// old value is returned.
    #[inline]
    pub fn insert(&self, arena: &'arena Arena, key: K, value: V) -> Option<V> {
        let hash = Self::hash_key(&key);
        let node = self.find_slot(key, hash);

        match node.get() {
            Some(node) => {
                let old = node.value.get();
                node.value.set(value);
                Some(old)
            },
            None => {
                let new = Some(arena.alloc(MapNode::new(key, hash, value)));

                if let Some(last) = self.last.get() {
                    last.next.set(new);
                }

                self.last.set(new);
                node.set(new);
                None
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
#[derive(Clone, Copy)]
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

    /// Get an iterator over key value pairs.
    #[inline]
    pub fn iter(&self) -> MapIter<'arena, K, V> {
        self.inner.iter()
    }

    /// Inserts a key-value pair into the map. If the key was previously set,
    /// old value is returned.
    #[inline]
    pub fn insert(&self, arena: &'arena Arena, key: K, value: V) -> Option<V> {
        self.filter.set(self.filter.get() | bloom(key));
        self.inner.insert(arena, key, value)
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
        let b = bloom(key);

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

/// An iterator over the entries in the map.
/// All entries are returned in insertion order.
pub struct MapIter<'arena, K, V>
where
    K: 'arena,
    V: 'arena + Copy,
{
    next: Option<&'arena MapNode<'arena, K, V>>
}

impl<'arena, K, V> Iterator for MapIter<'arena, K, V>
where
    K: 'arena,
    V: 'arena + Copy,
{
    type Item = (&'arena K, V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next;

        next.map(|map_node| {
            let key = &map_node.key;
            let value = map_node.value.get();
            self.next = map_node.next.get();
            (key, value)
        })
    }
}

impl<'arena, K, V> IntoIterator for Map<'arena, K, V>
where
    K: 'arena + Eq + Hash + Copy,
    V: 'arena + Copy,
{
    type Item = (&'arena K, V);
    type IntoIter = MapIter<'arena, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'arena, K, V> IntoIterator for BloomMap<'arena, K, V>
where
    K: 'arena + Eq + Hash + Copy + AsRef<[u8]>,
    V: 'arena + Copy,
{
    type Item = (&'arena K, V);
    type IntoIter = MapIter<'arena, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'arena, K, V> From<Map<'arena, K, V>> for BloomMap<'arena, K, V>
where
    K: 'arena + Eq + Hash + Copy + AsRef<[u8]>,
    V: 'arena + Copy,
{
    fn from(map: Map<'arena, K, V>) -> BloomMap<'arena, K, V> {
        let mut filter = 0;

        for (key, _) in map.iter() {
            filter |= bloom(key.as_ref());
        }

        BloomMap {
            filter: CopyCell::new(filter),
            inner: map,
        }
    }
}

impl<'arena, K, V> From<BloomMap<'arena, K, V>> for Map<'arena, K, V>
where
    K: 'arena + Eq + Hash + Copy + AsRef<[u8]>,
    V: 'arena + Copy,
{
    #[inline]
    fn from(bloom_map: BloomMap<'arena, K, V>) -> Map<'arena, K, V> {
        bloom_map.inner
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

    #[test]
    fn iter() {
        let arena = Arena::new();
        let map = Map::new();

        map.insert(&arena, "foo", 10u64);
        map.insert(&arena, "bar", 20);
        map.insert(&arena, "doge", 30);

        let mut iter = map.iter();

        assert_eq!(iter.next(), Some((&"foo", 10)));
        assert_eq!(iter.next(), Some((&"bar", 20)));
        assert_eq!(iter.next(), Some((&"doge", 30)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn insert_replace() {
        let arena = Arena::new();
        let map = Map::new();

        map.insert(&arena, "foo", 10u64);
        map.insert(&arena, "bar", 20);
        map.insert(&arena, "doge", 30);

        let mut iter = map.iter();

        assert_eq!(iter.next(), Some((&"foo", 10)));
        assert_eq!(iter.next(), Some((&"bar", 20)));
        assert_eq!(iter.next(), Some((&"doge", 30)));
        assert_eq!(iter.next(), None);

        map.insert(&arena, "bar", 42);

        let mut iter = map.iter();

        assert_eq!(iter.next(), Some((&"foo", 10)));
        assert_eq!(iter.next(), Some((&"bar", 42)));
        assert_eq!(iter.next(), Some((&"doge", 30)));
        assert_eq!(iter.next(), None);
    }

    // #[test]
    // fn from_eq() {
    //     let arena = Arena::new();
    //     let map = Map::new();

    //     map.insert(&arena, "foo", 10);
    //     map.insert(&arena, "bar", 20);
    //     map.insert(&arena, "doge", 30);

    //     let bloom_map = BloomMap::new();

    //     bloom_map.insert(&arena, "foo", 10);
    //     bloom_map.insert(&arena, "bar", 20);
    //     bloom_map.insert(&arena, "doge", 30);

    //     assert_eq!(map, Map::from(bloom_map));
    //     assert_eq!(BloomMap::from(map), bloom_map);
    // }
}
