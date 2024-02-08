use crate::db::env::Config;
use crate::hnsw::index::HNSW;
use crate::middlewares::cache::{NodeCache, PointCache};
use crate::models::request_models::{FetchModel, InsertModel, QueryModel};
use crate::responses::responses::CustomResponse;
use actix_web::web::Json;
use actix_web::{get, post, web, HttpMessage, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::time::Instant;

#[get("/health")]
pub async fn get_health_status() -> HttpResponse {
    let response = CustomResponse {
        success: true,
        data: "hello world!".to_string(),
        code: 200,
    };
    HttpResponse::Ok().json(response)
}

#[post("/dria/query")]
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
            let res = ind.knn_search(&payload.vector, payload.top_n, false, node_map, point_map);

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

#[post("/dria/fetch")]
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

fn ef_helper(ef: Option<usize>) -> usize {
    let level = ef.clone().unwrap_or(1);
    20 + (level * 30)
}
