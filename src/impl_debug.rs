use std::fmt::{self, Debug};
use list::{List, GrowableList, ListBuilder};
use map::{Map, BloomMap};
use set::{Set, BloomSet};

impl<'arena, T> Debug for List<'arena, T>
where
    T: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<'arena, T> Debug for GrowableList<'arena, T>
where
    T: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_list().fmt(f)
    }
}

impl<'arena, T> Debug for ListBuilder<'arena, T>
where
    T: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_list().fmt(f)
    }
}

impl<'arena, K, V> Debug for Map<'arena, K, V>
where
    K: Debug,
    V: Debug + Copy,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<'arena, K, V> Debug for BloomMap<'arena, K, V>
where
    K: Debug,
    V: Debug + Copy,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<'arena, I> Debug for Set<'arena, I>
where
    I: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl<'arena, I> Debug for BloomSet<'arena, I>
where
    I: Debug,
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
