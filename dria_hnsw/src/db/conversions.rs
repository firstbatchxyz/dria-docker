use crate::proto::index_buffer::{Point, LayerNode};
use crate::proto::request_buffer::{Batch, Singleton};
use prost::Message;


pub fn point_to_base64(point: &Point) -> String {

    let mut bytes = Vec::new();
    point.encode(&mut bytes).expect("Failed to encode message");
    let enc = base64::encode(&bytes); // Convert bytes to string if needed
    enc
}

pub fn base64_to_point(e_point: &str) -> Point {

    let bytes = base64::decode(e_point).unwrap();
    let point = Point::decode(bytes.as_slice()).unwrap(); // Deserialize
    point
}

pub fn node_to_base64(node: &LayerNode) -> String {

    let mut bytes = Vec::new();
    node.encode(&mut bytes).expect("Failed to encode message");
    let enc = base64::encode(&bytes); // Convert bytes to string if needed
    enc
}

pub fn base64_to_node(e_node: &str) -> LayerNode {

    let bytes = base64::decode(e_node).unwrap();
    let node = LayerNode::decode(bytes.as_slice()).unwrap(); // Deserialize
    node
}


pub fn base64_to_batch(batch: &str) -> Batch {

    let bytes = base64::decode(batch).unwrap();
    let node = Batch::decode(bytes.as_slice()).unwrap(); // Deserialize
    node
}

pub fn base64_to_singleton(singleton: &str) -> Singleton {

    let bytes = base64::decode(singleton).unwrap();
    let node = Singleton::decode(bytes.as_slice()).unwrap(); // Deserialize
    node
}