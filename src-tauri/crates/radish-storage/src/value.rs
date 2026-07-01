use im::{HashMap, HashSet, Vector};
use bytes::Bytes;
use serde::{Deserialize, Serialize};

/// Represents the data types supported by Radish.
/// All complex types use immutable data structures under the hood
/// to allow for zero-copy, O(1) background snapshots.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Value {
    /// A simple string or binary blob.
    String(Bytes),
    
    /// A linked list, implemented as an immutable vector for fast O(1) clone.
    List(Vector<Bytes>),
    
    /// An unordered set of unique strings.
    Set(HashSet<Bytes>),
    
    /// A hash map representing a Redis Hash.
    Hash(HashMap<Bytes, Bytes>),
    
    // ZSet will be added later when we implement Sorted Sets
}

impl Value {
    /// Returns the approximate memory usage of this value in bytes.
    /// This includes only the data payload, not structural overhead of the `im` containers.
    pub fn deep_size_of(&self) -> usize {
        match self {
            Value::String(b) => b.len(),
            Value::List(vec) => vec.iter().map(|b| b.len()).sum(),
            Value::Set(set) => set.iter().map(|b| b.len()).sum(),
            Value::Hash(map) => map.iter().map(|(k, v)| k.len() + v.len()).sum(),
        }
    }
}
