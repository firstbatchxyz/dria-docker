use crate::models::request_models::{QueryModel, InsertModel, FetchModel};
use actix_web::{post ,get, web, HttpResponse, HttpRequest, HttpMessage};
use actix_web::web::Json;
use crate::responses::responses::CustomResponse;
use crate::hnsw::index::HNSW;
use std::time::{Instant};
use serde_json::{json, Value};

use serde::{Serialize, Deserialize};
use crate::db::env::Config;
use std::env;

#[get("/health")]
pub async fn get_health_status() -> HttpResponse {
    let response = CustomResponse {
        success: true,
        data: "hello world!".to_string(),
        code: 200,
    };
    HttpResponse::Ok().json(response)
}

#[get("/dria/health")]
pub async fn get_health_status2() -> HttpResponse {
    let response = CustomResponse {
        success: true,
        data: "hello world!".to_string(),
        code: 200,
    };
    HttpResponse::Ok().json(response)
}

#[post("/dria/query")]
pub async fn query(req:HttpRequest, payload: Json<QueryModel>) -> HttpResponse {

    let mut ind = HNSW::new(16, 128, 0, "".to_string(), None);
    match env::var("CONTRACT_ID") {
        Ok(val) => {
            ind = HNSW::new(16, 128, ef_helper(payload.level), val, None);
        },
        Err(e) => {
            let response = CustomResponse {
                success: false,
                data: "Contract ID not found inside env variables",
                code: 400,
            };
            return HttpResponse::Forbidden().json(response)
        }
    }


    let res = ind.knn_search(&payload.vector, payload.top_n);

    let response = CustomResponse {
        success: true,
        data: json!(res),
        code: 200,
    };
    HttpResponse::Ok().json(response)

}

#[post ("/dria/fetch")]
pub async fn fetch(req:HttpRequest, payload:Json<FetchModel>) -> HttpResponse{

    let mut ind = HNSW::new(16, 128, 0, "".to_string(), None);
    match env::var("CONTRACT_ID") {
        Ok(val) => {
            ind = HNSW::new(16, 128, 0, val, None);
        },
        Err(e) => {
            let response = CustomResponse {
                success: false,
                data: "Contract ID not found inside env variables",
                code: 400,
            };
            return HttpResponse::Forbidden().json(response)
        }
    }
        let res = ind.db.get_metadatas(payload.id.clone());

        if res.is_err(){
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

#[post("/dria/insert")]
pub async fn insert(req:HttpRequest, payload: Json<InsertModel>) -> HttpResponse {


    let mut ind = HNSW::new(16, 128, 0, "".to_string(), None);
    match env::var("CONTRACT_ID") {
        Ok(val) => {
            ind = HNSW::new(16, 128, 20, val, None);
        },
        Err(e) => {
            let response = CustomResponse {
                success: false,
                data: "Contract ID not found inside env variables",
                code: 400,
            };
            return HttpResponse::Forbidden().json(response)
        }
    }

    let metadata = payload.metadata.clone().unwrap_or(json!({}));
    ind.insert(payload.vector.clone(), metadata).expect("Error inserting");

    let response = CustomResponse {
        success: true,
        data: "Success".to_string(),
        code: 200,
    };
    HttpResponse::Ok().json(response)

}

fn ef_helper(ef: Option<usize>)->usize{
    let level = ef.clone().unwrap_or(1);
    level * 100
}


