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
    pub redis_url: String,
    pub rocksdb_path: String,
}

impl Config {
    pub fn new() -> Config {
        match env::var("ROCKSDB_PATH") {
            Ok(rocksdb_path) => Config {
                env: "development".to_string(),
                debug: true,
                pk_length: 0,
                sk_length: 0,
                rate_limit: "".to_string(),
                global_rate_limit: "".to_string(),
                logging_level: "DEBUG".to_string(),
                redis_url: "redis://127.0.0.1/".to_string(),
                rocksdb_path,
            },
            Err(_) => Config {
                env: "development".to_string(),
                debug: true,
                pk_length: 0,
                sk_length: 0,
                rate_limit: "".to_string(),
                global_rate_limit: "".to_string(),
                logging_level: "DEBUG".to_string(),
                redis_url: "redis://127.0.0.1/".to_string(),
                rocksdb_path: "/tmp/rocksdb".to_string(),
            },
        }
    }
}
