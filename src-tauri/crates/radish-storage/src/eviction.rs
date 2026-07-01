use crate::keyspace::{now_ms, Keyspace};

/// Performs an active sweep of expired keys.
/// Iterates over keys with a TTL and deletes any that have passed their deadline.
pub fn active_sweep(db: &mut Keyspace) -> usize {
    let now = now_ms();
    let mut to_delete = Vec::new();
    
    for (k, &deadline) in db.expires_iter() {
        if now >= deadline {
            to_delete.push(k.clone());
        }
    }
    
    db.del_multi(&to_delete) as usize
}
