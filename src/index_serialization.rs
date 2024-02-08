use std::fmt::Display;

use serde::{ser::{Impossible, SerializeSeq, SerializeTuple}, Deserialize, Deserializer, Serializer};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerializationError {
    #[error("serialization error: {0}")]
    SerializeError(String),
    #[error("deserialization error: {0}")]
    DeserializeError(String)
}

impl serde::ser::Error for SerializationError {
    fn custom<T>(msg:T) -> Self where T:Display {
        Self::SerializeError(format!("{}", msg))
    }
}

impl serde::de::Error for SerializationError {
    fn custom<T>(msg:T) -> Self where T:Display {
        Self::DeserializeError(format!("{}", msg))
    }
}

pub struct IndexSerializer {
    buffer: Vec<u8>
}

impl IndexSerializer {
    pub fn new() -> IndexSerializer {
        IndexSerializer { buffer: Vec::new() }
    }

    pub fn take(self) -> Box<[u8]> {
        self.buffer.into_boxed_slice()
    }
}

#[allow(unused)]
impl<'a> Serializer for &'a mut IndexSerializer {
    type Ok = ();

    type Error = SerializationError;

    type SerializeSeq = Self;

    type SerializeTuple = Self;

    type SerializeTupleStruct = Impossible<(), Self::Error>;

    type SerializeTupleVariant = Impossible<(), Self::Error>;

    type SerializeMap = Impossible<(), Self::Error>;

    type SerializeStruct = Impossible<(), Self::Error>;

    type SerializeStructVariant = Impossible<(), Self::Error>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Err(SerializationError::SerializeError("unsupported serialization".to_owned()))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Err(SerializationError::SerializeError("unsupported serialization".to_owned()))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Err(SerializationError::SerializeError("unsupported serialization".to_owned()))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Err(SerializationError::SerializeError("unsupported serialization".to_owned()))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Err(SerializationError::SerializeError("unsupported serialization".to_owned()))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Err(SerializationError::SerializeError("unsupported serialization".to_owned()))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Err(SerializationError::SerializeError("unsupported serialization".to_owned()))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Err(SerializationError::SerializeError("unsupported serialization".to_owned()))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.buffer.extend(v.to_be_bytes());

        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Err(SerializationError::SerializeError("unsupported serialization".to_owned()))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Err(SerializationError::SerializeError("unsupported serialization".to_owned()))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Err(SerializationError::SerializeError("unsupported serialization".to_owned()))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v.len() as u64)?;
        self.buffer.extend(v.bytes());

        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(SerializationError::SerializeError("unsupported serialization".to_owned()))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(SerializationError::SerializeError("unsupported serialization".to_owned()))
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize {
        Err(SerializationError::SerializeError("unsupported serialization".to_owned()))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(SerializationError::SerializeError("unsupported serialization".to_owned()))
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(SerializationError::SerializeError("unsupported serialization".to_owned()))
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(SerializationError::SerializeError("unsupported serialization".to_owned()))
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize {
        Err(SerializationError::SerializeError("unsupported serialization".to_owned()))
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize {
        Err(SerializationError::SerializeError("unsupported serialization".to_owned()))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        if len.is_none() {
            return Err(SerializationError::SerializeError("unsupported serialization".to_owned()));
        }

        println!("serializing sequence...");
        self.serialize_u64(len.unwrap() as u64)?;
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(SerializationError::SerializeError("unsupported serialization".to_owned()))
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(SerializationError::SerializeError("unsupported serialization".to_owned()))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(SerializationError::SerializeError("unsupported serialization".to_owned()))
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(SerializationError::SerializeError("unsupported serialization".to_owned()))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(SerializationError::SerializeError("unsupported serialization".to_owned()))
    }
}


impl<'a> SerializeSeq for &'a mut IndexSerializer {
    type Ok = ();

    type Error = SerializationError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> SerializeTuple for &'a mut IndexSerializer {
    type Ok = ();

    type Error = SerializationError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

pub struct IndexDeserializer<'de> {
    buffer: &'de [u8]
}

impl<'de> IndexDeserializer<'de> {
    pub fn new(data: &'de [u8]) -> IndexDeserializer<'de> {
        IndexDeserializer { buffer: data }
    }

    pub fn next_u64(&mut self) -> Result<u64, SerializationError> {
        if self.buffer.len() < std::mem::size_of::<u64>() {
            return Err(SerializationError::DeserializeError("EOF".to_owned()));
        }

        let value = u64::from_be_bytes(self.buffer[..std::mem::size_of::<u64>()].try_into().unwrap());

        self.buffer = &self.buffer[std::mem::size_of::<u64>()..];

        Ok(value)
    }

    pub fn next_str(&mut self) -> Result<&str, SerializationError> {
        let len = self.next_u64()?;
        if self.buffer.len() < len as usize {
            return Err(SerializationError::DeserializeError("EOF".to_owned()));
        }

        let bytes = &self.buffer[..len as usize];
        let str = std::str::from_utf8(bytes).map_err(|_| SerializationError::DeserializeError("UTF-8 Error".to_owned()))?;

        self.buffer = &self.buffer[len as usize..];

        Ok(str)
    }
}

#[allow(unused)]
impl<'de, 'a> Deserializer<'de> for &'a mut IndexDeserializer<'de> {
    type Error = SerializationError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
            Err(SerializationError::DeserializeError("unsupported deserialization".to_owned()))
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        Err(SerializationError::DeserializeError("unsupported deserialization".to_owned()))
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        Err(SerializationError::DeserializeError("unsupported deserialization".to_owned()))
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        Err(SerializationError::DeserializeError("unsupported deserialization".to_owned()))
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        Err(SerializationError::DeserializeError("unsupported deserialization".to_owned()))
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        Err(SerializationError::DeserializeError("unsupported deserialization".to_owned()))
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        Err(SerializationError::DeserializeError("unsupported deserialization".to_owned()))
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        Err(SerializationError::DeserializeError("unsupported deserialization".to_owned()))
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        Err(SerializationError::DeserializeError("unsupported deserialization".to_owned()))
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        visitor.visit_u64(self.next_u64()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        Err(SerializationError::DeserializeError("unsupported deserialization".to_owned()))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        Err(SerializationError::DeserializeError("unsupported deserialization".to_owned()))
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        Err(SerializationError::DeserializeError("unsupported deserialization".to_owned()))
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        visitor.visit_str(self.next_str()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        visitor.visit_string(self.next_str()?.to_owned())
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        Err(SerializationError::DeserializeError("unsupported deserialization".to_owned()))
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        Err(SerializationError::DeserializeError("unsupported deserialization".to_owned()))
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        Err(SerializationError::DeserializeError("unsupported deserialization".to_owned()))
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        Err(SerializationError::DeserializeError("unsupported deserialization".to_owned()))
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        Err(SerializationError::DeserializeError("unsupported deserialization".to_owned()))
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        Err(SerializationError::DeserializeError("unsupported deserialization".to_owned()))
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        let len = self.next_u64()?;

        visitor.visit_seq(SeqAccess::new(self, len))
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        visitor.visit_seq(SeqAccess::new(self, len as u64))
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        Err(SerializationError::DeserializeError("unsupported deserialization".to_owned()))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        Err(SerializationError::DeserializeError("unsupported deserialization".to_owned()))
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        Err(SerializationError::DeserializeError("unsupported deserialization".to_owned()))
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        Err(SerializationError::DeserializeError("unsupported deserialization".to_owned()))
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        Err(SerializationError::DeserializeError("unsupported deserialization".to_owned()))
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        Err(SerializationError::DeserializeError("unsupported deserialization".to_owned()))
    }
}

struct SeqAccess<'a, 'de: 'a> {
    de: &'a mut IndexDeserializer<'de>,
    len: u64,
    pos: u64
}

impl<'a, 'de> SeqAccess<'a, 'de> {
    pub fn new(de: &'a mut IndexDeserializer<'de>, len: u64) -> SeqAccess<'a, 'de> {
        SeqAccess { de, len, pos: 0 }
    }
}

impl<'a, 'de> serde::de::SeqAccess<'de> for SeqAccess<'a, 'de> {
    type Error = SerializationError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de> {
        if self.len == self.pos {
            return Ok(None);
        }
        
        self.pos += 1;
        seed.deserialize(&mut *self.de).map(Some)
    }
}

pub fn index_from_bytes(bytes: &[u8]) -> Result<Box<[(String, u64, u64)]>, SerializationError> {
    let mut deserializer = IndexDeserializer::new(bytes);
    
    Box::<[(String, u64, u64)]>::deserialize(&mut deserializer)
}