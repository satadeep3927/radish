use im::HashMap;
use bytes::Bytes;
use crate::value::Value;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

fn default_cached_size() -> Arc<AtomicUsize> {
    Arc::new(AtomicUsize::new(usize::MAX))
}

/// The Keyspace manages the top-level mapping of keys to values.
/// It uses an immutable Hash Array Mapped Trie (HAMT) to allow for
/// non-blocking O(1) snapshots.
#[derive(Serialize, Deserialize)]
pub struct Keyspace {
    data: HashMap<Bytes, Value>,
    expires: HashMap<Bytes, u64>,
    #[serde(skip, default = "default_cached_size")]
    cached_size: Arc<AtomicUsize>,
}

impl Clone for Keyspace {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            expires: self.expires.clone(),
            cached_size: Arc::new(AtomicUsize::new(self.cached_size.load(Ordering::Acquire))),
        }
    }
}

impl Default for Keyspace {
    fn default() -> Self {
        Self::new()
    }
}

impl Keyspace {
    /// Create a new, empty Keyspace.
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            expires: HashMap::new(),
            cached_size: default_cached_size(),
        }
    }

    /// Retrieve a reference to a value by its key.
    /// Returns None if the key does not exist or has expired.
    pub fn get(&self, key: &[u8]) -> Option<&Value> {
        if let Some(&deadline) = self.expires.get(key) {
            if now_ms() >= deadline {
                return None;
            }
        }
        self.data.get(key)
    }

    /// Retrieve a mutable reference to a value by its key.
    /// Performs lazy expiration: deletes the key if it has expired.
    pub fn get_mut(&mut self, key: &[u8]) -> Option<&mut Value> {
        if let Some(&deadline) = self.expires.get(key) {
            if now_ms() >= deadline {
                self.del(key);
                return None;
            }
        }
        self.data.get_mut(key)
    }

    /// Insert or update a value for a given key. Clears any existing TTL.
    pub fn set(&mut self, key: Bytes, value: Value) {
        self.expires.remove(&key);
        self.data.insert(key, value);
        self.cached_size.store(usize::MAX, Ordering::Release);
    }

    /// Set a TTL deadline in Unix milliseconds for a given key.
    /// Returns true if the key exists and TTL was set, false otherwise.
    pub fn set_ttl(&mut self, key: &[u8], deadline_ms: u64) -> bool {
        if self.data.contains_key(key) {
            self.expires.insert(Bytes::copy_from_slice(key), deadline_ms);
            self.cached_size.store(usize::MAX, Ordering::Release);
            true
        } else {
            false
        }
    }

    /// Get the remaining TTL in milliseconds for a given key.
    /// Returns None if the key has no TTL or does not exist.
    pub fn get_ttl(&self, key: &[u8]) -> Option<u64> {
        if let Some(&deadline) = self.expires.get(key) {
            let now = now_ms();
            if now >= deadline {
                None // It's expired
            } else {
                Some(deadline - now)
            }
        } else {
            None
        }
    }

    /// Rename a key. Returns true if the old key existed and was renamed.
    pub fn rename(&mut self, old_key: &[u8], new_key: &[u8]) -> bool {
        // If the key has expired, treat it as non-existent
        if let Some(&deadline) = self.expires.get(old_key) {
            if now_ms() >= deadline {
                self.del(old_key);
                return false;
            }
        }
        
        if let Some(value) = self.data.remove(old_key) {
            let ttl = self.expires.remove(old_key);
            let new_bytes = Bytes::copy_from_slice(new_key);
            self.data.insert(new_bytes.clone(), value);
            if let Some(deadline) = ttl {
                self.expires.insert(new_bytes, deadline);
            }
            self.cached_size.store(usize::MAX, Ordering::Release);
            true
        } else {
            false
        }
    }

    /// Delete a key from the keyspace. Returns true if the key existed.
    pub fn del(&mut self, key: &[u8]) -> bool {
        self.expires.remove(key);
        let existed = self.data.remove(key).is_some();
        if existed {
            self.cached_size.store(usize::MAX, Ordering::Release);
        }
        existed
    }

    /// Delete multiple keys. Returns the number of keys successfully removed.
    pub fn del_multi(&mut self, keys: &[Bytes]) -> i64 {
        let mut count = 0;
        for key in keys {
            if self.del(key) {
                count += 1;
            }
        }
        count
    }

    /// Returns the approximate total memory usage of all non-expired keys and their values.
    /// Includes key name sizes and value data payloads.
    pub fn size_of(&self) -> usize {
        let cached = self.cached_size.load(Ordering::Acquire);
        if cached != usize::MAX {
            return cached;
        }

        let now = now_ms();
        let mut total = 0;
        for (key, value) in &self.data {
            if let Some(&deadline) = self.expires.get(key) {
                if now >= deadline {
                    continue;
                }
            }
            total += key.len() + value.deep_size_of();
        }
        self.cached_size.store(total, Ordering::Release);
        total
    }

    /// Returns an iterator over all keys that have not expired.
    pub fn keys(&self) -> impl Iterator<Item = &Bytes> {
        let now = now_ms();
        self.data.keys().filter(move |k| {
            if let Some(&deadline) = self.expires.get(*k) {
                deadline > now
            } else {
                true
            }
        })
    }

    /// Flushes all keys and expirations from the keyspace.
    pub fn flush(&mut self) {
        self.data.clear();
        self.expires.clear();
        self.cached_size.store(usize::MAX, Ordering::Release);
    }

    /// Returns a frozen, O(1) snapshot of the current keyspace.
    pub fn snapshot(&self) -> Self {
        self.clone()
    }

    /// Returns an iterator over all keys that have a TTL set, for the active sweeper.
    pub fn expires_iter(&self) -> impl Iterator<Item = (&Bytes, &u64)> {
        self.expires.iter()
    }

    /// Serializes the Keyspace to a file.
    pub fn save_to_disk(&self, path: &str) -> std::io::Result<()> {
        let file = std::fs::File::create(path)?;
        bincode::serialize_into(file, self).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        Ok(())
    }

    /// Deserializes the Keyspace from a file.
    pub fn load_from_disk(path: &str) -> std::io::Result<Self> {
        let file = std::fs::File::open(path)?;
        let ks = bincode::deserialize_from(file).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        Ok(ks)
    }
}
