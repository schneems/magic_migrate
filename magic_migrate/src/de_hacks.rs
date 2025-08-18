//! A module of hacky things to get an `impl serde::de::Deserializer` from popular libraries
//!
//! Do not use this directly as the API is not stable

/// Toml Deserializer Wrapper
///
/// With `toml` (0.9) the `toml::Deserializer::parse` function returns a
/// result instead of waiting until deserialize time to error.
///
/// We need to turn that into an `impl serde::de::Deserializer`. To do that, this wrapper
/// saves the error case until the deserializer is used and returns it then.
/// This (seems to) happen when the input is not valid toml.
#[cfg(feature = "toml")]
pub(crate) struct TomlDeWrapper<'de> {
    inner: Result<toml::de::Deserializer<'de>, toml::de::Error>,
}

#[cfg(feature = "toml")]
impl<'de> TomlDeWrapper<'de> {
    pub(crate) fn new(input: &'de str) -> Self {
        TomlDeWrapper {
            inner: toml::Deserializer::parse(input),
        }
    }
}

#[cfg(feature = "toml")]
impl<'de> serde::de::Deserializer<'de> for TomlDeWrapper<'de> {
    type Error = toml::de::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, toml::de::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.inner {
            Ok(deserializer) => deserializer.deserialize_any(visitor),
            Err(e) => Err(e),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, toml::de::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.inner {
            Ok(deserializer) => deserializer.deserialize_bool(visitor),
            Err(e) => Err(e),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, toml::de::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.inner {
            Ok(deserializer) => deserializer.deserialize_i8(visitor),
            Err(e) => Err(e),
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, toml::de::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.inner {
            Ok(deserializer) => deserializer.deserialize_i16(visitor),
            Err(e) => Err(e),
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, toml::de::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.inner {
            Ok(deserializer) => deserializer.deserialize_i32(visitor),
            Err(e) => Err(e),
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, toml::de::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.inner {
            Ok(deserializer) => deserializer.deserialize_i64(visitor),
            Err(e) => Err(e),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, toml::de::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.inner {
            Ok(deserializer) => deserializer.deserialize_u8(visitor),
            Err(e) => Err(e),
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, toml::de::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.inner {
            Ok(deserializer) => deserializer.deserialize_u16(visitor),
            Err(e) => Err(e),
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, toml::de::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.inner {
            Ok(deserializer) => deserializer.deserialize_u32(visitor),
            Err(e) => Err(e),
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, toml::de::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.inner {
            Ok(deserializer) => deserializer.deserialize_u64(visitor),
            Err(e) => Err(e),
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, toml::de::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.inner {
            Ok(deserializer) => deserializer.deserialize_f32(visitor),
            Err(e) => Err(e),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, toml::de::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.inner {
            Ok(deserializer) => deserializer.deserialize_f64(visitor),
            Err(e) => Err(e),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, toml::de::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.inner {
            Ok(deserializer) => deserializer.deserialize_char(visitor),
            Err(e) => Err(e),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, toml::de::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.inner {
            Ok(deserializer) => deserializer.deserialize_str(visitor),
            Err(e) => Err(e),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, toml::de::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.inner {
            Ok(deserializer) => deserializer.deserialize_string(visitor),
            Err(e) => Err(e),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, toml::de::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.inner {
            Ok(deserializer) => deserializer.deserialize_bytes(visitor),
            Err(e) => Err(e),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, toml::de::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.inner {
            Ok(deserializer) => deserializer.deserialize_byte_buf(visitor),
            Err(e) => Err(e),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, toml::de::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.inner {
            Ok(deserializer) => deserializer.deserialize_option(visitor),
            Err(e) => Err(e),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, toml::de::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.inner {
            Ok(deserializer) => deserializer.deserialize_unit(visitor),
            Err(e) => Err(e),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, toml::de::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.inner {
            Ok(deserializer) => deserializer.deserialize_unit_struct(name, visitor),
            Err(e) => Err(e),
        }
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, toml::de::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.inner {
            Ok(deserializer) => deserializer.deserialize_newtype_struct(name, visitor),
            Err(e) => Err(e),
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, toml::de::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.inner {
            Ok(deserializer) => deserializer.deserialize_seq(visitor),
            Err(e) => Err(e),
        }
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, toml::de::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.inner {
            Ok(deserializer) => deserializer.deserialize_tuple(len, visitor),
            Err(e) => Err(e),
        }
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, toml::de::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.inner {
            Ok(deserializer) => deserializer.deserialize_tuple_struct(name, len, visitor),
            Err(e) => Err(e),
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, toml::de::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.inner {
            Ok(deserializer) => deserializer.deserialize_map(visitor),
            Err(e) => Err(e),
        }
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, toml::de::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.inner {
            Ok(deserializer) => deserializer.deserialize_struct(name, fields, visitor),
            Err(e) => Err(e),
        }
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, toml::de::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.inner {
            Ok(deserializer) => deserializer.deserialize_enum(name, variants, visitor),
            Err(e) => Err(e),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, toml::de::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.inner {
            Ok(deserializer) => deserializer.deserialize_identifier(visitor),
            Err(e) => Err(e),
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, toml::de::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.inner {
            Ok(deserializer) => deserializer.deserialize_ignored_any(visitor),
            Err(e) => Err(e),
        }
    }
}
