use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use dria_hnsw::db::env::Config;
use dria_hnsw::db::rocksdb_client::RocksdbClient;
use dria_hnsw::middlewares::cache::{NodeCache, PointCache};
use dria_hnsw::worker::{fetch, get_health_status, insert_vector, query};

pub fn config(conf: &mut web::ServiceConfig) {
    conf.service(get_health_status);
    conf.service(query);
    conf.service(fetch);
    conf.service(insert_vector);
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let node_cache = web::Data::new(NodeCache::new());
    let point_cache = web::Data::new(PointCache::new());
    let cfg = Config::new();

    let rocksdb_client = RocksdbClient::new(cfg.contract_id.clone());

    if rocksdb_client.is_err() {
        println!("Rocksdb client failed to initialize");
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Rocksdb client failed to initialize",
        ));
    }
    let rdb = rocksdb_client.unwrap();
    let rocksdb_client = web::Data::new(rdb);

    let factory = move || {
        App::new()
            .app_data(web::JsonConfig::default().limit(152428800))
            .app_data(node_cache.clone())
            .app_data(rocksdb_client.clone())
            .app_data(point_cache.clone())
            .configure(config)
            .wrap(Logger::default())
            .wrap(Cors::permissive())
    };

    let url = format!("0.0.0.0:{}", cfg.port);
    println!("Dria HNSW listening at {}", url);
    HttpServer::new(factory).bind(url)?.run().await?;
    Ok(())
}
