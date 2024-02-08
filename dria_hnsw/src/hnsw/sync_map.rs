use crate::proto::index_buffer::LayerNode;
use crossbeam_channel::{bounded, Receiver, Sender};
use dashmap::DashMap;
use hashbrown::HashSet;
use std::collections::HashMap;
use std::sync::Arc;
//use tokio::sync::{Mutex, RwLock, RwLockWriteGuard};
use mini_moka::sync::Cache;
use std::time::Duration;

use parking_lot::{Mutex, RwLock, RwLockWriteGuard};

pub struct SynchronizedNodes {
    pub map: Cache<String, LayerNode>, //Arc<DashMap<String, LayerNode>>,
    pub lock_map: Arc<DashMap<String, RwLock<()>>>,
    wait_map: Mutex<HashMap<String, (Sender<()>, Receiver<()>)>>,
}

impl SynchronizedNodes {
    pub fn new() -> Self {
        let cache = Cache::builder()
            //if a key is not used (get or insert) for 2 hour, expire it
            .time_to_idle(Duration::from_secs(4 * 60 * 60))
            .max_capacity(1_000_000)
            .build();

        SynchronizedNodes {
            map: cache, //Arc::new(DashMap::with_capacity(1_000_000)), //this makes to 50mb of memory for 32 mmax0
            lock_map: Arc::new(DashMap::new()),
            wait_map: Mutex::new(HashMap::new()),
        }
    }

    pub fn insert_and_notify(&self, node: &LayerNode) {
        let key = format!("{}:{}", node.level, node.idx);

        let node_lock = self
            .lock_map
            .entry(key.clone())
            .or_insert_with(|| RwLock::new(()));
        let _write_guard = node_lock.write();

        // Lock guard on wait_map for atomic check-and-notify
        //let mut wait_map_guard = self.wait_map.lock().unwrap();

        // Insert or update the node in the DashMap
        self.map.insert(key.clone(), node.clone());
        drop(_write_guard);
        // Notify all waiting threads registered for this key
        let mut wait_map_guard = self.wait_map.lock();
        if let Some((sender, _receiver)) = wait_map_guard.remove(&key) {
            // You can drop the lock here since the channel will be removed
            // This avoid deadlocks if the receiving end tries to acquire the same lock.
            drop(wait_map_guard);
            let _ = sender.send(()); // It's safe to ignore the send result
        }
    }

    pub fn insert_batch_and_notify(&self, nodes: Vec<LayerNode>) {
        //let mut wait_map_guard = self.wait_map.lock().unwrap();
        let mut keys_to_notify = HashSet::new();

        for node in nodes.iter() {
            let key = format!("{}:{}", node.level, node.idx);

            let node_lock = self
                .lock_map
                .entry(key.clone())
                .or_insert_with(|| RwLock::new(()));
            let _write_guard = node_lock.write(); // Lock for writing

            self.map.insert(key.clone(), node.clone());
            keys_to_notify.insert(key);
            drop(_write_guard);
        }

        // Now, after all nodes have been processed, issue the notifications.
        let mut wait_map_guard = self.wait_map.lock();
        for key in keys_to_notify {
            if let Some((sender, reciever)) = wait_map_guard.get(&key) {
                let _ = sender.send(()); // It's safe to ignore the send result
            }
        }
    }

    pub fn get_or_wait(&self, key: &str) -> LayerNode {
        loop {
            if let Some(value) = self.map.get(&key.to_string()) {
                return value.clone();
            }

            // Register for notification before checking if a node is being inserted
            let receiver = self.register_for_notification(key);

            receiver.recv().unwrap(); // Block the thread until notification is received
        }
    }

    pub fn get_or_wait_opt(&self, key: &str) -> Option<LayerNode> {
        loop {
            if let Some(value) = self.map.get(&key.to_string()) {
                return Some(value.clone());
            }

            // Check if this key is expected to be updated soon
            let receiver = {
                let wait_map_guard = self.wait_map.lock();
                if let Some((_sender, receiver)) = wait_map_guard.get(key) {
                    Some(receiver.clone()) // Clone the receiver
                } else {
                    None // Key is not expected to be updated soon
                }
            };

            if let Some(receiver) = receiver {
                // Wait for the notification if the key is expected to be updated
                match receiver.recv() {
                    Ok(_) => {
                        // Handle the received message
                    }
                    Err(e) => {
                        // Handle the error, e.g., log it or perform a fallback action
                        eprintln!("Error receiving message: {:?}", e);
                        return None;
                    }
                }
            } else {
                // If the key is not expected to be updated soon, return None
                return None;
            }
        }
    }

    pub fn register_for_notification(&self, key: &str) -> Receiver<()> {
        let mut wait_map_guard = self.wait_map.lock();
        if !wait_map_guard.contains_key(key) {
            // Create a new sender/receiver pair if it doesn't exist
            let (sender, receiver) = bounded(10);
            wait_map_guard.insert(key.to_string(), (sender, receiver.clone()));
            receiver
        } else {
            // If it already exists, return the existing receiver
            wait_map_guard.get(key).unwrap().1.clone()
        }
    }

    pub fn notify(&self, key: &str) {
        let mut wait_map_guard = self.wait_map.lock();
        if let Some((sender, _)) = wait_map_guard.remove(key) {
            drop(wait_map_guard); // Drop the lock before sending to avoid deadlocks
            let _ = sender.send(()); // It's safe to ignore the send result
        }
    }
}