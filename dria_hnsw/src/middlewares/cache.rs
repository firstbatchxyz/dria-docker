use crate::hnsw::sync_map::SynchronizedNodes;
use crate::proto::index_buffer::{Point, PointQuant};
use dashmap::DashMap;
use std::sync::Arc;

use mini_moka::sync::Cache;

use std::time::Duration;

pub struct NodeCache {
    pub caches: Cache<String, Arc<SynchronizedNodes>>,
}

impl NodeCache {
    pub fn new() -> Self {
        let cache = Cache::builder()
            //if a key is not used (get or insert) for 2 days hour, expire it
            .time_to_idle(Duration::from_secs(48 * 60 * 60))
            .max_capacity(5_000)
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
    pub caches: Cache<String, Cache<String, Point>>, //Cache<String, Arc<DashMap<String, Point>>>,
}

impl PointCache {
    pub fn new() -> Self {
        let cache = Cache::builder()
            //if a key is not used (get or insert) for 2 hour, expire it
            .time_to_idle(Duration::from_secs(24 * 60 * 60))
            .max_capacity(5_000) // around 106MB for 1536 dim vectors
            .build();

        PointCache { caches: cache }
    }

    pub fn get_cache(&self, key: String) -> Cache<String, Point> {
        //let cache = self.caches.entry(key.to_string()).or_insert_with(|| Arc::new(DashMap::new()));
        let my_cache = self.caches.clone();
        let point_cache = my_cache.get(&key).unwrap_or_else(|| {
            //let new_cache = Arc::new(DashMap::new());
            let new_cache = Cache::builder()
                //if a key is not used (get or insert) for 2 hour, expire it
                //.time_to_live(Duration::from_secs(1 * 60 * 60))
                .max_capacity(200_000) // around 2060MB for 1536 dim vectors
                .build();
            my_cache.insert(key.to_string(), new_cache.clone());
            new_cache
        });
        point_cache.clone()
    }

    pub fn add_cache(&self, key: &str, cache: Cache<String, Point>) {
        self.caches.insert(key.to_string(), cache);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn test_cache() {
        let cache = Cache::builder()
            //if a key is not used (get or insert) for 2 hour, expire it
            .time_to_idle(Duration::from_secs(120 * 60))
            .max_capacity(100)
            .build();

        for i in 0..105 {
            cache.insert(i.to_string(), i);
        }

        let ix = cache.get(&"0".to_string());
        print!("ix: {:?}", ix);

        let current_weight = AtomicU32::new(1); // Start weights from 1 to avoid assigning a weight of 0

        let cache = Cache::builder()
            .weigher(move |_key, _value: &String| -> u32 {
                // Use the current weight and increment for the next use
                current_weight.fetch_add(1, Ordering::SeqCst)
            })
            // Assuming a simple numeric weight limit for demonstration purposes
            .max_capacity(100)
            .build();

        // Example inserts - in a real scenario, make sure to manage the size and weights appropriately
        for i in 0..105 {
            cache.insert(i, format!("Value {}", i));
        }

        let ix = cache.get(&0);
        print!("ix: {:?}", ix);
    }
}
