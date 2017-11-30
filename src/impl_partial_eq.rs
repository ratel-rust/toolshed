use std::hash::Hash;
use list::List;
use map::{Map, BloomMap};
use set::{Set, BloomSet};

impl<'arena, T> PartialEq for List<'arena, T>
where
    T: 'arena + PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<'arena, K, V> PartialEq for Map<'arena, K, V>
where
    K: 'arena + Eq + Hash + Copy,
    V: 'arena + PartialEq + Copy,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<'arena, K, V> PartialEq for BloomMap<'arena, K, V>
where
    K: 'arena + Eq + Hash + Copy + AsRef<[u8]>,
    V: 'arena + PartialEq + Copy,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<'arena, I> PartialEq for Set<'arena, I>
where
    I: 'arena + Eq + Hash + Copy,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<'arena, I> PartialEq for BloomSet<'arena, I>
where
    I: 'arena + Eq + Hash + Copy + AsRef<[u8]>,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}
