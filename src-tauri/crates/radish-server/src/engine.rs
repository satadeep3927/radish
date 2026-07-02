use std::collections::HashMap as StdHashMap;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};

use radish_proto::Frame;
use radish_storage::Keyspace;
use radish_storage::eviction::active_sweep;

use crate::commands;

pub enum EngineMessage {
    Connect {
        conn_id: u64,
    },
    Command {
        conn_id: u64,
        frame: Frame,
        responder: mpsc::Sender<Frame>,
    },
    Disconnect {
        conn_id: u64,
    }
}

pub async fn run(mut receiver: mpsc::Receiver<EngineMessage>, config: crate::config::RadishConfig, shutdown_notify: mpsc::Sender<()>) {
    let start_time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
    let dump_path = config.get_resolved_dump_path().to_string_lossy().to_string();
    let mut db = Keyspace::load_from_disk(&dump_path).unwrap_or_else(|_| Keyspace::new());
    let mut pubsub_channels: StdHashMap<String, Vec<mpsc::Sender<Frame>>> = StdHashMap::new();
    
    // Active connections: Set of connection IDs currently online
    let mut active_connections: std::collections::HashSet<u64> = std::collections::HashSet::new();
    // Auth tracking: Connection ID -> Authenticated Username
    let mut connected_clients: StdHashMap<u64, String> = StdHashMap::new();
    // Default user list for now
    let mut users: StdHashMap<String, String> = StdHashMap::new();
    
    if !config.password.is_empty() {
        users.insert("default".to_string(), config.password.clone());
    } else {
        users.insert("default".to_string(), "radish".to_string());
    }
    
    // Setup the ticker if persistence is enabled
    let mut ticker = config.save_interval.map(|secs| interval(Duration::from_secs(secs)));

    loop {
        let msg_opt = if let Some(ref mut t) = ticker {
            tokio::select! {
                msg = receiver.recv() => msg,
                _ = t.tick() => {
                    // Active Sweeper using the new domain abstraction
                    active_sweep(&mut db);

                    let snapshot = db.snapshot();
                    let dump_path = config.get_resolved_dump_path().to_string_lossy().to_string();
                    tokio::task::spawn_blocking(move || {
                        if let Err(e) = snapshot.save_to_disk(&dump_path) {
                            log::error!("Failed to save snapshot: {}", e);
                        } else {
                            log::info!("Saved database snapshot to {}", dump_path);
                        }
                    });
                    continue;
                }
            }
        } else {
            receiver.recv().await
        };

        let msg = match msg_opt {
            Some(msg) => msg,
            None => break, // Channel closed
        };
        
        match msg {
            EngineMessage::Connect { conn_id } => {
                active_connections.insert(conn_id);
            }
            EngineMessage::Disconnect { conn_id } => {
                active_connections.remove(&conn_id);
                connected_clients.remove(&conn_id);
            }
            EngineMessage::Command { conn_id, frame, responder } => {
                let mut should_shutdown = false;
                let response = match frame {
                    Frame::Array(frames) if !frames.is_empty() => {
                        let cmd_name = match &frames[0] {
                            Frame::Bulk(b) => String::from_utf8_lossy(b).to_uppercase(),
                            Frame::Simple(s) => s.to_uppercase(),
                            _ => "".to_string(),
                        };

                        // ACL Middleware Check
                        // In local dev, we default to no-auth required unless explicitly configured.
                        let requires_auth = config.requires_auth; 
                        let is_authenticated = connected_clients.contains_key(&conn_id);

                        if requires_auth && !is_authenticated && !matches!(cmd_name.as_str(), "AUTH" | "PING" | "HELLO") {
                            Frame::Error("NOAUTH Authentication required.".to_string())
                        } else {
                            match cmd_name.as_str() {
                                "AUTH" | "ACL" => {
                                    crate::commands::acl::handle(
                                        cmd_name.as_str(), 
                                        &frames, 
                                        conn_id, 
                                        &mut connected_clients, 
                                        &mut users
                                    )
                                }
                                "PING" | "HELLO" => commands::connection::handle(cmd_name.as_str(), &frames),
                                "SET" | "GET" | "MSET" | "MGET" | "SETEX" | "INCR" | "DECR" | "GETRANGE" | "SETNX" | "GETSET" => {
                                    commands::string::handle(cmd_name.as_str(), &frames, &mut db)
                                }
                                "HSET" | "HGET" | "HGETALL" | "HDEL" | "HEXISTS" | "HLEN" | "HSCAN" => {
                                    commands::hash::handle(cmd_name.as_str(), &frames, &mut db)
                                }
                                "LPUSH" | "RPUSH" | "LRANGE" | "LPOP" | "RPOP" | "LLEN" | "LSET" => {
                                    commands::list::handle(cmd_name.as_str(), &frames, &mut db)
                                }
                                "SADD" | "SMEMBERS" | "SREM" | "SISMEMBER" | "SCARD" | "SSCAN" => {
                                    commands::set::handle(cmd_name.as_str(), &frames, &mut db)
                                }
                                "KEYS" | "SCAN" | "FLUSHDB" | "FLUSHALL" | "DEL" | "EXISTS" | "TTL" | "EXPIRE" | "RENAME" | "TYPE" | "MEMORY" | "PEXPIRE" | "EXPIREAT" | "PEXPIREAT" | "PERSIST" | "PTTL" | "OBJECT" => {
                                    commands::generic::handle(cmd_name.as_str(), &frames, &mut db, &config)
                                }
                                "SHUTDOWN" => {
                                    should_shutdown = true;
                                    commands::generic::handle(cmd_name.as_str(), &frames, &mut db, &config)
                                }
                                "SUBSCRIBE" | "PUBLISH" | "UNSUBSCRIBE" | "PSUBSCRIBE" | "PUNSUBSCRIBE" => {
                                    commands::pubsub::handle(cmd_name.as_str(), &frames, &mut pubsub_channels, &responder)
                                }
                                "INFO" => {
                                    let uptime = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() - start_time;
                                    let mut info = String::new();
                                    info.push_str("# Server\r\n");
                                    info.push_str(&format!("radish_version:{}\r\n", env!("CARGO_PKG_VERSION")));
                                    info.push_str(&format!("process_id:{}\r\n", std::process::id()));
                                    info.push_str(&format!("uptime_in_seconds:{}\r\n", uptime));
                                    info.push_str(&format!("os:{}\r\n", std::env::consts::OS));
                                    info.push_str(&format!("arch:{}\r\n", std::env::consts::ARCH));
                                    info.push_str("# Clients\r\n");
                                    info.push_str(&format!("connected_clients:{}\r\n", active_connections.len()));
                                    info.push_str("# Memory\r\n");
                                    let total_memory = db.size_of();
                                    info.push_str(&format!("used_memory:{}\r\n", total_memory));
                                    info.push_str("# Keyspace\r\n");
                                    info.push_str(&format!("db0_keys:{}\r\n", db.keys().count()));
                                    
                                    Frame::Bulk(bytes::Bytes::from(info))
                                }
                                _ => Frame::Error(format!("ERR unknown command '{}'", cmd_name)),
                            }
                        }
                    }
                    _ => Frame::Error("ERR syntax error".to_string()),
                };

                let _ = responder.send(response).await;
                if should_shutdown {
                    let _ = shutdown_notify.send(()).await;
                    break;
                }
            }
        }
    }

    log::info!("Engine shutting down, forcing final snapshot save...");
    let snapshot = db.snapshot();
    let dump_path = config.get_resolved_dump_path().to_string_lossy().to_string();
    if let Err(e) = snapshot.save_to_disk(&dump_path) {
        log::error!("Failed to save snapshot on shutdown: {}", e);
    } else {
        log::info!("Saved database snapshot to {} on shutdown successfully.", dump_path);
    }
}
