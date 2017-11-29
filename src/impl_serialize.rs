use std::hash::Hash;
use serde::ser::{Serialize, Serializer};
use list::List;
use map::{Map, BloomMap};
use set::{Set, BloomSet};

impl<'arena, T> Serialize for List<'arena, T>
where
    T: Serialize
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        serializer.collect_seq(self.iter())
    }
}

impl<'arena, K, V> Serialize for Map<'arena, K, V>
where
    K: 'arena + Serialize + Eq + Hash + Copy,
    V: 'arena + Serialize + Copy,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        serializer.collect_map(self.iter())
    }
}

impl<'arena, K, V> Serialize for BloomMap<'arena, K, V>
where
    K: 'arena + Serialize + Eq + Hash + Copy + AsRef<[u8]>,
    V: 'arena + Serialize + Copy,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        serializer.collect_map(self.iter())
    }
}

impl<'arena, I> Serialize for Set<'arena, I>
where
    I: 'arena + Serialize + Eq + Hash + Copy,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        serializer.collect_seq(self.iter())
    }
}

impl<'arena, I> Serialize for BloomSet<'arena, I>
where
    I: 'arena + Serialize + Eq + Hash + Copy + AsRef<[u8]>,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        serializer.collect_seq(self.iter())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json;
    use Arena;

    #[test]
    fn list_can_be_serialized() {
        let arena = Arena::new();
        let list = List::from_iter(&arena, ["doge", "to", "the", "moon!"].iter().cloned());
        let json = serde_json::to_string(&list).unwrap();

        assert_eq!(json, r#"["doge","to","the","moon!"]"#);
    }

    #[test]
    fn map_can_be_serialized() {
        let arena = Arena::new();
        let map = Map::new();

        map.insert(&arena, "foo", 10u64);
        map.insert(&arena, "bar", 20);
        map.insert(&arena, "doge", 30);

        let json = serde_json::to_string(&map).unwrap();

        assert_eq!(json, r#"{"foo":10,"bar":20,"doge":30}"#);
    }

    #[test]
    fn bloom_map_can_be_serialized() {
        let arena = Arena::new();
        let map = BloomMap::new();

        map.insert(&arena, "foo", 10u64);
        map.insert(&arena, "bar", 20);
        map.insert(&arena, "doge", 30);

        let json = serde_json::to_string(&map).unwrap();

        assert_eq!(json, r#"{"foo":10,"bar":20,"doge":30}"#);
    }

    #[test]
    fn set_can_be_serialized() {
        let arena = Arena::new();
        let set = Set::new();

        set.insert(&arena, "foo");
        set.insert(&arena, "bar");
        set.insert(&arena, "doge");

        let json = serde_json::to_string(&set).unwrap();

        assert_eq!(json, r#"["foo","bar","doge"]"#);
    }

    #[test]
    fn bloom_set_can_be_serialized() {
        let arena = Arena::new();
        let set = BloomSet::new();

        set.insert(&arena, "foo");
        set.insert(&arena, "bar");
        set.insert(&arena, "doge");

        let json = serde_json::to_string(&set).unwrap();

        assert_eq!(json, r#"["foo","bar","doge"]"#);
    }
}
