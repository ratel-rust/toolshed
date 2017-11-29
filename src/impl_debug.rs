use std::fmt::{self, Debug};
use std::hash::Hash;
use list::List;
use map::{Map, BloomMap};
use set::{Set, BloomSet};

impl<'arena, T> Debug for List<'arena, T>
where
    T: 'arena + Debug + Copy,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<'arena, K, V> Debug for Map<'arena, K, V>
where
    K: 'arena + Debug + Eq + Hash + Copy,
    V: 'arena + Debug + Copy,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<'arena, K, V> Debug for BloomMap<'arena, K, V>
where
    K: 'arena + Debug + Eq + Hash + Copy + AsRef<[u8]>,
    V: 'arena + Debug + Copy,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<'arena, I> Debug for Set<'arena, I>
where
    I: 'arena + Debug + Eq + Hash + Copy,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl<'arena, I> Debug for BloomSet<'arena, I>
where
    I: 'arena + Debug + Eq + Hash + Copy + AsRef<[u8]>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use Arena;

    #[test]
    fn list_debug() {
        let arena = Arena::new();
        let list = List::from_iter(&arena, ["doge", "to", "the", "moon!"].iter().cloned());

        let debug = format!("{:?}", list);

        assert_eq!(debug, r#"["doge", "to", "the", "moon!"]"#);
    }

    #[test]
    fn map_debug() {
        let arena = Arena::new();
        let map = Map::new();

        map.insert(&arena, "foo", 10u64);
        map.insert(&arena, "bar", 20);
        map.insert(&arena, "doge", 30);

        let debug = format!("{:?}", map);

        assert_eq!(debug, r#"{"foo": 10, "bar": 20, "doge": 30}"#);
    }

    #[test]
    fn bloom_map_debug() {
        let arena = Arena::new();
        let map = BloomMap::new();

        map.insert(&arena, "foo", 10u64);
        map.insert(&arena, "bar", 20);
        map.insert(&arena, "doge", 30);

        let debug = format!("{:?}", map);

        assert_eq!(debug, r#"{"foo": 10, "bar": 20, "doge": 30}"#);
    }

    #[test]
    fn set_debug() {
        let arena = Arena::new();
        let set = Set::new();

        set.insert(&arena, "foo");
        set.insert(&arena, "bar");
        set.insert(&arena, "doge");

        let debug = format!("{:?}", set);

        assert_eq!(debug, r#"{"foo", "bar", "doge"}"#);
    }

    #[test]
    fn bloom_set_debug() {
        let arena = Arena::new();
        let set = BloomSet::new();

        set.insert(&arena, "foo");
        set.insert(&arena, "bar");
        set.insert(&arena, "doge");

        let debug = format!("{:?}", set);

        assert_eq!(debug, r#"{"foo", "bar", "doge"}"#);
    }
}
