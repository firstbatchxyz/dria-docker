#![allow(non_snake_case)]

extern crate redis;

use actix_web::web::Data;
use hashbrown::HashSet;
use mini_moka::sync::Cache;
use redis::Commands;
use std::borrow::Borrow;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::sync::atomic::{AtomicIsize, AtomicUsize, Ordering};
use std::sync::Arc;

use simsimd::SimSIMD;

use rand::{thread_rng, Rng, SeedableRng};

use crate::proto::index_buffer::{LayerNode, Point};
use prost::Message;

use crate::errors::errors::DeserializeError;
use crate::hnsw::utils::{create_max_heap, create_min_heap, IntoHeap, IntoMap, Numeric};

use crate::hnsw::scalar::ScalarQuantizer;
use rayon::prelude::*;
use serde_json::{json, Value};

use crate::db::rocksdb_client::RocksdbClient;
use crate::hnsw::sync_map::SynchronizedNodes;

pub const SINGLE_THREADED_HNSW_BUILD_THRESHOLD: usize = 256;

/*
Redis Scheme

Points: "0", "1" ...
Graph:  graph_level.idx : "2.5320" layer 2, node idx 5320
*/
pub struct HNSW {
    pub m: usize,
    pub m_max0: usize,
    pub rng_seed: u64,
    pub ml: f32,
    pub ef_construction: usize,
    pub ef: usize,
    pub db: Data<RocksdbClient>,
    quantizer: ScalarQuantizer,
    metric: Option<String>,
}

impl HNSW {
    pub fn new(
        M: usize,
        ef_construction: usize,
        ef: usize,
        //contract_id: String,
        metric: Option<String>,
        db: Data<RocksdbClient>,
    ) -> HNSW {
        let m = M;
        let m_max0 = M * 2;
        let ml = 1.0 / (M as f32).ln();
        //let db = RocksdbClient::new(contract_id).expect("Error creating RocksdbClient");
        let sq = ScalarQuantizer::new(256, 1000, 256);

        HNSW {
            m,
            m_max0,
            rng_seed: 0,
            ml,
            ef_construction,
            ef,
            db,
            quantizer: sq,
            metric,
        }
    }

    pub fn set_rng_seed(&mut self, seed: u64) {
        self.rng_seed = seed;
    }

    pub fn set_ef(&mut self, ef: usize) {
        self.ef = ef;
    }

    pub fn select_layer(&self) -> usize {
        let mut random = thread_rng();
        let rand_float: f32 = random.gen_range(1e-6..1.0); // Avoid very small values
        let result = (-1.0 * rand_float.ln() * self.ml) as usize;

        // Optionally clamp to a maximum value if applicable
        let max_layer = 1000; // Example maximum layer
        std::cmp::min(result, max_layer)
    }

    fn distance(&self, x: &[f32], y: &[f32], dist: &Option<String>) -> f32 {
        let dist = match dist.as_ref().map(String::as_str) {
            Some("sqeuclidean") => SimSIMD::sqeuclidean(x, y),
            Some("inner") => SimSIMD::inner(x, y),
            Some("cosine") | None => SimSIMD::cosine(x, y),
            _ => panic!("Unsupported distance metric"),
        };
        if dist.is_none() {
            println!("Error in distance"); //make the error propagate
        }
        dist.unwrap()
    }

    fn get_points_w_memory(
        &self,
        indices: &Vec<u32>,
        point_map: Cache<String, Point>,
    ) -> Vec<Point> {
        // Initialize points with None to reserve the space and maintain order
        let mut points: Vec<Option<Point>> = vec![None; indices.len()];

        // Track missing indices and their positions
        let mut missing_indices_with_pos: Vec<(usize, u32)> = Vec::new();

        for (pos, idx) in indices.iter().enumerate() {
            let key = format!("p:{}", idx);
            if let Some(point) = point_map.get(&key) {
                points[pos] = Some(point.clone());
            } else {
                missing_indices_with_pos.push((pos, *idx));
            }
        }

        if !missing_indices_with_pos.is_empty() {
            let missing_indices: Vec<u32> = missing_indices_with_pos
                .iter()
                .map(|&(_, idx)| idx)
                .collect();

            let fetched_points = self.db.get_points(&missing_indices);

            if fetched_points.is_err() {
                println!(
                    "Error getting points, get points _w _memory {:?}",
                    &missing_indices
                );
            }

            let fetched_points = fetched_points.unwrap();

            for point in fetched_points {
                let key = format!("p:{}", point.idx);
                point_map.insert(key, point.clone());

                if let Some(&(pos, _)) = missing_indices_with_pos
                    .iter()
                    .find(|&&(_, idx)| idx == point.idx)
                {
                    points[pos] = Some(point);
                }
            }
        }
        points.into_iter().filter_map(|p| p).collect()
    }

    fn get_neighbors_w_memory(
        &self,
        layer: usize,
        indices: &Vec<u32>,
        node_map: Arc<SynchronizedNodes>,
    ) -> Vec<LayerNode> {
        let mut nodes = Vec::with_capacity(indices.len());
        let mut missing_indices = Vec::new();

        // First pass: Fill in nodes from node_map or mark as missing
        for &idx in indices {
            let key = format!("{}:{}", layer, idx);
            match node_map.get_or_wait_opt(&key) {
                Some(node) => nodes.push(node.clone()),
                None => {
                    missing_indices.push(idx);
                    nodes.push(LayerNode::new(0, 0)); // Placeholder for missing nodes
                }
            }
        }
        if !missing_indices.is_empty() {
            let fetched_nodes = self
                .db
                .get_neighbors(layer, missing_indices)
                .expect("Error getting neighbors");

            for fetched_node in fetched_nodes.iter() {
                let index = indices.iter().position(|&i| i == fetched_node.idx).unwrap();
                nodes[index] = fetched_node.clone();
            }
            node_map.insert_batch_and_notify(fetched_nodes);
        }

        nodes
    }

    fn get_neighbor_w_memory(
        &self,
        layer: usize,
        idx: usize,
        node_map: Arc<SynchronizedNodes>,
    ) -> LayerNode {
        let key = format!("{}:{}", layer, idx);

        let node_option = node_map.get_or_wait_opt(&key);
        return if let Some(node) = node_option {
            node.clone()
        } else {
            let node_ = self.db.get_neighbor(layer, idx);
            if node_.is_err() {
                println!("Sync issue, awaiting notification...");
                let value = node_map.get_or_wait(&key);
                return value.clone();
            }
            let node_ = node_.unwrap();
            node_map.insert_and_notify(&node_);
            node_
        };
    }

    pub fn insert_w_preset(
        &self,
        idx: usize,
        node_map: Arc<SynchronizedNodes>,
        point_map: Cache<String, Point>,
        nl: Arc<AtomicUsize>,
        epa: Arc<AtomicIsize>,
    ) -> Result<(), DeserializeError> {
        let mut W = HashMap::new();

        let mut ep_index = None;
        let ep_index_ = epa.load(Ordering::SeqCst);
        if ep_index_ != -1 {
            ep_index = Some(ep_index_ as u32);
        }

        let mut num_layers = nl.load(Ordering::Relaxed);

        let L = if num_layers == 0 { 0 } else { num_layers - 1 };
        let l = self.select_layer();

        let qs = self.get_points_w_memory(&vec![idx as u32], point_map.clone());
        let q = qs[0].v.clone();

        if ep_index.is_some() {
            let ep_index_ = ep_index.unwrap();

            let points = self.get_points_w_memory(&vec![ep_index_], point_map.clone());
            let point = points.first().unwrap();
            let dist = self.distance(&q, &point.v, &self.metric);
            let mut ep = HashMap::from([(ep_index_, dist)]);

            for i in ((l + 1)..=L).rev() {
                W = self.search_layer(&q, ep.clone(), 1, i, node_map.clone(), point_map.clone())?;

                if let Some((_, value)) = W.iter().next() {
                    if &dist < value {
                        ep = W;
                    }
                }
            }

            for l_c in (0..=std::cmp::min(L, l)).rev() {
                W = self.search_layer(
                    &q,
                    ep,
                    self.ef_construction,
                    l_c,
                    node_map.clone(),
                    point_map.clone(),
                )?;

                //upsert expire = true by default, populate upserted_keys for replication
                node_map.insert_and_notify(&LayerNode::new(l_c, idx));

                ep = W.clone();
                let neighbors = self.select_neighbors(&q, W, l_c, true);

                let M = if l_c == 0 { self.m_max0 } else { self.m };

                //read neighbors of all nodes in selected neighbors
                let mut indices = neighbors.iter().map(|x| *x.0).collect::<Vec<u32>>();
                indices.push(idx as u32);
                let idx_i = indices.len() - 1;

                let mut nodes = self.get_neighbors_w_memory(l_c, &indices, node_map.clone());

                for (i, (e_i, dist)) in neighbors.iter().enumerate() {
                    if i == idx_i {
                        // We want to skip last layernode, which is idx -> layernode
                        continue;
                    }

                    nodes[i].neighbors.insert(idx as u32, *dist);
                    nodes[idx_i].neighbors.insert(*e_i, *dist);
                }

                // TODO: remove redundant
                for (i, (e_i, dist)) in neighbors.iter().enumerate() {
                    if i == idx_i {
                        // We want to skip last layernode, which is idx -> layernode
                        continue;
                    }
                    let eConn = nodes[i].neighbors.clone();
                    if eConn.len() > M {
                        let eNewConn = self.select_neighbors(&q, eConn, l_c, true);
                        nodes[i].neighbors = eNewConn.clone();
                    }
                }

                node_map.insert_batch_and_notify(nodes);
            }
        }

        for i in num_layers..=l {
            node_map.insert_and_notify(&LayerNode::new(i, idx));
            let _ = epa.fetch_update(Ordering::SeqCst, Ordering::Relaxed, |x| Some(idx as isize));
        }

        let _ = nl.fetch_update(Ordering::SeqCst, Ordering::Acquire, |v| {
            if l + 1 > v {
                Some(l + 1)
            } else {
                None
            }
        });

        Ok(())
    }

    fn search_layer(
        &self,
        q: &Vec<f32>,
        ep: HashMap<u32, f32>,
        ef: usize,
        l_c: usize,
        node_map: Arc<SynchronizedNodes>,
        point_map: Cache<String, Point>,
    ) -> Result<HashMap<u32, f32>, DeserializeError> {
        let mut v = HashSet::new();

        for (k, _) in ep.iter() {
            v.insert(k.clone());
        }

        let mut C = ep.clone().into_minheap();
        let mut W = ep.into_maxheap();

        while !C.is_empty() {
            let c = C.pop().unwrap().0;
            let f_value = W.peek().unwrap().0 .0;

            if c.0 .0 > f_value {
                break;
            }

            let layernd = self.get_neighbor_w_memory(l_c, c.1 as usize, node_map.clone());

            let mut pairs: Vec<_> = layernd.neighbors.into_iter().collect();
            //pairs.sort_by(|&(_, a), &(_, b)| a.partial_cmp(&b).unwrap());
            pairs.sort_by(|&(_, a), &(_, b)| {
                if a.is_nan() || b.is_nan() {
                    println!("NaN value detected: a = {}, b = {}", a, b);
                    std::cmp::Ordering::Greater
                } else {
                    a.partial_cmp(&b).unwrap_or_else(|| {
                        println!("Unexpected comparison error: a = {}, b = {}", a, b);
                        std::cmp::Ordering::Greater
                    })
                }
            });
            let sorted_keys: Vec<u32> = pairs.into_iter().map(|(k, _)| k).collect();

            let neighbors: Vec<u32> = sorted_keys
                .into_iter()
                .filter_map(|x| if !v.contains(&x) { Some(x) } else { None })
                .collect();

            let points = self.get_points_w_memory(&neighbors, point_map.clone());

            let distances = points
                .iter()
                .map(|x| self.distance(&q, &x.v, &self.metric))
                .collect::<Vec<f32>>();

            for (i, d) in neighbors.iter().zip(distances.iter()) {
                v.insert(i.clone());
                if d < &f_value || W.len() < ef {
                    C.push(Reverse((Numeric(d.clone()), i.clone())));
                    W.push((Numeric(d.clone()), i.clone()));
                    if W.len() > ef {
                        W.pop();
                    }
                }
            }
        }

        if ef == 1 {
            if W.len() > 0 {
                let W_map = W.into_map();
                let mut W_min = W_map.into_minheap();
                let mut single_map = HashMap::new();
                let min_val = W_min.pop().unwrap().0;
                single_map.insert(min_val.1, min_val.0 .0);
                return Ok(single_map);
            } else {
                return Ok(HashMap::new());
            }
        }

        Ok(W.into_map())
    }

    fn select_neighbors(
        &self,
        q: &Vec<f32>,
        C: HashMap<u32, f32>,
        l_c: usize,
        k_p_c: bool,
    ) -> HashMap<u32, f32> {
        let mut R = create_min_heap();
        let mut W = C.into_minheap();

        let mut M = 0;

        if l_c > 0 {
            M = self.m
        } else {
            M = self.m_max0;
        }

        let mut W_d = create_min_heap();
        while W.len() > 0 && R.len() < M {
            let e = W.pop().unwrap().0;

            if R.len() == 0 || e.0 < R.peek().unwrap().0 .0 {
                R.push(Reverse(e));
            } else {
                W_d.push(Reverse(e));
            }
        }

        if k_p_c {
            while W_d.len() > 0 && R.len() < M {
                R.push(W_d.pop().unwrap());
            }
        }
        R.into_map()
    }

    pub fn knn_search(
        &self,
        q: &Vec<f32>,
        K: usize,
        node_map: Arc<SynchronizedNodes>,
        point_map: Cache<String, Point>,
    ) -> Vec<Value> {
        let mut W = HashMap::new();

        let ep_index = self.db.get_ep().expect("") as u32;
        let num_layers = self.db.get_num_layers().expect("Error getting num_layers");

        let points = self.get_points_w_memory(&vec![ep_index], point_map.clone());
        let point = points.first().unwrap();
        let dist = self.distance(&q, &point.v, &self.metric);
        let mut ep = HashMap::from([(ep_index, dist)]);

        for l_c in (1..=num_layers - 1).rev() {
            W = self
                .search_layer(&q, ep, 1, l_c, node_map.clone(), point_map.clone())
                .expect("Error searching layer");
            ep = W;
        }

        let ep_ = self
            .search_layer(q, ep, self.ef, 0, node_map.clone(), point_map.clone())
            .expect("Error searching layer");

        let mut heap = ep_.into_minheap();
        let mut sorted_vec = Vec::new();
        while !heap.is_empty() && sorted_vec.len() < K {
            let item = heap.pop().unwrap().0;
            sorted_vec.push((item.1, 1.0 - item.0 .0));
        }
        let indices = sorted_vec.iter().map(|x| x.0).collect::<Vec<u32>>();
        let metadata = self
            .db
            .get_metadatas(indices)
            .expect("Error getting metadatas");

        let result = sorted_vec
            .iter()
            .zip(metadata.iter())
            .map(|(x, y)| json!({"id":x.0, "score":x.1, "metadata":y.clone()}))
            .collect::<Vec<Value>>();
        result
    }
}
