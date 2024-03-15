use crate::proto::index_buffer::LayerNode;
use crossbeam_channel::{bounded, Receiver, Sender};
use dashmap::DashMap;
use hashbrown::HashSet;
use std::collections::HashMap;
use std::sync::Arc;
//use tokio::sync::{Mutex, RwLock, RwLockWriteGuard};
use parking_lot::{Mutex, RwLock};
use std::time::Duration;

//static ref RESET_SIZE =  120_000;

pub struct SynchronizedNodes {
    pub map: Arc<DashMap<String, LayerNode>>, //Cache<String, LayerNode>,  //Arc<DashMap<String, LayerNode>>,
    pub lock_map: Arc<DashMap<String, RwLock<()>>>,
    wait_map: Mutex<HashMap<String, (Sender<()>, Receiver<()>)>>,
}

impl SynchronizedNodes {
    pub fn new() -> Self {
        SynchronizedNodes {
            map: Arc::new(DashMap::new()), //Arc::new(DashMap::new()),cache
            lock_map: Arc::new(DashMap::new()),
            wait_map: Mutex::new(HashMap::new()),
        }
    }

    pub fn reset(&self) {
        if self.map.len() > 120_000 {
            self.map.clear();
        }
    }

    pub fn insert_and_notify(&self, node: &LayerNode) {
        let key = format!("{}:{}", node.level, node.idx);

        {
            let node_lock = self
                .lock_map
                .entry(key.clone())
                .or_insert_with(|| RwLock::new(()));

            let _write_guard = node_lock.write();
            // Insert or update the node in the DashMap
            self.map.insert(key.clone(), node.clone());
        }

        // Notify all waiting threads registered for this key
        self.notify(&key);
    }

    pub fn insert_batch_and_notify(&self, nodes: Vec<LayerNode>) {
        //let mut wait_map_guard = self.wait_map.lock().unwrap();
        let mut keys_to_notify = HashSet::new();

        for node in nodes.iter() {
            let key = format!("{}:{}", node.level, node.idx);

            {
                let node_lock = self
                    .lock_map
                    .entry(key.clone())
                    .or_insert_with(|| RwLock::new(()));
                let _write_guard = node_lock.write(); // Lock for writing
                self.map.insert(key.clone(), node.clone());
            }

            keys_to_notify.insert(key);
            //drop(_write_guard);
        }

        for key in keys_to_notify {
            self.notify(&key);
        }
    }

    pub fn get_or_wait(&self, key: &str) -> LayerNode {
        loop {
            // Register for notification before checking if a node is being inserted

            if let Some(value) = self.map.get(&key.to_string()) {
                return value.value().clone();
            }

            let receiver = self.register_for_notification(key);

            // A secondary check
            if let Some(value) = self.map.get(&key.to_string()) {
                println!("Second check grabbed key: {}", key);
                return value.value().clone();
            }

            //receiver.recv().unwrap(); // Block the thread until notification is received
            match receiver.recv_timeout(Duration::from_millis(500)) {
                Ok(_) => { /* Handle reception */ }
                Err(e) => {
                    // Handle timeout or other errors
                    eprintln!("Error or timeout waiting for message: {:?}", e);
                    continue; // or handle differently
                }
            }
        }
    }

    pub fn get_or_wait_opt(&self, key: &str) -> Option<LayerNode> {
        loop {
            if let Some(value) = self.map.get(&key.to_string()) {
                return Some(value.value().clone());
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

#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::db::conversions::{base64_to_node, node_to_base64};
    // use crate::proto::index_buffer::LayerNode;
    // use crate::proto::index_buffer::Point;
    // use std::sync::Arc;
    // use std::thread;
    // use std::time::Duration;

    #[test]
    #[ignore = "todo"]
    fn test_synchronized_nodes() {}
}
