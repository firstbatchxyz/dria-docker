use crate::db::conversions::{
    base64_to_batch_str, base64_to_batch_vec, base64_to_singleton_str, base64_to_singleton_vec,
};
use crate::db::env::Config;
use crate::db::rocksdb_client::RocksdbClient;
use crate::hnsw::index::HNSW;
use crate::hnsw::sync_map::SynchronizedNodes;
use crate::middlewares::cache::{NodeCache, PointCache};
use crate::models::request_models::{FetchModel, InsertBatchModel, QueryModel};
use crate::proto::index_buffer::{LayerNode, Point};
use crate::responses::responses::CustomResponse;
use actix_web::http::StatusCode;
use actix_web::web::Json;
use actix_web::{get, post, web, HttpMessage, HttpRequest, HttpResponse};
use dashmap::DashMap;
use futures_util::future::err;
use log::{debug, error, info, trace, warn};
use mini_moka::sync::Cache;
use rayon::prelude::*;
use rayon::ThreadPool;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::borrow::Borrow;
use std::env;
use std::sync::atomic::{AtomicIsize, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::task;

pub const SINGLE_THREADED_HNSW_BUILD_THRESHOLD: usize = 256;

#[get("/health")]
pub async fn get_health_status() -> HttpResponse {
    let response = CustomResponse {
        success: true,
        data: "hello world!".to_string(),
        code: 200,
    };
    HttpResponse::Ok().json(response)
}

#[post("/query")]
pub async fn query(req: HttpRequest, payload: Json<QueryModel>) -> HttpResponse {
    let mut ind: HNSW;
    match env::var("CONTRACT_ID") {
        Ok(val) => {
            ind = HNSW::new(16, 128, ef_helper(payload.level), val.clone(), None);
            let node_cache = req
                .app_data::<web::Data<NodeCache>>()
                .expect("Error getting node cache"); //Arc<SynchronizedNodes> = Arc::new(SynchronizedNodes::new());
            let point_cache = req
                .app_data::<web::Data<PointCache>>()
                .expect("Error getting point cache"); //Arc<DashMap<String, Point>> = Arc::new(DashMap::new());

            let node_map = node_cache.get_cache(val.clone()); //Arc<SynchronizedNodes> = Arc::new(SynchronizedNodes::new());
            let point_map = point_cache.get_cache(val.clone());
            let res = ind.knn_search(&payload.vector, payload.top_n, node_map, point_map);

            let response = CustomResponse {
                success: true,
                data: json!(res),
                code: 200,
            };
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            let response = CustomResponse {
                success: false,
                data: "Contract ID not found inside env variables",
                code: 400,
            };
            return HttpResponse::Forbidden().json(response);
        }
    }
}

#[post("/fetch")]
pub async fn fetch(req: HttpRequest, payload: Json<FetchModel>) -> HttpResponse {
    let mut ind: HNSW;
    match env::var("CONTRACT_ID") {
        Ok(val) => {
            ind = HNSW::new(16, 128, 0, val, None);
        }
        Err(e) => {
            let response = CustomResponse {
                success: false,
                data: "Contract ID not found inside env variables",
                code: 400,
            };
            return HttpResponse::Forbidden().json(response);
        }
    }
    let res = ind.db.get_metadatas(payload.id.clone());

    if res.is_err() {
        let response = CustomResponse {
            success: false,
            data: "Error fetching metadata".to_string(),
            code: 500,
        };
        return HttpResponse::InternalServerError().json(response);
    }

    let res = res.unwrap();

    let response = CustomResponse {
        success: true,
        data: json!(res),
        code: 200,
    };
    HttpResponse::Ok().json(response)
}

#[post("/insert_vector")]
pub async fn insert_vector(req: HttpRequest, payload: Json<InsertBatchModel>) -> HttpResponse {
    let cid = match env::var("CONTRACT_ID") {
        Ok(cid) => cid,
        Err(e) => {
            let response = CustomResponse {
                success: false,
                data: "Contract ID not found inside env variables",
                code: 400,
            };
            return HttpResponse::Forbidden().json(response);
        }
    };

    let data = base64_to_batch_vec(&payload.data);
    let cfg = Config::new();
    let mut vectors = Vec::new();
    let mut metadata_batch = Vec::new();
    for d in data.s.iter() {
        let metadata = json!(d.map.clone());
        vectors.push(d.v.clone()); //this should not be clone
        metadata_batch.push(metadata);
    }

    if vectors.len() > 1000 {
        let response = CustomResponse {
            success: false,
            data: "Batch size should be smaller than 1000.".to_string(),
            code: 401,
        };
        return HttpResponse::InternalServerError().json(response);
    }

    let node_cache = req
        .app_data::<web::Data<NodeCache>>()
        .expect("Error getting node cache"); //Arc<SynchronizedNodes> = Arc::new(SynchronizedNodes::new());
    let point_cache = req
        .app_data::<web::Data<PointCache>>()
        .expect("Error getting point cache"); //Arc<DashMap<String, Point>> = Arc::new(DashMap::new());

    let node_map = node_cache.get_cache(cid.clone()); //Arc<SynchronizedNodes> = Arc::new(SynchronizedNodes::new());
    let point_map = point_cache.get_cache(cid.clone()); //Arc<DashMap<String, Point>> = Arc::new(DashMap::new());
    let cid_clone = cid.clone();
    let result = task::spawn_blocking(move || {
        train_worker(
            vectors,
            metadata_batch,
            node_map,
            point_map,
            10_000,
            cid.clone()
        )
    }).await;

    let node_map = node_cache.get_cache(cid_clone); //Arc<SynchronizedNodes> = Arc::new(SynchronizedNodes::new());
    node_map.reset();

    let response = CustomResponse {
        success: true,
        data: "Values are successfully added to index.".to_string(),
        code: 200,
    };
    HttpResponse::Ok().json(response)
}

fn ef_helper(ef: Option<usize>) -> usize {
    let level = ef.clone().unwrap_or(1);
    20 + (level * 30)
}

fn train_worker(
    vectors: Vec<Vec<f32>>,
    metadata_batch: Vec<Value>,
    node_map: Arc<SynchronizedNodes>,
    point_map: Cache<String, Point>,
    batch_size: usize,
    contract_id: String,
) -> (String, u16) {
    //let rdb = RocksdbClient::new(contract_id.clone()).unwrap();

    let ind = HNSW::new(16, 128, ef_helper(Some(1)), contract_id.clone(), None);

    let mut ds = 0;
    let nl = ind.db.get_num_layers();
    let num_layers = Arc::new(AtomicUsize::new(0));

    if nl.is_err() {
        error!("{}", nl.err().unwrap());
        ind.db.set_datasize(0).expect("Error setting datasize");
    } else {
        let nl_value = nl.expect("").clone();
        ds = ind.db.get_datasize().expect("Error getting datasize");

        let res = num_layers.fetch_update(Ordering::SeqCst, Ordering::Relaxed, |x| Some(nl_value));

        if res.is_err() {
            error!("{}", res.err().unwrap());
            return ("Error setting num layers, atomic".to_string(), 500);
        }
    }

    let r1 = ind.db.add_points_batch(&vectors, ds);
    let r2 = ind.db.set_metadata_batch(metadata_batch, ds);
    let r3 = ind.db.set_datasize(ds + vectors.len());

    if r1.is_err() || r2.is_err() || r3.is_err() {
        error!("Error adding points as batch");
        return ("Error adding points as batch".to_string(), 500);
    }

    let epa = Arc::new(AtomicIsize::new(-1));

    let ep = ind.db.get_ep();

    if ep.is_ok() {
        let ep_value = ep.expect("").clone();
        let res = epa.fetch_update(Ordering::SeqCst, Ordering::Relaxed, |x| {
            // Your update logic here
            Some(ep_value as isize)
        });
        if res.is_err() {
            error!("{}", res.err().unwrap());
            return ("Error setting ep, atomic".to_string(), 500);
        }
    }
    let pool = rayon::ThreadPoolBuilder::new()
        .thread_name(|idx| format!("hnsw-build-{idx}"))
        .num_threads(8)
        .build()
        .expect("Error building threadpool");

    if ds < SINGLE_THREADED_HNSW_BUILD_THRESHOLD {
        let iter_ind = vectors.len().min(SINGLE_THREADED_HNSW_BUILD_THRESHOLD - ds);
        for i in 0..iter_ind {
            ind.insert_w_preset(
                ds + i,
                node_map.clone(),
                point_map.clone(),
                num_layers.clone(),
                epa.clone(),
            )
            .expect("Error inserting");
        }

        if SINGLE_THREADED_HNSW_BUILD_THRESHOLD < (vectors.len() + ds) {
            pool.install(|| {
                (iter_ind..vectors.len())
                    .into_par_iter()
                    .try_for_each(|item| {
                        ind.insert_w_preset(
                            ds + item,
                            node_map.clone(),
                            point_map.clone(),
                            num_layers.clone(),
                            epa.clone(),
                        )
                    })
            })
            .expect("Error inserting");
        }
    } else {
        pool.install(|| {
            (0..vectors.len()).into_par_iter().try_for_each(|item| {
                ind.insert_w_preset(
                    item + ds,
                    node_map.clone(),
                    point_map.clone(),
                    num_layers.clone(),
                    epa.clone(),
                )
            })
        })
        .expect("Error inserting");
    }

    //replicate neighbors
    let values: Vec<LayerNode> = node_map
        .clone()
        .map
        .iter()
        .map(|entry| entry.value().clone())
        .collect();

    let ep_value = epa.clone().load(Ordering::Relaxed);
    let num_layers = num_layers.clone().load(Ordering::Relaxed);

    let batch_size: usize = batch_size;

    for chunk in values.chunks(batch_size) {
        let r1 = ind.db.upsert_neighbors(chunk.to_vec());
        if r1.is_err() {
            error!("{}", r1.err().unwrap());
            return ("Error writing batch to blockchain".to_string(), 500);
        }
    }
    let r2 = ind.db.set_ep(ep_value as usize, false);
    let r3 = ind.db.set_num_layers(num_layers, false);

    if r2.is_err() || r3.is_err() {
        error!("Error writing to blockchain");
        return ("Error writing batch to blockchain".to_string(), 500);
    }
    return ("Values are successfully added to index.".to_string(), 200);
}
