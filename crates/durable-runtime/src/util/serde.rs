pub(crate) struct EmptyMapDeserializer;

impl<'de> serde::Deserializer<'de> for EmptyMapDeserializer {
    type Error = EmptyMapError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct struct enum identifier ignored_any
    }
}

impl<'de> serde::de::MapAccess<'de> for EmptyMapDeserializer {
    type Error = EmptyMapError;

    fn next_key_seed<K>(&mut self, _: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        Ok(None)
    }

    fn next_value_seed<V>(&mut self, _: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        unimplemented!("there are no keys in the map so next_value should never be called")
    }
}

#[derive(Debug)]
pub(crate) enum EmptyMapError {}

impl serde::de::Error for EmptyMapError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        panic!("encountered an error when deserializing an empty map: {msg}")
    }
}

impl std::fmt::Display for EmptyMapError {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {}
    }
}

impl std::error::Error for EmptyMapError {}
