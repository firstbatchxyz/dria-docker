extern crate redis;

use std::cmp::Reverse;
use hashbrown::HashSet;
use std::collections::HashMap;

use redis::Commands;

use rand::{Rng, thread_rng, SeedableRng};

use crate::proto::index_buffer::{Point, LayerNode};
use prost::Message;
use prost_types::Any; // For handling the Any type

use crate::errors::errors::{DeserializeError};
use crate::hnsw::utils::{create_min_heap, create_max_heap, IntoHeap, IntoMap, Numeric};

use crate::distance::metric_old_aarch::cosine_distance_aarch;
use serde_json::{json, Value};
use crate::db::redis_client::RedisClient;


/*
Redis Scheme

Points: "0", "1" ...
Graph:  graph_level.idx : "2.5320" layer 2, node idx 5320
*/

pub struct HNSW{
    pub data_size: usize,
    pub num_layers: usize,
    pub m: usize,
    pub m_max0: usize,
    pub rng_seed: u64,
    pub ml: f32,
    pub ef_construction: usize,
    pub ef: usize,
    pub ep: Option<u32>,
    pub db: RedisClient
    //pub metric: Metric,
}

impl HNSW{

    pub fn new(M: usize, ef_construction:usize, ef:usize, contract_id: String) -> HNSW{

        let m = M;
        let m_max0 = M * 2;
        let ml = 1.0 / (M as f32).ln();
        let mut db = RedisClient::new(contract_id).unwrap();


        let mut num_layers = 0;
        let mut data_size = 0;

        let nl = db.get_num_layers();
        let mut ep = None;
        if nl.is_err(){
            db.set_num_layers(0).expect("Error setting num_layers");
            db.set_datasize(0).expect("Error setting datasize");
        }
        else{
            num_layers = nl.unwrap();
            data_size = db.get_datasize().expect("Error setting num_layers");
            ep = Some(db.get_ep().expect("Error getting ep") as u32);
        }

        HNSW{
            data_size,
            num_layers,
            m,
            m_max0,
            rng_seed: 0,
            ml,
            ef_construction,
            ef,
            ep,
            db
            //metric: Metric::Angular,
        }
    }

    pub fn set_rng_seed(&mut self, seed: u64){
        self.rng_seed = seed;
    }

    pub fn set_ef(&mut self, ef: usize){
        self.ef = ef;
    }

    pub fn select_layer(&self)->usize{
        let mut random = thread_rng();
        let rand_float: f32 = random.gen();
        (-1.0 * rand_float.ln() * &self.ml) as usize
    }

    pub fn insert(&mut self, q: Vec<f32>, metadata: Value)->Result<(), DeserializeError>{

        let mut W = HashMap::new();
        let ep_index = self.ep.clone();
        self.num_layers = self.db.get_num_layers().expect("Error getting num_layers") as usize;
        let L = if self.num_layers == 0 { 0 } else { self.num_layers - 1 };
        let l = self.select_layer();


        self.data_size = self.db.get_datasize().expect("Error getting datasize") as usize;
        let idx = self.data_size;

        self.db.add_points(q.clone(), idx).expect("Error adding points");
        self.db.set_metadata(metadata, idx).expect("Error setting metadata");

        let ds = self.db.get_datasize().expect("Error getting datasize") as usize;
        self.db.set_datasize(ds + 1).expect("Error setting datasize");

        if ep_index.is_some(){
            let ep_index_ = ep_index.unwrap();
            let points = self.db.get_points(vec![ep_index_]).expect("Error getting points");
            let point = points.first().unwrap();
            let dist = unsafe {cosine_distance_aarch(&q, &point.v)};
            let mut ep = HashMap::from([(ep_index_, dist)]);

            for i in ((l + 1)..=L).rev() {
                W = self.search_layer(&q, ep.clone(), 1, i);

                if let Some((_, value)) = W.iter().next(){
                    if &dist < value {
                        ep = W;
                    }
                }
            }

            for l_c in (0..=std::cmp::min(L, l)).rev() {
                W = self.search_layer(&q, ep, self.ef_construction, l_c);

                self.db.upsert_neighbor(LayerNode::new(l_c, idx)).expect("Error upserting neighbor");
                ep = W.clone();
                let neighbors = self.select_neighbors(&q, W, l_c, true);

                let M = if l_c == 0 { self.m_max0 } else { self.m };
                //read neighbors of all nodes in selected neighbors
                let mut indices = neighbors.iter().map(|x| *x.0).collect::<Vec<u32>>();
                indices.push(idx as u32);
                let idx_i = indices.len()-1;

                let mut nodes = self.db.get_neighbors(l_c, indices).expect("Error getting neighbors");

                for (i, (e_i,dist)) in neighbors.iter().enumerate(){
                    if i == idx_i{
                        // We wan't to skip last layernode, which is idx -> layernode
                        continue;
                    }
                    nodes[i].neighbors.insert(idx as u32, *dist);
                    nodes[idx_i].neighbors.insert(*e_i, *dist);
                }

                for (i, (e_i,dist)) in neighbors.iter().enumerate(){
                    if i == idx_i{
                        // We wan't to skip last layernode, which is idx -> layernode
                        continue;
                    }
                    let eConn = nodes[i].neighbors.clone();
                    if eConn.len() > M{
                        let eNewConn = self.select_neighbors(&q, eConn, l_c, true);
                        nodes[i].neighbors = eNewConn;
                    }

                }

                self.db.upsert_neighbors(nodes).expect("Error upserting neighbors");

            }
        }

        for i in self.num_layers..=l {

            self.db.upsert_neighbor(LayerNode::new(i, idx)).expect("Error upserting neighbor");
            self.ep = Some(idx as u32);
            self.db.set_ep(idx).expect("Error setting ep");
            self.num_layers += 1;
        }

        self.db.set_num_layers(self.num_layers).expect("Error setting num_layers");

        Ok(())

    }

    fn search_layer(&mut self, q: &Vec<f32>, ep: HashMap<u32, f32>, ef: usize, l_c: usize)->HashMap<u32, f32>{

        let mut v = HashSet::new();
        let _ = ep.keys().into_iter().map(|x| v.insert(x.clone()));

        let mut C = ep.clone().into_minheap();
        let mut W = ep.into_maxheap();

        while !C.is_empty() {
            let c = C.pop().unwrap().0;
            let f_value = W.peek().unwrap().0 .0;

            if c.0.0 > f_value {
                break;
            }

            let layernd_ = self.db.get_neighbor(l_c, c.1 as usize);
            if layernd_.is_err(){
                print!("Error getting neighbor");
                continue;
            }
            let layernd = layernd_.unwrap();

            let neighbors: Vec<u32> = layernd.neighbors
                .into_iter()
                .filter_map(|x| if !v.contains(&x.0) { Some(x.0) } else { None })
                .collect();

            let points = self.db.get_points(neighbors.clone()).expect("Error getting points");
            let distances = points.iter().map(|x|
                unsafe {cosine_distance_aarch(&q, &x.v)}
            ).collect::<Vec<f32>>();

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

        if ef == 1{
            if W.len() > 0{
                let W_map = W.into_map();
                let mut W_min = W_map.into_minheap();
                let mut single_map = HashMap::new();
                let min_val = W_min.pop().unwrap().0;
                single_map.insert(min_val.1, min_val.0.0);
                return single_map;
            }
            else{
                return HashMap::new()
            }
        }

        W.into_map()
    }


    fn select_neighbors(&self, q: &Vec<f32>, C: HashMap<u32, f32>, l_c: usize, k_p_c: bool)->HashMap<u32, f32>{

        let mut R = create_min_heap();
        let mut W = C.into_minheap();

        let mut M = 0;

        if l_c > 0{
            M = self.m
        }
        else{
            M = self.m_max0;
        }

        let mut W_d = create_min_heap();
        while W.len() > 0 && R.len() < M {
            let e = W.pop().unwrap().0;

            if R.len() == 0 || e.0 < R.peek().unwrap().0.0{
                R.push(Reverse(e));
            }
            else{
                W_d.push(Reverse(e));
            }
        }

        if k_p_c{
            while W_d.len() > 0 && R.len() < M{
                R.push(W_d.pop().unwrap());
            }
        }
    R.into_map()
    }

    pub fn knn_search(&mut self, q: &Vec<f32>, K: usize, re_rank: bool) -> Vec<Value> {
        let mut W = HashMap::new();

        let ep_index = self.db.get_ep().expect("") as u32;
        self.num_layers = self.db.get_num_layers().expect("Error getting num_layers");

        let points = self.db.get_points(vec![ep_index]).expect("Error getting points");
        let point = points.first().unwrap();
        let dist = unsafe { cosine_distance_aarch(&q, &point.v) };
        let mut ep = HashMap::from([(ep_index, dist)]);

        for l_c in (1..=self.num_layers - 1).rev() {
            W = self.search_layer(&q, ep, 1, l_c);
            ep = W;
        }

        let ep_ = self.search_layer(q, ep, self.ef, 0);
        let mut heap = ep_.into_minheap();
        let mut sorted_vec = Vec::new();
        while !heap.is_empty() && sorted_vec.len() < K{
            let item = heap.pop().unwrap().0;
            sorted_vec.push((item.1, 1.0 - item.0.0));
        }
        let indices = sorted_vec.iter().map(|x| x.0).collect::<Vec<u32>>();
        let metadata = self.db.get_metadatas(indices).expect("Error getting metadatas");

        let result = sorted_vec.iter().zip(metadata.iter()).map(|(x, y)| json!({"id":x.0, "score":x.1, "metadata":y.clone()})).collect::<Vec<Value>>();
        result


    }
}