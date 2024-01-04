use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use dria_hnsw::worker::{fetch, get_health_status, insert, query};
use std::sync::Arc;
use tokio::sync::Mutex;
//use lambda_web::{run_actix_on_lambda};
//use userembeddings::middlewares::ext_client::ExternalClient;

pub fn config(conf: &mut web::ServiceConfig) {
    conf.service(get_health_status);
    conf.service(query);
    conf.service(insert);
    conf.service(fetch);
}

/*
TODO:
Middleware, token auth
Remove (mark nodes as unsearchable)
parallelism**
 */

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let factory = move || {
        App::new()
            //.wrap(ExternalClient)
            .configure(config)
            .wrap(Logger::default())
            .wrap(Cors::permissive())
    };

    println!("Dria HNSW listening...");
    HttpServer::new(factory).bind("0.0.0.0:8080")?.run().await?;
    Ok(())
}
