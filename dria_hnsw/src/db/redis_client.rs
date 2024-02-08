extern crate redis;
use redis::{Client, Commands, Connection};

use crate::db::conversions::{base64_to_node, node_to_base64, point_to_base64};
use crate::db::env::Config;
use crate::errors::errors::DeserializeError;
use crate::proto::index_buffer::{LayerNode, Point};
use prost::Message;
use serde_json::map::Values;
use serde_json::Value;

pub struct RedisClient {
    client: Client,
    connection: Connection,
    tag: String,
}

impl RedisClient {
    pub fn new(contract_id: String) -> Result<RedisClient, DeserializeError> {
        let cfg = Config::new();
        let client =
            Client::open(cfg.redis_url).map_err(|_| DeserializeError::RedisConnectionError)?;
        let connection = client
            .get_connection()
            .map_err(|_| DeserializeError::RedisConnectionError)?;

        Ok(RedisClient {
            client,
            connection,
            tag: contract_id,
        })
    }

    pub fn set(&mut self, key: String, value: String) -> Result<(), DeserializeError> {
        let keys_local = format!("{}.value.{}", self.tag, key);
        let _: () = self
            .connection
            .set(&keys_local, &value)
            .map_err(|_| DeserializeError::RedisConnectionError)?;
        Ok(())
    }

    pub fn get_neighbor(
        &mut self,
        layer: usize,
        idx: usize,
    ) -> Result<LayerNode, DeserializeError> {
        let key = format!(
            "{}.value.{}:{}",
            self.tag,
            layer.to_string(),
            idx.to_string()
        );

        let node_str = match self
            .connection
            .get::<_, String>(key)
            .map_err(|_| DeserializeError::RedisConnectionError)
        {
            Ok(node_str) => node_str,
            Err(_) => {
                return Err(DeserializeError::RedisConnectionError);
            }
        };

        //let node_:Value = serde_json::from_str(&node_str).unwrap();
        Ok(base64_to_node(&node_str))
    }

    pub fn get_neighbors(
        &mut self,
        layer: usize,
        indices: Vec<u32>,
    ) -> Result<Vec<LayerNode>, DeserializeError> {
        let keys = indices
            .iter()
            .map(|x| format!("{}.value.{}:{}", self.tag, layer.to_string(), x.to_string()))
            .collect::<Vec<String>>();

        let values_str: Vec<String> = self
            .connection
            .mget(keys)
            .map_err(|_| DeserializeError::RedisConnectionError)?;

        let neighbors = values_str
            .iter()
            .map(|s| {
                let bytes = base64::decode(s).unwrap();
                let p = LayerNode::decode(bytes.as_slice()).unwrap(); // Deserialize
                Ok(p)
            })
            .collect::<Result<Vec<LayerNode>, DeserializeError>>()?;

        Ok(neighbors)
    }

    pub fn upsert_neighbor(&mut self, node: LayerNode) -> Result<(), DeserializeError> {
        let key = format!("{}.value.{}:{}", self.tag, node.level, node.idx);

        let node_str = node_to_base64(&node);
        self.set(key, node_str)?;

        Ok(())
    }

    pub fn upsert_neighbors(&mut self, nodes: Vec<LayerNode>) -> Result<(), DeserializeError> {
        let mut pairs = Vec::new();
        for node in nodes {
            let key = format!("{}.value.{}:{}", self.tag, node.level, node.idx);
            let node_str = node_to_base64(&node);
            pairs.push((key, node_str));
        }

        let _ = self
            .connection
            .mset(pairs.as_slice())
            .map_err(|_| DeserializeError::RedisConnectionError)?;

        Ok(())
    }

    pub fn get_points(&mut self, indices: &Vec<u32>) -> Result<Vec<Point>, DeserializeError> {
        let keys = indices
            .iter()
            .map(|x| format!("{}.value.{}", self.tag, x))
            .collect::<Vec<String>>();

        if keys.is_empty() {
            return Ok(vec![]);
        }
        let values_str: Vec<String> = self
            .connection
            .mget(keys)
            .map_err(|_| DeserializeError::RedisConnectionError)?;

        let points = values_str
            .into_iter()
            .map(|s| {
                let bytes = base64::decode(s).unwrap();
                let p = Point::decode(bytes.as_slice()).unwrap(); // Deserialize
                Ok(p)
            })
            .collect::<Result<Vec<Point>, DeserializeError>>()?;
        Ok(points)
    }

    pub fn add_points_batch(
        &mut self,
        v: &Vec<Vec<f32>>,
        start_idx: usize,
    ) -> Result<(), DeserializeError> {
        let mut pairs = Vec::new();

        for (i, p) in v.iter().enumerate() {
            let idx = start_idx + i;
            let p = Point::new(p.clone(), idx);
            let p_str = point_to_base64(&p);
            pairs.push((format!("{}.value.{}", self.tag, idx), p_str));
        }

        let _ = self
            .connection
            .mset(pairs.as_slice())
            .map_err(|_| DeserializeError::RedisConnectionError)?;
        Ok(())
    }

    pub fn add_points(&mut self, v: Vec<f32>, idx: usize) -> Result<(), DeserializeError> {
        let p = Point::new(v, idx);
        let p_str = point_to_base64(&p);

        self.set(format!("{}.value.{}", self.tag, idx), p_str)?;
        Ok(())
    }

    pub fn set_datasize(&mut self, datasize: usize) -> Result<(), DeserializeError> {
        let _: () = self
            .connection
            .set(format!("{}.value.datasize", self.tag), datasize.to_string())
            .map_err(|_| DeserializeError::RedisConnectionError)?;
        Ok(())
    }

    pub fn get_datasize(&mut self) -> Result<usize, DeserializeError> {
        let datasize: String = self
            .connection
            .get(format!("{}.value.datasize", self.tag))
            .map_err(|_| DeserializeError::RedisConnectionError)?;
        let datasize = datasize.parse::<usize>().unwrap();
        Ok(datasize)
    }

    pub fn set_num_layers(&mut self, num_layers: usize) -> Result<(), DeserializeError> {
        let _: () = self
            .connection
            .set(
                format!("{}.value.num_layers", self.tag),
                num_layers.to_string(),
            )
            .map_err(|_| DeserializeError::RedisConnectionError)?;
        Ok(())
    }

    pub fn get_num_layers(&mut self) -> Result<usize, DeserializeError> {
        let num_layers: String = self
            .connection
            .get(format!("{}.value.num_layers", self.tag))
            .map_err(|_| DeserializeError::RedisConnectionError)?;
        let num_layers = num_layers.parse::<usize>().unwrap();
        Ok(num_layers)
    }

    pub fn set_ep(&mut self, ep: usize) -> Result<(), DeserializeError> {
        let _: () = self
            .connection
            .set(format!("{}.value.ep", self.tag), ep.to_string())
            .map_err(|_| DeserializeError::RedisConnectionError)?;
        Ok(())
    }

    pub fn get_ep(&mut self) -> Result<usize, DeserializeError> {
        let ep: String = self
            .connection
            .get(format!("{}.value.ep", self.tag))
            .map_err(|_| DeserializeError::RedisConnectionError)?;
        let ep = ep.parse::<usize>().unwrap();
        Ok(ep)
    }

    pub fn set_metadata_batch(
        &mut self,
        metadata: Vec<Value>,
        idx: usize,
    ) -> Result<(), DeserializeError> {
        let mut pairs = Vec::new();
        for (i, m) in metadata.iter().enumerate() {
            let key = format!("{}.value.m:{}", self.tag, idx + i);
            let metadata_str = serde_json::to_string(&m).unwrap();
            pairs.push((key, metadata_str));
        }
        let _ = self
            .connection
            .mset(pairs.as_slice())
            .map_err(|_| DeserializeError::RedisConnectionError)?;
        Ok(())
    }

    pub fn set_metadata(&mut self, metadata: Value, idx: usize) -> Result<(), DeserializeError> {
        let key = format!("{}.value.m:{}", self.tag, idx);
        let metadata_str = serde_json::to_string(&metadata).unwrap();
        self.set(key, metadata_str)?;
        Ok(())
    }

    pub fn get_metadata(&mut self, idx: usize) -> Result<Value, DeserializeError> {
        let key = format!("{}.value.m:{}", self.tag, idx);
        let metadata_str: String = self
            .connection
            .get(&key)
            .map_err(|_| DeserializeError::RedisConnectionError)?;
        let metadata: Value = serde_json::from_str(&metadata_str).unwrap();
        Ok(metadata)
    }

    pub fn get_metadatas(&mut self, indices: Vec<u32>) -> Result<Vec<Value>, DeserializeError> {
        let keys = indices
            .iter()
            .map(|x| format!("{}.value.m:{}", self.tag, x))
            .collect::<Vec<String>>();

        let metadata_str: Vec<String> = self
            .connection
            .mget(&keys)
            .map_err(|_| DeserializeError::RedisConnectionError)?;

        let metadata = metadata_str
            .into_iter()
            .map(|s| {
                let m: Value = serde_json::from_str(&s).unwrap();
                Ok(m)
            })
            .collect::<Result<Vec<Value>, DeserializeError>>()?;

        Ok(metadata)
    }
}
