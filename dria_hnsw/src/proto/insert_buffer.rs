use serde::ser::{Serialize, SerializeStruct, Serializer};

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MetadataValue {
    #[prost(oneof = "metadata_value::ValueType", tags = "1, 2, 3, 4")]
    pub value_type: ::core::option::Option<metadata_value::ValueType>,
}
/// Nested message and enum types in `MetadataValue`.
pub mod metadata_value {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum ValueType {
        #[prost(float, tag = "1")]
        FloatValue(f32),
        #[prost(int64, tag = "2")]
        IntValue(i64),
        #[prost(string, tag = "3")]
        StringValue(::prost::alloc::string::String),
        #[prost(bool, tag = "4")]
        BoolValue(bool),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SingletonVec {
    /// Vector of floats
    #[prost(float, repeated, tag = "1")]
    pub v: ::prost::alloc::vec::Vec<f32>,
    #[prost(map = "string, message", tag = "2")]
    pub map: ::std::collections::HashMap<::prost::alloc::string::String, MetadataValue>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BatchVec {
    #[prost(message, repeated, tag = "1")]
    pub s: ::prost::alloc::vec::Vec<SingletonVec>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SingletonStr {
    /// Vector of strings
    #[prost(string, tag = "1")]
    pub v: ::prost::alloc::string::String,
    #[prost(map = "string, message", tag = "2")]
    pub map: ::std::collections::HashMap<::prost::alloc::string::String, MetadataValue>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BatchStr {
    #[prost(message, repeated, tag = "1")]
    pub s: ::prost::alloc::vec::Vec<SingletonStr>,
}

impl Serialize for metadata_value::ValueType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize the value directly without wrapping it in a structure.
        match *self {
            metadata_value::ValueType::FloatValue(f) => serializer.serialize_f32(f),
            metadata_value::ValueType::IntValue(i) => serializer.serialize_i64(i),
            metadata_value::ValueType::StringValue(ref s) => serializer.serialize_str(s),
            metadata_value::ValueType::BoolValue(b) => serializer.serialize_bool(b),
        }
    }
}

impl Serialize for MetadataValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Use the inner value's serialization directly.
        if let Some(ref value_type) = self.value_type {
            value_type.serialize(serializer)
        } else {
            // Decide how you want to handle MetadataValue when it's None.
            // For example, you might serialize it as null or an empty object.
            serializer.serialize_none()
        }
    }
}
