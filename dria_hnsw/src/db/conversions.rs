use crate::proto::index_buffer::{LayerNode, Point};
use crate::proto::insert_buffer::{BatchStr, BatchVec, SingletonStr, SingletonVec};
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

//*************** Batch to Singleton ***************//

pub fn base64_to_batch_vec(batch: &str) -> BatchVec {
    let bytes = base64::decode(batch).unwrap();
    let node = BatchVec::decode(bytes.as_slice()).unwrap(); // Deserialize
    node
}

pub fn base64_to_singleton_vec(singleton: &str) -> SingletonVec {
    let bytes = base64::decode(singleton).unwrap();
    let node = SingletonVec::decode(bytes.as_slice()).unwrap(); // Deserialize
    node
}

pub fn base64_to_batch_str(batch: &str) -> BatchStr {
    let bytes = base64::decode(batch).unwrap();
    let node = BatchStr::decode(bytes.as_slice()).unwrap(); // Deserialize
    node
}

pub fn base64_to_singleton_str(singleton: &str) -> SingletonStr {
    let bytes = base64::decode(singleton).unwrap();
    let node = SingletonStr::decode(bytes.as_slice()).unwrap(); // Deserialize
    node
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_to_base64() {
        let point = Point {
            idx: 1,
            v: vec![1.0, 2.0, 3.0],
        };
        let enc = point_to_base64(&point);
        let dec = base64_to_point(&enc);
        assert_eq!(point, dec);
    }

    #[test]
    fn test_node_to_base64() {
        let node = LayerNode {
            level: 1,
            idx: 1,
            visible: true,
            neighbors: std::collections::HashMap::new(),
        };
        let enc = node_to_base64(&node);
        let dec = base64_to_node(&enc);
        assert_eq!(node, dec);
    }

    #[test]
    fn test_node_to_base64_from_string() {
        let node = LayerNode {
            level: 1,
            idx: 1,
            visible: true,
            neighbors: std::collections::HashMap::new(),
        };
        let enc = "CAEQARgB".to_string();
        let dec = base64_to_node(&enc);
        assert_eq!(node, dec);
    }

    #[test]
    fn test_batch_to_singleton() {
        let singleton = SingletonVec {
            v: vec![1.0, 2.0, 3.0],
            map: std::collections::HashMap::new(),
        };
        let batch = BatchVec {
            s: vec![singleton.clone()],
        };
        let enc = base64::encode(&batch.encode_to_vec());
        let dec = base64_to_batch_vec(&enc);
        assert_eq!(batch, dec);

        let singleton = SingletonStr {
            v: "test".to_string(),
            map: std::collections::HashMap::new(),
        };
        let batch = BatchStr {
            s: vec![singleton.clone()],
        };
        let enc = base64::encode(&batch.encode_to_vec());
        let dec = base64_to_batch_str(&enc);
        assert_eq!(batch, dec);
    }
}
