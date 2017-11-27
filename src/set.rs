use std::hash::Hash;

use map::{Map, BloomMap};
use arena::Arena;
use bloom::Bloom;

pub struct Set<'arena, I: 'arena> {
    map: Map<'arena, I, ()>,
}

impl<'arena, I> Set<'arena, I>
where
    I: Eq + Hash + Copy,
{
    #[inline]
    pub fn new() -> Self {
        Set {
            map: Map::new(),
        }
    }

    #[inline]
    pub fn insert(&self, arena: &'arena Arena, item: I) {
        self.map.insert(arena, item, ());
    }

    #[inline]
    pub fn contains(&self, item: I) -> bool {
        self.map.contains_key(item)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    #[inline]
    pub fn clear(&self) {
        self.map.clear()
    }
}


pub struct BloomSet<'arena, I: 'arena> {
    map: BloomMap<'arena, I, ()>,
}

impl<'arena, I> BloomSet<'arena, I>
where
    I: Eq + Hash + Copy + Bloom,
{
    #[inline]
    pub fn new() -> Self {
        BloomSet {
            map: BloomMap::new(),
        }
    }

    #[inline]
    pub fn insert(&self, arena: &'arena Arena, item: I) {
        self.map.insert(arena, item, ());
    }

    #[inline]
    pub fn contains(&self, item: I) -> bool {
        self.map.contains_key(item)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    #[inline]
    pub fn clear(&self) {
        self.map.clear()
    }
}
