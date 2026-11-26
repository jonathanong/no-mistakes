use serde::de::value::{MapDeserializer, SeqDeserializer};
use serde::de::{Deserializer, Error as _, IntoDeserializer, Unexpected, Visitor};
use serde_yaml::{Error, Value};

// Retain YAML scalar types and reject null for typed collections at every depth.
pub(super) struct StrictValue(pub(super) Value);

impl<'de> IntoDeserializer<'de, Error> for StrictValue {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self {
        self
    }
}

macro_rules! scalar_methods {
    ($($method:ident),+ $(,)?) => {
        $(
            fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
                self.0.$method(visitor)
            }
        )+
    };
}

impl<'de> Deserializer<'de> for StrictValue {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.0 {
            Value::Mapping(map) => MapDeserializer::new(
                map.into_iter()
                    .map(|(key, value)| (StrictValue(key), StrictValue(value))),
            )
            .deserialize_any(visitor),
            Value::Sequence(items) => {
                SeqDeserializer::new(items.into_iter().map(StrictValue)).deserialize_any(visitor)
            }
            value => value.deserialize_any(visitor),
        }
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match untag(self.0) {
            value @ Value::Sequence(_) => StrictValue(value).deserialize_any(visitor),
            Value::Null => Err(Error::invalid_type(Unexpected::Other("null"), &visitor)),
            value => value.deserialize_seq(visitor),
        }
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match untag(self.0) {
            value @ Value::Mapping(_) => StrictValue(value).deserialize_any(visitor),
            Value::Null => Err(Error::invalid_type(Unexpected::Other("null"), &visitor)),
            value => value.deserialize_map(visitor),
        }
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.deserialize_map(visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match untag(self.0) {
            Value::Null => visitor.visit_none(),
            value => visitor.visit_some(StrictValue(value)),
        }
    }

    fn deserialize_tuple<V: Visitor<'de>>(
        self,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.0.deserialize_unit_struct(name, visitor)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.0.deserialize_enum(name, variants, visitor)
    }

    scalar_methods!(
        deserialize_bool,
        deserialize_i8,
        deserialize_i16,
        deserialize_i32,
        deserialize_i64,
        deserialize_i128,
        deserialize_u8,
        deserialize_u16,
        deserialize_u32,
        deserialize_u64,
        deserialize_u128,
        deserialize_f32,
        deserialize_f64,
        deserialize_char,
        deserialize_str,
        deserialize_string,
        deserialize_bytes,
        deserialize_byte_buf,
        deserialize_unit,
        deserialize_identifier,
        deserialize_ignored_any,
    );
}

fn untag(mut value: Value) -> Value {
    while let Value::Tagged(tagged) = value {
        value = tagged.value;
    }
    value
}
