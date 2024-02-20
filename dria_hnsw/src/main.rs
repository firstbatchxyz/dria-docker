use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
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

    let factory = move || {
        App::new()
            .app_data(web::JsonConfig::default().limit(152428800))
            .app_data(node_cache.clone())
            .app_data(point_cache.clone())
            .configure(config)
            .wrap(Logger::default())
            .wrap(Cors::permissive())
    };

    println!("Dria HNSW listening...");
    HttpServer::new(factory).bind("0.0.0.0:8082")?.run().await?;
    Ok(())
}
