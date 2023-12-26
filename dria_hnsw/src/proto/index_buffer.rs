#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LayerNode {
    /// Using uint64 as a safe alternative for usize
    #[prost(uint32, tag="1")]
    pub level: u32,
    /// Using uint64 as a safe alternative for usize
    #[prost(uint32, tag="2")]
    pub idx: u32,
    /// Whether the node is visible
    #[prost(bool, tag="3")]
    pub visible: bool,
    /// Neighbor idx and its distance
    #[prost(map="uint32, float", tag="4")]
    pub neighbors: ::std::collections::HashMap<u32, f32>,
}
impl LayerNode{
    pub fn new(level:usize, idx:usize)->LayerNode{
        LayerNode{
            level: level as u32,
            idx: idx as u32,
            visible: true,
            neighbors: ::std::collections::HashMap::new()
        }
    }
}


#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Point {
    /// Using uint64 as a safe alternative for usize
    #[prost(uint32, tag="1")]
    pub idx: u32,
    /// Vector of floats
    #[prost(float, repeated, tag="2")]
    pub v: ::prost::alloc::vec::Vec<f32>,
}
impl Point{
    pub fn new(vec:Vec<f32>, idx:usize)->Point{
        Point{
            idx: idx as u32,
            v: vec
        }
    }
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PointQuant {
    /// Using uint64 as a safe alternative for usize
    #[prost(uint32, tag="1")]
    pub idx: u32,
    /// Vector of ints
    #[prost(uint32, repeated, tag="2")]
    pub v: ::prost::alloc::vec::Vec<u32>,
}
impl PointQuant{
    pub fn new(vec:Vec<u32>, idx:usize)->PointQuant{
        PointQuant{
            idx: idx as u32,
            v: vec
        }
    }
}