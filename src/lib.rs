use serde::de::{
    self, Visitor,
    value::{MapDeserializer, SeqDeserializer},
};
use std::any::type_name;
use thiserror::Error;

pub mod macros {
    macro_rules! unsupported_type {
        ($trait_fn:ident) => {
            fn $trait_fn<V>(self, _: V) -> Result<V::Value, Self::Error>
            where
                V: serde::de::Visitor<'de>,
            {
                Err(crate::ParamsDeserializationError::unsupported_type::<
                    V::Value,
                >())
            }
        };
    }

    macro_rules! parse_single_value {
        ($trait_fn:ident, $visit_fn:ident, $ty:literal) => {
            fn $trait_fn<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: serde::de::Visitor<'de>,
            {
                if self.0.len() != 1 {
                    return Err(ParamsDeserializationError::WrongNumberOfParameters {
                        got: self.0.len(),
                        expected: 1,
                    });
                }

                let value = self.0.0[0].1;
                let value = value
                    .parse()
                    .map_err(|_| ParamsDeserializationError::ParseError {
                        value: value.to_string(),
                        expected_type: $ty,
                    })?;
                visitor.$visit_fn(value)
            }
        };
    }
    pub(crate) use parse_single_value;
    pub(crate) use unsupported_type;
}

#[derive(Debug, Clone)]
pub struct Params<'de>(&'de [(&'de str, &'de str)]);

impl<'de> Params<'de> {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn iter_entries(&self) -> impl Iterator<Item = (&'de str, &'de str)> {
        self.0.iter().map(|&(k, v)| (k, v))
    }

    fn values(&self) -> impl Iterator<Item = &'de str> {
        self.0.iter().map(|&(_k, v)| v)
    }
}

#[derive(Debug, Clone, Error)]
pub enum ParamsDeserializationError {
    #[error("Unsupported type: {0}")]
    UnsupportedType(&'static str),
    #[error("{0}")]
    Custom(String),
    #[error("Wrong number of parameters. Expected {expected}, got {got}")]
    WrongNumberOfParameters { got: usize, expected: usize },
    #[error("Failed to parse value \"{value}\" as type {expected_type}")]
    ParseError {
        value: String,
        expected_type: &'static str,
    },
    #[error("Failed to parse value \"{value}\" at key \"{key}\" as type {expected_type}")]
    ParseErrorAtKey {
        key: String,
        value: String,
        expected_type: &'static str,
    },

    #[error("Failed to parse value \"{value}\" at index \"{index}\" as type {expected_type}")]
    ParseErrorAtIndex {
        index: usize,
        value: String,
        expected_type: &'static str,
    },
}

impl ParamsDeserializationError {
    fn unsupported_type<T>() -> Self {
        Self::UnsupportedType(type_name::<T>())
    }
}

impl de::Error for ParamsDeserializationError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Custom(msg.to_string())
    }
}

pub struct ParamsDeserializer<'de>(Params<'de>);

impl<'de> ParamsDeserializer<'de> {
    pub fn new(params: Params<'de>) -> Self {
        Self(params)
    }
}

impl<'de> de::Deserializer<'de> for &ParamsDeserializer<'de> {
    type Error = ParamsDeserializationError;

    macros::unsupported_type!(deserialize_bytes);
    macros::unsupported_type!(deserialize_option);
    macros::unsupported_type!(deserialize_identifier);
    macros::unsupported_type!(deserialize_ignored_any);
    macros::unsupported_type!(deserialize_str);
    macros::unsupported_type!(deserialize_any);

    macros::parse_single_value!(deserialize_bool, visit_bool, "bool");
    macros::parse_single_value!(deserialize_i8, visit_i8, "i8");
    macros::parse_single_value!(deserialize_i16, visit_i16, "i16");
    macros::parse_single_value!(deserialize_i32, visit_i32, "i32");
    macros::parse_single_value!(deserialize_i64, visit_i64, "i64");
    macros::parse_single_value!(deserialize_i128, visit_i128, "i128");
    macros::parse_single_value!(deserialize_u8, visit_u8, "u8");
    macros::parse_single_value!(deserialize_u16, visit_u16, "u16");
    macros::parse_single_value!(deserialize_u32, visit_u32, "u32");
    macros::parse_single_value!(deserialize_u64, visit_u64, "u64");
    macros::parse_single_value!(deserialize_u128, visit_u128, "u128");
    macros::parse_single_value!(deserialize_f32, visit_f32, "f32");
    macros::parse_single_value!(deserialize_f64, visit_f64, "f64");
    macros::parse_single_value!(deserialize_string, visit_string, "String");
    macros::parse_single_value!(deserialize_byte_buf, visit_string, "String");
    macros::parse_single_value!(deserialize_char, visit_char, "char");

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(SeqDeserializer::new(self.0.values()))
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.0.len() != len {
            return Err(Self::Error::WrongNumberOfParameters {
                got: self.0.len(),
                expected: len,
            });
        }
        visitor.visit_seq(SeqDeserializer::new(self.0.values()))
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.0.len() != len {
            return Err(Self::Error::WrongNumberOfParameters {
                got: self.0.len(),
                expected: len,
            });
        }

        visitor.visit_seq(SeqDeserializer::new(self.0.values()))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(MapDeserializer::new(self.0.iter_entries()))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(ParamsDeserializationError::unsupported_type::<V::Value>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use matchit::Match;
    use serde::Deserialize;

    #[test]
    fn test() {
        let mut router = matchit::Router::new();
        router.insert("/{principal}/{path}", ()).unwrap();
        let Match { params, .. } = router.at("/user/interesting").unwrap();

        let params: Vec<(&str, &str)> = params.iter().collect();
        let params = Params(params.as_slice());

        let deserializer = ParamsDeserializer::new(params);

        #[derive(Debug, Clone, Deserialize, PartialEq)]
        struct Path {
            principal: String,
            path: String,
        }

        let path = Path::deserialize(&deserializer).unwrap();
        assert_eq!(
            path,
            Path {
                principal: "user".to_owned(),
                path: "interesting".to_owned()
            }
        );

        let path = <(String, String)>::deserialize(&deserializer).unwrap();
        assert_eq!(path, ("user".to_owned(), "interesting".to_owned()));
    }
}
