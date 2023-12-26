extern crate redis;

use std::cmp::Reverse;
use hashbrown::HashSet;
use ahash::AHashMap;
use std::collections::HashMap;
use std::fmt::format;

use redis::Commands;

use rand::{Rng, thread_rng, SeedableRng};

use crate::proto::index_buffer::{Point, LayerNode};
use prost::Message;
use prost_types::Any; // For handling the Any type

use crate::errors::errors::{DeserializeError};
use crate::hnsw::utils::{create_min_heap, create_max_heap, IntoHeap, IntoMap, Numeric};

use serde_json::{json, Value};
use tokio::time::Instant;
use crate::db::redis_client::RedisClient;
use rayon::prelude::*;
use crate::hnsw::scalar::ScalarQuantizer;
use crate::distance::metric::{Metric, MetricL2, MetricCosine, MetricL1};

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
    pub db: RedisClient,
    quantizer: ScalarQuantizer,
    metric: Option<String>
}

impl HNSW{

    pub fn new(M: usize, ef_construction:usize, ef:usize, contract_id: String, metric: Option<String>) -> HNSW{

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

        let sq = ScalarQuantizer::new(256, 1000, 256);


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
            db,
            quantizer:sq,
            metric
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

    fn distance(&self, x: &[f32], y: &[f32], dist: &Option<String>) -> f32 {
        match dist.as_ref().map(String::as_str) {
            Some("l2") => unsafe { MetricL2::compare(x, y) },
            Some("l1") => unsafe { MetricL1::compare(x, y) },
            Some("cosine") | None => unsafe { MetricCosine::compare(x, y) },
            _ => panic!("Unsupported distance metric"),
        }
    }

    pub fn insert_w_preset(&mut self, q: Vec<f32>, idx: usize)->Result<(), DeserializeError>{

        let mut W = HashMap::new();
        let ep_index = self.ep.clone();
        self.num_layers = self.db.get_num_layers().expect("Error getting num_layers") as usize;
        let L = if self.num_layers == 0 { 0 } else { self.num_layers - 1 };
        let l = self.select_layer();

        if ep_index.is_some(){

            let ep_index_ = ep_index.unwrap();
            let points = self.db.get_points(&vec![ep_index_]).expect("Error getting points");
            let point = points.first().unwrap();
            let dist = self.distance(&q, &point.v, &self.metric);
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

        //println!("************");
        Ok(())

    }

    pub fn insert(&mut self, q: Vec<f32>, metadata: Value)->Result<(), DeserializeError>{

        let mut W = HashMap::new();
        let ep_index = self.ep.clone();
        self.num_layers = self.db.get_num_layers().expect("Error getting num_layers") as usize;
        let L = if self.num_layers == 0 { 0 } else { self.num_layers - 1 };
        let l = self.select_layer();

        //get time with chrono

        self.data_size = self.db.get_datasize().expect("Error getting datasize") as usize;
        let idx = self.data_size;


        self.db.add_points(q.clone(), idx).expect("Error adding points");
        self.db.set_metadata(metadata, idx).expect("Error setting metadata");


        let ds = self.db.get_datasize().expect("Error getting datasize") as usize;
        self.db.set_datasize(ds + 1).expect("Error setting datasize");

        if ep_index.is_some(){

            let ep_index_ = ep_index.unwrap();
            let points = self.db.get_points(&vec![ep_index_]).expect("Error getting points");
            let point = points.first().unwrap();
            let dist = self.distance(&q, &point.v, &self.metric);
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

        //println!("************");
        Ok(())

    }

    fn search_layer(&mut self, q: &Vec<f32>, ep: HashMap<u32, f32>, ef: usize, l_c: usize)->HashMap<u32, f32>{


        let mut v = HashSet::new();
        let _ = ep.keys().into_iter().map(|x| v.insert(x.clone()));
        /*
        let mut visited_list = self.vlp.get_free_visited_list();
        for key in ep.keys(){
            visited_list.mass[key.clone() as usize] = visited_list.cur_v as u16;
        }
         */
        let mut memory_node: AHashMap<String, LayerNode> = AHashMap::new();
        let mut memory_points:AHashMap<String, Point> = AHashMap::new();
        //let mut memory_node:DashMap<String, LayerNode> = DashMap::new();
        //let mut memory_points:DashMap<String, Point> = DashMap::new();

        let mut C = ep.clone().into_minheap();
        let mut W = ep.into_maxheap();

        while !C.is_empty() {

            let c = C.pop().unwrap().0;
            let f_value = W.peek().unwrap().0 .0;

            if c.0.0 > f_value {
                break;
            }

            let node_key = format!("{}:{}", l_c, c.1);
            let layernd = match memory_node.get(&node_key) {
                Some(node) => node.clone(), // Clone the node if it exists in memory
                None => {
                    // Fetch from the database if it doesn't exist in memory
                    let node_result = self.db.get_neighbor(l_c, c.1 as usize);
                    match node_result {
                        Ok(node) => {
                            // Insert the fetched node into memory and return it
                            memory_node.insert(node_key, node.clone());
                            node
                        },
                        Err(_) => {
                            println!("Error getting neighbor");
                            continue;
                        }
                    }
                }
            };
            //let layernd_ = self.db.get_neighbor(l_c, c.1 as usize);
            //let layernd = value.unwrap();


            let neighbors: Vec<u32> = layernd.neighbors
                .into_iter()
                .filter_map(|x| if !v.contains(&x.0) { Some(x.0) } else { None })
                .collect();
            /*
            let neighbors: Vec<u32> = layernd.neighbors
                .into_iter()
                .filter_map(|x| if visited_list.mass[x.0.clone() as usize]  != visited_list.cur_v as u16 { // Check if not visited
                    Some(x.0)
                } else { None })
                .collect();

             */

            //let points = self.db.get_points(neighbors.clone()).expect("Error getting points");

            let mut points_to_fetch = Vec::new();
            let mut points = Vec::new();

            // Collect the neighbors that need to be fetched from the database
            for &neighbor in neighbors.iter() {
                let neighbor_key = neighbor.to_string();
                if !memory_points.contains_key(&neighbor_key) {
                    points_to_fetch.push(neighbor);
                }
            }

            // Fetch the points from the database if there are any to fetch
            if !points_to_fetch.is_empty() {
                match self.db.get_points(&points_to_fetch) {
                    Ok(fetched_points) => {
                        // Update the cache and collect points
                        for (neighbor, point) in points_to_fetch.iter().zip(fetched_points.iter()) {
                            let neighbor_key = neighbor.to_string();
                            memory_points.insert(neighbor_key.clone(), point.clone());
                            points.push(point.clone());
                        }
                    },
                    Err(_) => {
                        println!("Error getting points for neighbors");
                        // Handle the error appropriately
                    }
                }
            }

            // Add the points from the cache
            for &neighbor in neighbors.iter() {
                let neighbor_key = neighbor.to_string();
                if let Some(point) = memory_points.get(&neighbor_key) {
                    points.push(point.clone());
                }
            }

            let distances = points.iter().map(|x|
                self.distance(&q, &x.v, &self.metric)
            ).collect::<Vec<f32>>();

            for (i, d) in neighbors.iter().zip(distances.iter()) {
                v.insert(i.clone());
                //visited_list.mass[*i as usize] = 1;
                if d < &f_value || W.len() < ef {
                    C.push(Reverse((Numeric(d.clone()), i.clone())));
                    W.push((Numeric(d.clone()), i.clone()));
                    if W.len() > ef {
                        W.pop();
                    }
                }
            }
        }

        //self.vlp.release_visited_list(visited_list);

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

    pub fn knn_search(&mut self, q: &Vec<f32>, K: usize) -> Vec<Value> {
        let mut W = HashMap::new();

        let ep_index = self.db.get_ep().expect("") as u32;
        self.num_layers = self.db.get_num_layers().expect("Error getting num_layers");

        let points = self.db.get_points(&vec![ep_index]).expect("Error getting points");
        let point = points.first().unwrap();
        let dist = self.distance(&q, &point.v, &self.metric);
        let mut ep = HashMap::from([(ep_index, dist)]);
        let st = Instant::now();

        for l_c in (1..=self.num_layers - 1).rev() {
            W = self.search_layer(&q, ep, 1, l_c);
            ep = W;
        }

        print!("Search time 1 : {} ms\n", st.elapsed().as_millis());


        let st = Instant::now();

        let ep_ = self.search_layer(q, ep, self.ef, 0);

        print!("Search time 2: {} ms\n", st.elapsed().as_millis());
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