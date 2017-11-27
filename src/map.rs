use std::hash::{Hash, Hasher};
use std::fmt::{self, Debug};
use fxhash::FxHasher;

use cell::CopyCell;
use arena::Arena;
use bloom::Bloom;

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

pub struct Map<'arena, K: 'arena, V: 'arena + Copy> {
    root: CopyCell<Option<&'arena MapNode<'arena, K, V>>>,
}

impl<'arena, K, V> Map<'arena, K, V>
where
    K: Eq + Hash + Copy,
    V: Copy,
{
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

    #[inline]
    pub fn get(&self, key: K) -> Option<V> {
        let hash = Self::hash_key(&key);

        self.find_slot(key, hash).get().map(|node| node.value.get())
    }

    #[inline]
    pub fn contains_key(&self, key: K) -> bool {
        let hash = Self::hash_key(&key);

        self.find_slot(key, hash).get().is_some()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.root.get().is_none()
    }

    #[inline]
    pub fn clear(&self) {
        self.root.set(None);
    }
}


pub struct BloomMap<'arena, K: 'arena, V: 'arena + Copy> {
    filter: CopyCell<u64>,
    inner: Map<'arena, K, V>,
}

impl<'arena, K, V> BloomMap<'arena, K, V>
where
    K: Eq + Hash + Copy + Bloom,
    V: Copy,
{
    #[inline]
    pub fn new() -> Self {
        BloomMap {
            filter: CopyCell::new(0),
            inner: Map::new(),
        }
    }

    #[inline]
    pub fn insert(&self, arena: &'arena Arena, key: K, value: V) {
        self.filter.set(self.filter.get() | key.bloom());
        self.inner.insert(arena, key, value);
    }

    #[inline]
    pub fn get(&self, key: K) -> Option<V> {
        let b = key.bloom();

        if self.filter.get() & b == b {
            self.inner.get(key)
        } else {
            None
        }
    }

    #[inline]
    pub fn contains_key(&self, key: K) -> bool {
        let b = key.bloom();

        self.filter.get() & b == b && self.inner.contains_key(key)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    #[inline]
    pub fn clear(&self) {
        self.filter.set(0);
        self.inner.clear();
    }
}
