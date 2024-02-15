use crate::db::conversions::{base64_to_node, base64_to_point, node_to_base64, point_to_base64};
use crate::errors::errors::DeserializeError;
use crate::proto::index_buffer::{LayerNode, Point};
use prost::Message;
use rocksdb;
use rocksdb::{Options, DB, WriteBatchWithTransaction, WriteBatch, DBWithThreadMode, MultiThreaded};
use serde::de::Unexpected::Option;
use serde_json::map::Values;
use serde_json::Value;
use crate::db::env::Config;

#[derive(Debug)]
pub struct RocksdbClient {
    tag: String,
    client: DB,
}

impl RocksdbClient {
    pub fn new(contract_id: String) -> Result<RocksdbClient, DeserializeError> {
        let cfg = Config::new();
        // Create a new database options instance.
        let mut opts = Options::default();
        opts.create_if_missing(true); // Creates a database if it does not exist.
        //let x = DBWithThreadMode::open(&opts, cfg.rocksdb_path);
        let db = DB::open(&opts, cfg.rocksdb_path).unwrap();

        Ok(RocksdbClient {
            tag: contract_id,
            client: db,
        })
    }

    pub fn set(&self, key: String, value: String) -> Result<(), DeserializeError> {
        let _: () = self
            .client.put(key.as_bytes(), value.as_bytes())
            .map_err(|_| DeserializeError::RedisConnectionError)?;
        Ok(())
    }

    pub fn get_neighbor(&self, layer: usize, idx: usize) -> Result<LayerNode, DeserializeError> {
        let key = format!("{}.value.{}:{}", self.tag, layer, idx);

        let value = self
            .client
            .get(key.as_bytes())
            .map_err(|_| DeserializeError::RocksDBConnectionError)?; // Handle RocksDB errors appropriately

        let node_str = match value {
            Some(value) => String::from_utf8(value).map_err(|_| DeserializeError::InvalidForm)?, // Convert bytes to String and handle UTF-8 error
            None => return Err(DeserializeError::MissingKey), // Handle case where key is not found
        };

        Ok(base64_to_node(&node_str))
    }

    pub fn get_neighbors(
        &self,
        layer: usize,
        indices: Vec<u32>,
    ) -> Result<Vec<LayerNode>, DeserializeError> {
        // Collect keys as a Vec<Vec<u8>> for multi_get
        let keys = indices
            .iter()
            .map(|&x| format!("{}.value.{}:{}", self.tag, layer, x).into_bytes())
            .collect::<Vec<Vec<u8>>>();

        // Use multi_get to fetch values for all keys at once
        let values = self.client.multi_get(keys);

        let mut neighbors = Vec::new();
        for value_result in values {
            // Correctly handle the Result<Option<Vec<u8>>, E> for each value
            match value_result {
                Ok(Some(v)) => {
                    let node_str =
                        String::from_utf8(v).map_err(|_| DeserializeError::InvalidForm)?; // Convert bytes to String and handle UTF-8 error
                    let node = base64_to_node(&node_str); // Convert String to LayerNode and handle base64 error
                    neighbors.push(node);
                }
                Ok(None) => return Err(DeserializeError::MissingKey), // Handle case where key is not found
                Err(_) => return Err(DeserializeError::RocksDBConnectionError), // Handle error in fetching value
            }
        }

        Ok(neighbors)
    }

    pub fn upsert_neighbor(&self, node: LayerNode) -> Result<(), DeserializeError> {
        let key = format!("{}:{}", node.level, node.idx);

        let node_str = node_to_base64(&node);
        self.set(key, node_str)?;

        Ok(())
    }

    pub fn upsert_neighbors(&self, nodes: Vec<LayerNode>) -> Result<(), DeserializeError> {

        let mut batch = WriteBatch::default();
        for node in nodes {
            let key = format!("{}:{}", node.level, node.idx);
            let node_str = node_to_base64(&node);
            batch.put(key.as_bytes(), node_str.as_bytes());
        }

        let _ = self.client.write(batch).map_err(|_| DeserializeError::RocksDBConnectionError)?;

        Ok(())
    }

    pub fn get_points(&self, indices: &Vec<u32>) -> Result<Vec<Point>, DeserializeError> {
        let keys = indices
            .iter()
            .map(|x| format!("{}.value.{}", self.tag, x).into_bytes())
            .collect::<Vec<Vec<u8>>>();

        if keys.is_empty() {
            return Ok(vec![]);
        }

        // Assuming multi_get directly returns Vec<Result<Option<Vec<u8>>, E>>
        let values = self.client.multi_get(keys);

        let mut points = Vec::new();
        for value_result in values {
            match value_result {
                Ok(Some(value)) => {
                    let point_str =
                        String::from_utf8(value).map_err(|_| DeserializeError::InvalidForm)?; // Handle UTF-8 conversion error
                    let point = base64_to_point(&point_str); // Handle potential error from base64_to_point
                    points.push(point);
                }
                Ok(None) => return Err(DeserializeError::MissingKey), // Key not found
                Err(_) => return Err(DeserializeError::RocksDBConnectionError), // Error fetching from RocksDB
            }
        }

        Ok(points)
    }

    pub fn add_points(&self, v: Vec<f32>, idx: usize) -> Result<(), DeserializeError> {
        let p = Point::new(v, idx);
        let p_str = point_to_base64(&p);
        let key = format!("{}.value.{}", self.tag, idx).into_bytes();
        self.client.put(key, p_str.as_bytes()).map_err(|_| DeserializeError::RocksDBConnectionError)?;
        //self.put_multi_hashtag(&[idx.to_string()], &[json!(p_str)], false)?;
        Ok(())
    }

    pub fn add_points_batch(&self, v: &Vec<Vec<f32>>, start_idx: usize) -> Result<(), DeserializeError> {

        let mut batch = WriteBatch::default();
        for (i, p) in v.iter().enumerate() {
            let idx = start_idx + i;
            let p = Point::new(p.clone(), idx);
            let p_str = point_to_base64(&p);
            let key = format!("{}.value.{}", self.tag, idx).into_bytes();

            //keys.push(idx.to_string());
            //values.push(json!(p_str));

            batch.put(key, p_str.as_bytes());
        }

        self.client.write(batch).map_err(|_| DeserializeError::RocksDBConnectionError)?;

        Ok(())
    }

    pub fn set_datasize(&self, datasize: usize) -> Result<(), DeserializeError> {
        //self.put_multi_hashtag(&["datasize".to_string()], &[json!(datasize)], false)?;
        self.client.put(format!("{}.value.datasize", self.tag).into_bytes(), datasize.to_string().as_bytes()).map_err(|_| DeserializeError::RocksDBConnectionError)?;
        Ok(())
    }

    pub fn get_datasize(&self) -> Result<usize, DeserializeError> {
        let datasize_key: String = format!("{}.value.datasize", self.tag);
        let value = self
            .client
            .get(datasize_key.as_bytes())
            .map_err(|_| DeserializeError::RocksDBConnectionError)?;

        let datasize = match value {
            Some(value_bytes) => {
                let value_str = String::from_utf8(value_bytes)
                    .map_err(|_| DeserializeError::InvalidForm)?; // Handle UTF-8 error gracefully
                value_str.parse::<usize>()
                    .map_err(|_| DeserializeError::InvalidForm)? // Handle parse error gracefully
            },
            None => return Err(DeserializeError::MissingKey), // Handle case where key is not found
        };
        Ok(datasize)
    }

    pub fn get_num_layers(&self) -> Result<usize, DeserializeError> {
        let num_layers_key: String = format!("{}.value.num_layers", self.tag);
        let value = self
            .client
            .get(num_layers_key.as_bytes())
            .map_err(|_| DeserializeError::RocksDBConnectionError)?;

        let num_layers = match value {
            Some(value_bytes) => {
                let value_str = String::from_utf8(value_bytes)
                    .map_err(|_| DeserializeError::InvalidForm)?; // Handle UTF-8 error gracefully
                value_str.parse::<usize>()
                    .map_err(|_| DeserializeError::InvalidForm)? // Handle parse error gracefully
            },
            None => return Err(DeserializeError::MissingKey), // Handle case where key is not found
        };
        Ok(num_layers)
    }

    pub fn set_num_layers(&self, num_layers: usize, expire: bool) -> Result<(), DeserializeError> {
        self.client.put(format!("{}.value.num_layers", self.tag).into_bytes(), num_layers.to_string().as_bytes()).map_err(|_| DeserializeError::RocksDBConnectionError)?;
        Ok(())
    }

    pub fn set_ep(&self, ep: usize, expire: bool) -> Result<(), DeserializeError> {
        self.client.put(format!("{}.value.ep", self.tag).into_bytes(), ep.to_string().as_bytes()).map_err(|_| DeserializeError::RocksDBConnectionError)?;
        Ok(())
    }

    pub fn get_ep(&self) -> Result<usize, DeserializeError> {
        let ep_key: String = format!("{}.value.ep", self.tag);
        let value = self
            .client
            .get(ep_key.as_bytes())
            .map_err(|_| DeserializeError::RocksDBConnectionError)?;

        // Attempt to convert the fetched value from bytes to String, then parse it into usize
        let ep_usize = match value {
            Some(value_bytes) => {
                // Convert bytes to String
                let value_str = String::from_utf8(value_bytes)
                    .map_err(|_| DeserializeError::InvalidForm)?; // Handle UTF-8 error gracefully
                // Parse String to usize
                value_str.parse::<usize>()
                    .map_err(|_| DeserializeError::InvalidForm)? // Handle parse error gracefully
            },
            None => return Err(DeserializeError::MissingKey), // Handle case where key is not found
        };
        Ok(ep_usize)
    }

    pub fn set_metadata(&self, metadata: Value, idx: usize) -> Result<(), DeserializeError> {
        let key = format!("{}.value.m:{}", self.tag, idx);
        let metadata_str = serde_json::to_vec(&metadata).unwrap();
        self.client.put(key.as_bytes(), metadata_str).map_err(|_| DeserializeError::RocksDBConnectionError)?;
        Ok(())
    }

    pub fn set_metadata_batch(&self, metadata: Vec<Value>, idx: usize) -> Result<(), DeserializeError> {
        let mut batch = WriteBatch::default();

        for (i, m) in metadata.iter().enumerate() {
            let key = format!("m:{}", idx + i);
            let metadata_str = serde_json::to_vec(&m).unwrap();
            batch.put(key.as_bytes(), metadata_str);
        }
        self.client.write(batch).map_err(|_| DeserializeError::RocksDBConnectionError)?;
        Ok(())
    }

    pub fn get_metadata(&self, idx: usize) -> Result<Value, DeserializeError> {
        let key = format!("{}.value.m:{}", self.tag, idx);

        let value = self
            .client
            .get(key.as_bytes())
            .map_err(|_| DeserializeError::RocksDBConnectionError)?;

        let metadata = match value {
            Some(value) => serde_json::from_slice(&value).unwrap()
            , // Convert bytes to String and handle UTF-8 error
            None => return Err(DeserializeError::MissingKey), // Handle case where key is not found
        };
        Ok(metadata)
    }

    pub fn get_metadatas(&self, indices: Vec<u32>) -> Result<Vec<Value>, DeserializeError> {
        let keys = indices
            .iter()
            .map(|x| format!("{}.value.m:{}", self.tag, x).as_bytes().to_vec())
            .collect::<Vec<Vec<u8>>>();

        // Assuming multi_get returns Vec<Result<Option<Vec<u8>>, E>> directly
        let values = self.client.multi_get(&keys);

        let mut metadata = Vec::new();
        for value_result in values {
            match value_result {
                Ok(Some(v)) => {
                    // Properly handle potential serde_json deserialization errors
                    match serde_json::from_slice::<Value>(&v) {
                        Ok(meta) => metadata.push(meta),
                        Err(_) => return Err(DeserializeError::InvalidForm), // Add a DeserializationError variant if not already present
                    }
                }
                Ok(None) => return Err(DeserializeError::MissingKey), // Key not found
                Err(_) => return Err(DeserializeError::RocksDBConnectionError), // Error fetching from RocksDB
            }
        }

        Ok(metadata)
    }
}
