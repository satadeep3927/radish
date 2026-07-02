pub mod connection;
pub mod generic;
pub mod hash;
pub mod list;
pub mod pubsub;
pub mod set;
pub mod string;
pub mod acl;

use bytes::Bytes;
use radish_proto::Frame;

/// Helper to safely extract Bytes from Simple or Bulk strings
pub fn extract_bytes(frame: &Frame) -> Option<Bytes> {
    match frame {
        Frame::Bulk(b) => Some(b.clone()),
        Frame::Simple(s) => Some(Bytes::from(s.clone())),
        _ => None,
    }
}

/// Iteratively matches a glob pattern against a string to prevent stack overflow
pub fn match_glob(pattern: &[u8], string: &[u8]) -> bool {
    let mut p_idx = 0;
    let mut s_idx = 0;
    let mut star_idx = None;
    let mut s_recall = 0;

    while s_idx < string.len() {
        if p_idx < pattern.len() && (pattern[p_idx] == b'?' || pattern[p_idx] == string[s_idx]) {
            p_idx += 1;
            s_idx += 1;
        } else if p_idx < pattern.len() && pattern[p_idx] == b'*' {
            star_idx = Some(p_idx);
            s_recall = s_idx;
            p_idx += 1;
        } else if let Some(last_star) = star_idx {
            p_idx = last_star + 1;
            s_recall += 1;
            s_idx = s_recall;
        } else {
            return false;
        }
    }

    while p_idx < pattern.len() && pattern[p_idx] == b'*' {
        p_idx += 1;
    }

    p_idx == pattern.len()
}

/// Routes the command to the appropriate handler, acquiring necessary locks from SharedState
pub fn dispatch(
    cmd_name: &str,
    frames: &[Frame],
    conn_id: u64,
    state: &crate::shared::SharedState,
    resp_tx: &tokio::sync::mpsc::Sender<Frame>,
) -> Frame {
    match cmd_name {
        "AUTH" | "ACL" => {
            let mut auth_guard = state.auth.write().unwrap();
            let auth = &mut *auth_guard;
            acl::handle(cmd_name, frames, conn_id, &mut auth.connected_clients, &mut auth.users)
        }
        "PING" | "HELLO" => connection::handle(cmd_name, frames),
        "SET" | "GET" | "MSET" | "MGET" | "SETEX" | "INCR" | "DECR" | "GETRANGE" | "SETNX" | "GETSET" => {
            let mut db = state.db.write().unwrap();
            string::handle(cmd_name, frames, &mut *db)
        }
        "HSET" | "HGET" | "HGETALL" | "HDEL" | "HEXISTS" | "HLEN" | "HSCAN" => {
            let mut db = state.db.write().unwrap();
            hash::handle(cmd_name, frames, &mut *db)
        }
        "LPUSH" | "RPUSH" | "LRANGE" | "LPOP" | "RPOP" | "LLEN" | "LSET" => {
            let mut db = state.db.write().unwrap();
            list::handle(cmd_name, frames, &mut *db)
        }
        "SADD" | "SMEMBERS" | "SREM" | "SISMEMBER" | "SCARD" | "SSCAN" => {
            let mut db = state.db.write().unwrap();
            set::handle(cmd_name, frames, &mut *db)
        }
        "KEYS" | "SCAN" | "FLUSHDB" | "FLUSHALL" | "DEL" | "EXISTS" | "TTL" | "EXPIRE" | "RENAME" | "TYPE" | "MEMORY" | "PEXPIRE" | "EXPIREAT" | "PEXPIREAT" | "PERSIST" | "PTTL" | "OBJECT" => {
            let mut db = state.db.write().unwrap();
            generic::handle(cmd_name, frames, &mut *db, &state.config)
        }
        "SHUTDOWN" => {
            state.shutdown.notify_one();
            Frame::Simple("OK".to_string())
        }
        "SUBSCRIBE" | "PUBLISH" | "UNSUBSCRIBE" | "PSUBSCRIBE" | "PUNSUBSCRIBE" => {
            let mut pubsub = state.pubsub.write().unwrap();
            pubsub::handle(cmd_name, frames, &mut *pubsub, resp_tx)
        }
        "INFO" => {
            let uptime = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() - state.start_time;
            let mut info = String::new();
            info.push_str("# Server\r\n");
            info.push_str(&format!("radish_version:{}\r\n", env!("CARGO_PKG_VERSION")));
            info.push_str(&format!("process_id:{}\r\n", std::process::id()));
            info.push_str(&format!("uptime_in_seconds:{}\r\n", uptime));
            info.push_str(&format!("os:{}\r\n", std::env::consts::OS));
            info.push_str(&format!("arch:{}\r\n", std::env::consts::ARCH));
            info.push_str("# Clients\r\n");
            let active = state.active_connections.load(std::sync::atomic::Ordering::Relaxed);
            info.push_str(&format!("connected_clients:{}\r\n", active));
            info.push_str("# Memory\r\n");
            let total_memory = { state.db.read().unwrap().size_of() };
            info.push_str(&format!("used_memory:{}\r\n", total_memory));
            info.push_str("# Keyspace\r\n");
            let keys = { state.db.read().unwrap().keys().count() };
            info.push_str(&format!("db0_keys:{}\r\n", keys));
            
            Frame::Bulk(bytes::Bytes::from(info))
        }
        _ => Frame::Error(format!("ERR unknown command '{}'", cmd_name)),
    }
}
