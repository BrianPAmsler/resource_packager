use std::{fmt::Display};
#[cfg(feature = "read")]
use std::io::{Read, Seek};

#[cfg(feature = "write")]
use serde::{Serialize, Serializer, ser::{SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple, SerializeTupleStruct, SerializeTupleVariant}};
#[cfg(feature = "read")]
use serde::{Deserialize, Deserializer};
use thiserror::Error;

#[cfg(feature = "read")]
use crate::peekable_stream::Peekable;

// Why did I waste my time with this instead of using postcard?
// Nobody will ever know.

#[derive(Error, Debug)]
pub enum SerializerError {
    #[error("serialization error: {0}")]
    SerializeError(String),
    #[error("deserialization error: {0}")]
    DeserializeError(String),
    #[error(transparent)]
    IoError(#[from] std::io::Error)
}

impl serde::ser::Error for SerializerError {
    fn custom<T>(msg:T) -> Self where T:Display {
        Self::SerializeError(format!("{}", msg))
    }
}

impl serde::de::Error for SerializerError {
    fn custom<T>(msg:T) -> Self where T:Display {
        Self::DeserializeError(format!("{}", msg))
    }
}

#[cfg(feature = "write")]
struct BytesSerializer {
    buffer: Vec<u8>
}

#[cfg(feature = "write")]
impl BytesSerializer {
    pub fn new() -> BytesSerializer {
        BytesSerializer { buffer: Vec::new() }
    }

    pub fn take(self) -> Vec<u8> {
        self.buffer
    }
}

#[cfg(feature = "write")]
impl<'a> Serializer for &'a mut BytesSerializer {
    type Ok = ();

    type Error = SerializerError;

    type SerializeSeq = Self;

    type SerializeTuple = Self;

    type SerializeTupleStruct = Self;

    type SerializeTupleVariant = Self;

    type SerializeMap = Self;

    type SerializeStruct = Self;

    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.buffer.push(v as u8);

        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.buffer.push(v as u8);

        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.buffer.extend(v.to_be_bytes());

        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.buffer.extend(v.to_be_bytes());

        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.buffer.extend(v.to_be_bytes());

        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.buffer.extend(v.to_be_bytes());

        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.buffer.extend(v.to_be_bytes());

        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.buffer.extend(v.to_be_bytes());

        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.buffer.extend(v.to_be_bytes());

        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.buffer.extend(v.to_be_bytes());

        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.buffer.extend(v.to_be_bytes());

        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.buffer.extend(v.encode_utf8(&mut [0; 4]).bytes());

        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v.len() as u64)?;
        self.buffer.extend(v.bytes());

        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let len = v.len() as u64;
        
        self.serialize_u64(len)?;
        self.buffer.extend_from_slice(v);

        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_u8(0)
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize {
        self.serialize_u8(1)?;
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_u32(variant_index)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize {
        self.serialize_u32(variant_index)?;
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        if len.is_none() {
            return Err(SerializerError::SerializeError("unsupported serialization".to_owned()));
        }

        self.serialize_u64(len.unwrap() as u64)?;
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.serialize_u32(variant_index)?;
        Ok(self)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        if len.is_none() {
            return Err(SerializerError::SerializeError("unsupported serialization".to_owned()));
        }

        self.serialize_u64(len.unwrap() as u64)?;

        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.serialize_u32(variant_index)?;

        Ok(self)
    }
}


#[cfg(feature = "write")]
impl<'a> SerializeSeq for &'a mut BytesSerializer {
    type Ok = ();

    type Error = SerializerError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

#[cfg(feature = "write")]
impl<'a> SerializeTuple for &'a mut BytesSerializer {
    type Ok = ();

    type Error = SerializerError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

#[cfg(feature = "write")]
impl<'a> SerializeTupleStruct for &'a mut BytesSerializer {
    type Ok = ();

    type Error = SerializerError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
        where
            T: serde::Serialize {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

#[cfg(feature = "write")]
impl<'a> SerializeTupleVariant for &'a mut BytesSerializer {
    type Ok = ();

    type Error = SerializerError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
        where
            T: serde::Serialize {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

#[cfg(feature = "write")]
impl<'a> SerializeMap for &'a mut BytesSerializer {
    type Ok = ();

    type Error = SerializerError;

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    
    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize {
        key.serialize(&mut **self)
    }
    
    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize {
        value.serialize(&mut **self)
    }
}

#[cfg(feature = "write")]
impl<'a> SerializeStruct for &'a mut BytesSerializer {
    type Ok = ();

    type Error = SerializerError;

    fn serialize_field<T: ?Sized>(
            &mut self,
            _key: &'static str,
            value: &T,
        ) -> Result<(), Self::Error>
        where
            T: serde::Serialize {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

#[cfg(feature = "write")]
impl<'a> SerializeStructVariant for &'a mut BytesSerializer {
    type Ok = ();

    type Error = SerializerError;

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    
    fn serialize_field<T: ?Sized>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::Serialize {
        value.serialize(&mut **self)
    }
}

#[cfg(feature = "read")]
trait ReadSeek: Read + Seek {}
#[cfg(feature = "read")]
impl<T: Read + Seek> ReadSeek for T {}

#[cfg(feature = "read")]
struct StreamDeserializer<'de> {
    stream: Peekable<&'de mut dyn ReadSeek>,
}

#[cfg(feature = "read")]
impl<'de> StreamDeserializer<'de> {
    pub fn new<R: Read + Seek + 'static>(stream: &'de mut R) -> StreamDeserializer<'de> {
        let stream: Peekable<&'de mut dyn ReadSeek> = Peekable::new(stream);
        StreamDeserializer { stream }
    }

    pub fn next<T>(&mut self) -> Result<Vec<u8>, SerializerError> {
        let mut buffer = vec![0; std::mem::size_of::<T>()];
        self.stream.read_exact(&mut buffer)?;

        Ok(buffer)
    }

    pub fn next_char(&mut self) -> Result<char, SerializerError> {
        let mut char = vec![0; 4];
        self.stream.peek_exact(&mut char)?;
        let char = char.utf8_chunks().next().ok_or(SerializerError::DeserializeError("Char is not utf-8.".into()))?;
        let char = char.valid().chars().next().ok_or(SerializerError::DeserializeError("Char is not utf-8.".into()))?;
        let bytes_used = char.encode_utf8(&mut [0; 4]).len();

        self.stream.seek_relative(bytes_used as i64)?;

        Ok(char)
    }

    pub fn next_byte_seq(&mut self) -> Result<Vec<u8>, SerializerError> {
        let len = u64::from_be_bytes(self.next::<u64>()?.try_into().unwrap()) as usize;

        let mut bytes = vec![0; len];

        self.stream.read_exact(&mut bytes)?;

        Ok(bytes)
    }
}

#[cfg(feature = "read")]
impl<'de, 'a> Deserializer<'de> for &'a mut StreamDeserializer<'de> {
    type Error = SerializerError;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>
    {
            Err(SerializerError::DeserializeError("unsupported deserialization".to_owned()))
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>
    {
        let value = self.next::<u8>()?[0] > 0;
        visitor.visit_bool(value)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>
    {
        let value = self.next::<i8>()?[0] as i8;
        visitor.visit_i8(value)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>
    {
        let value = i16::from_be_bytes(self.next::<i16>()?.try_into().unwrap());
        visitor.visit_i16(value)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>
    {
        let value = i32::from_be_bytes(self.next::<i32>()?.try_into().unwrap());
        visitor.visit_i32(value)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>
    {
        let value = i64::from_be_bytes(self.next::<i64>()?.try_into().unwrap());
        visitor.visit_i64(value)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>
    {
        let value = self.next::<u8>()?[0];
        visitor.visit_u8(value)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>
    {
        let value = u16::from_be_bytes(self.next::<u16>()?.try_into().unwrap());
        visitor.visit_u16(value)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>
    {
        let value = u32::from_be_bytes(self.next::<u32>()?.try_into().unwrap());
        visitor.visit_u32(value)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>
    {
        let value = u64::from_be_bytes(self.next::<u64>()?.try_into().unwrap());
        visitor.visit_u64(value)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>
    {
        let value = f32::from_be_bytes(self.next::<f32>()?.try_into().unwrap());
        visitor.visit_f32(value)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>
    {
        let value = f64::from_be_bytes(self.next::<f64>()?.try_into().unwrap());
        visitor.visit_f64(value)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>
    {
        let value = self.next_char()?;

        visitor.visit_char(value)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>
    {
        let seq = self.next_byte_seq()?;
        let str = str::from_utf8(&seq)
            .map_err(|_| SerializerError::DeserializeError("Encoded string is not utf-8.".to_owned()))?;

        visitor.visit_str(str)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>
    {
        let seq = self.next_byte_seq()?;
        let str = str::from_utf8(&seq)
            .map_err(|_| SerializerError::DeserializeError("Encoded string is not utf-8.".to_owned()))?;

        visitor.visit_string(str.to_owned())
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>
    {
        let value = self.next_byte_seq()?;
        visitor.visit_bytes(&value)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>
    {
        let value = self.next_byte_seq()?;
        visitor.visit_byte_buf(value.to_vec())
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>
    {
        let byte = self.next::<u8>()?[0];

        if byte == 0 {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>
    {
        let len = u64::from_be_bytes(self.next::<u64>()?.try_into().unwrap()) as usize;
        visitor.visit_seq(SeqAccess::new(self, len))
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>
    {
        visitor.visit_seq(SeqAccess::new(self, len))
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>
    {
        visitor.visit_seq(SeqAccess::new(self, len))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>
    {
        let len = u64::from_be_bytes(self.next::<u64>()?.try_into().unwrap()) as usize;
        visitor.visit_map(MapAccess::new(self, len))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>
    {
        visitor.visit_seq(SeqAccess::new(self, fields.len()))
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>
    {
        visitor.visit_enum(EnumAccess::new(self, name))
    }

    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>
    {
        Err(SerializerError::DeserializeError("unsupported deserialization".to_owned()))
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>
    {
        Err(SerializerError::DeserializeError("unsupported deserialization".to_owned()))
    }
}

#[cfg(feature = "read")]
struct SeqAccess<'a, 'de: 'a> {
    de: &'a mut StreamDeserializer<'de>,
    len: usize,
    count: usize
}

#[cfg(feature = "read")]
impl<'a, 'de> SeqAccess<'a, 'de> {
    pub fn new(de: &'a mut StreamDeserializer<'de>, len: usize) -> SeqAccess<'a, 'de> {
        SeqAccess { de, len, count: 0 }
    }
}

#[cfg(feature = "read")]
impl<'a, 'de> serde::de::SeqAccess<'de> for SeqAccess<'a, 'de> {
    type Error = SerializerError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de> {
        if self.len == self.count {
            return Ok(None);
        }
        
        self.count += 1;
        seed.deserialize(&mut *self.de).map(Some)
    }
}

#[cfg(feature = "read")]
struct MapAccess<'a, 'de: 'a> {
    de: &'a mut StreamDeserializer<'de>,
    len: usize,
    count: usize
}

#[cfg(feature = "read")]
impl<'a, 'de> MapAccess<'a, 'de> {
    pub fn new(de: &'a mut StreamDeserializer<'de>, len: usize) -> MapAccess<'a, 'de> {
        MapAccess { de, len, count: 0 }
    }
}

#[cfg(feature = "read")]
impl<'a, 'de> serde::de::MapAccess<'de> for MapAccess<'a, 'de> {
    type Error = SerializerError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
        where
            K: serde::de::DeserializeSeed<'de> {
        if self.count == self.len {
            return Ok(None);
        }
        
        self.count += 1;
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::DeserializeSeed<'de> {
        seed.deserialize(&mut *self.de)
    }
}

#[cfg(feature = "read")]
struct EnumAccess<'a, 'de: 'a> {
    de: &'a mut StreamDeserializer<'de>,
    name: &'static str
}

#[cfg(feature = "read")]
impl<'a, 'de> EnumAccess<'a, 'de> {
    pub fn new(de: &'a mut StreamDeserializer<'de>, name: &'static str) -> EnumAccess<'a, 'de> {
        EnumAccess { de, name }
    }
}

#[cfg(feature = "read")]
impl<'a, 'de> serde::de::EnumAccess<'de> for EnumAccess<'a, 'de> {
    type Error = SerializerError;
    
    type Variant = VariantAccess<'a, 'de>;
    
    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: serde::de::DeserializeSeed<'de> {
        Ok((seed.deserialize(&mut *self.de)?, VariantAccess::new(self.de, self.name)))
    }

    
}

#[cfg(feature = "read")]
struct VariantAccess<'a, 'de: 'a> {
    de: &'a mut StreamDeserializer<'de>,
    name: &'static str
}

#[cfg(feature = "read")]
impl<'a, 'de> VariantAccess<'a, 'de> {
    pub fn new(de: &'a mut StreamDeserializer<'de>, name: &'static str) -> VariantAccess<'a, 'de> {
        VariantAccess { de, name }
    }
}

#[cfg(feature = "read")]
impl<'a, 'de> serde::de::VariantAccess<'de> for VariantAccess<'a, 'de> {
    type Error = SerializerError;
    
    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }
    
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de> {
        seed.deserialize(&mut *self.de)
    }
    
    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        self.de.deserialize_tuple(len, visitor)
    }
    
    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de> {
        self.de.deserialize_struct(self.name, fields, visitor)
    }
}

#[cfg(feature = "write")]
pub fn serialize<S: Serialize>(value: &S) -> Result<Vec<u8>, SerializerError> {
    let mut serializer = BytesSerializer::new();
    value.serialize(&mut serializer)?;

    Ok(serializer.take())
}

#[cfg(feature = "read")]
pub fn deserialize<'de, R: Read + Seek + 'static, D: Deserialize<'de>>(stream: &'de mut R) -> Result<D, SerializerError> {
    let mut deserializer = StreamDeserializer::new(stream);
    let value = D::deserialize(&mut deserializer)?;

    Ok(value)
}