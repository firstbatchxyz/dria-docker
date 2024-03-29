use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub struct Config {
    env: String,
    debug: bool,
    pk_length: u32,
    sk_length: u32,
    rate_limit: String,
    global_rate_limit: String,
    logging_level: String,
    pub contract_id: String,
    pub redis_url: String,
    pub port: String,
    pub rocksdb_path: String,
}

impl Config {
    pub fn new() -> Config {
        let rocksdb_path = match env::var("ROCKSDB_PATH") {
            Ok(val) => val,
            Err(_) => "/tmp/rocksdb".to_string(),
        };

        let port = match env::var("PORT") {
            Ok(val) => val,
            Err(_) => "8080".to_string(),
        };

        let contract_id = match env::var("CONTRACT_ID") {
            Ok(val) => val,
            Err(_) => {
                println!("CONTRACT_ID not found, using default");
                "default".to_string()
            }
        };

        Config {
            env: "development".to_string(),
            debug: true,
            pk_length: 0,
            sk_length: 0,
            rate_limit: "".to_string(),
            global_rate_limit: "".to_string(),
            logging_level: "DEBUG".to_string(),
            contract_id,
            redis_url: "redis://127.0.0.1/".to_string(),
            port,
            rocksdb_path,
        }
    }
}
