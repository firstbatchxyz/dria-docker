#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Singleton {
    /// Vector of floats
    #[prost(float, repeated, tag="1")]
    pub v: ::prost::alloc::vec::Vec<f32>,
    /// Neighbor idx and its distance
    #[prost(map="string, string", tag="2")]
    pub metadata: ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Batch {
    #[prost(string, repeated, tag="1")]
    pub b: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}