use crate::hnsw::sync_map::SynchronizedNodes;
use crate::proto::index_buffer::Point;
use dashmap::DashMap;
use std::sync::Arc;

use mini_moka::sync::Cache;
use std::time::Duration;

pub struct NodeCache {
    //pub caches: Arc<DashMap<String, Arc<SynchronizedNodes>>>
    pub caches: Cache<String, Arc<SynchronizedNodes>>,
}

impl NodeCache {
    pub fn new() -> Self {
        let cache = Cache::builder()
            //if a key is not used (get or insert) for 2 hour, expire it
            .time_to_idle(Duration::from_secs(4 * 60 * 60))
            .max_capacity(1_000)
            .build();

        NodeCache { caches: cache }
    }

    pub fn get_cache(&self, key: String) -> Arc<SynchronizedNodes> {
        let my_cache = self.caches.clone();
        //let cache = my_cache.entry(key.to_string()).or_insert_with(|| Arc::new(SynchronizedNodes::new()));

        let node_cache = my_cache.get(&key).unwrap_or_else(|| {
            let new_cache = Arc::new(SynchronizedNodes::new());
            my_cache.insert(key.to_string(), new_cache.clone());
            new_cache
        });
        node_cache.clone()
    }

    pub fn add_cache(&self, key: &str, cache: Arc<SynchronizedNodes>) {
        let my_cache = self.caches.clone();
        my_cache.insert(key.to_string(), cache);
    }
}

pub struct PointCache {
    pub caches: Cache<String, Arc<DashMap<String, Point>>>,
}

impl PointCache {
    pub fn new() -> Self {
        let cache = Cache::builder()
            //if a key is not used (get or insert) for 2 hour, expire it
            .time_to_idle(Duration::from_secs(4 * 60 * 60))
            .max_capacity(1_000) // around 106MB for 1536 dim vectors
            .build();

        PointCache { caches: cache }
    }

    pub fn get_cache(&self, key: String) -> Arc<DashMap<String, Point>> {
        //let cache = self.caches.entry(key.to_string()).or_insert_with(|| Arc::new(DashMap::new()));
        let my_cache = self.caches.clone();
        let point_cache = my_cache.get(&key).unwrap_or_else(|| {
            let new_cache = Arc::new(DashMap::new());
            my_cache.insert(key.to_string(), new_cache.clone());
            new_cache
        });
        point_cache.clone()
    }

    pub fn add_cache(&self, key: &str, cache: Arc<DashMap<String, Point>>) {
        self.caches.insert(key.to_string(), cache);
    }
}
