use serde::ser::{Serialize, Serializer, SerializeSeq};
use list::List;

impl<'arena, T> Serialize for List<'arena, T>
where
    T: Serialize
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let mut seq = serializer.serialize_seq(None)?;
        for element in self.iter() {
            seq.serialize_element(element)?;
        }
        seq.end()
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
}
