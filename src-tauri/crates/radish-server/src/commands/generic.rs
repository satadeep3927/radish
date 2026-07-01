use radish_proto::Frame;
use radish_storage::Keyspace;
use super::{extract_bytes, match_glob};
use crate::config::RadishConfig;

pub fn handle(cmd: &str, frames: &[Frame], db: &mut Keyspace, config: &RadishConfig) -> Frame {
    match cmd {
        "FLUSHDB" | "FLUSHALL" => {
            db.flush();
            Frame::Simple("OK".to_string())
        }
        "KEYS" => {
            if frames.len() != 2 {
                Frame::Error("ERR wrong number of arguments for 'keys' command".to_string())
            } else if let Some(pattern) = extract_bytes(&frames[1]) {
                let mut res = Vec::new();
                for k in db.keys() {
                    if match_glob(&pattern, k) {
                        res.push(Frame::Bulk(k.clone()));
                    }
                }
                Frame::Array(res)
            } else {
                Frame::Error("ERR invalid arguments".to_string())
            }
        }
        "SCAN" => {
            if frames.len() < 2 {
                Frame::Error("ERR wrong number of arguments for 'scan' command".to_string())
            } else {
                let mut pattern = b"*".to_vec();
                let mut i = 2;
                while i < frames.len() {
                    if let Some(arg) = extract_bytes(&frames[i]) {
                        let arg_str = String::from_utf8_lossy(&arg).to_uppercase();
                        if arg_str == "MATCH" && i + 1 < frames.len() {
                            if let Some(p) = extract_bytes(&frames[i + 1]) {
                                pattern = p.to_vec();
                            }
                            i += 2;
                            continue;
                        } else if arg_str == "COUNT" || arg_str == "TYPE" {
                            i += 2;
                            continue;
                        }
                    }
                    i += 1;
                }

                let mut res = Vec::new();
                for k in db.keys() {
                    if match_glob(&pattern, k) {
                        res.push(Frame::Bulk(k.clone()));
                    }
                }
                
                let mut scan_result = Vec::new();
                scan_result.push(Frame::Bulk(bytes::Bytes::from("0"))); // cursor 0 = done
                scan_result.push(Frame::Array(res));
                
                Frame::Array(scan_result)
            }
        }
        "DEL" => {
            if frames.len() < 2 {
                Frame::Error("ERR wrong number of arguments for 'del' command".to_string())
            } else {
                let mut count = 0;
                for f in &frames[1..] {
                    if let Some(k) = extract_bytes(f) {
                        if db.del(&k) { count += 1; }
                    }
                }
                Frame::Integer(count)
            }
        }
        "EXISTS" => {
            if frames.len() < 2 {
                Frame::Error("ERR wrong number of arguments for 'exists' command".to_string())
            } else {
                let mut count = 0;
                for f in &frames[1..] {
                    if let Some(k) = extract_bytes(f) {
                        if db.get(&k).is_some() { count += 1; }
                    }
                }
                Frame::Integer(count)
            }
        }
        "EXPIRE" => {
            if frames.len() != 3 {
                Frame::Error("ERR wrong number of arguments for 'expire' command".to_string())
            } else if let (Some(k), Some(sec_b)) = (extract_bytes(&frames[1]), extract_bytes(&frames[2])) {
                let sec_str = String::from_utf8_lossy(&sec_b);
                if let Ok(sec) = sec_str.parse::<u64>() {
                    let deadline = radish_storage::keyspace::now_ms() + (sec * 1000);
                    if db.set_ttl(&k, deadline) {
                        Frame::Integer(1)
                    } else {
                        Frame::Integer(0)
                    }
                } else {
                    Frame::Error("ERR value is not an integer or out of range".to_string())
                }
            } else {
                Frame::Error("ERR invalid arguments".to_string())
            }
        }
        "TTL" => {
            if frames.len() != 2 {
                Frame::Error("ERR wrong number of arguments for 'ttl' command".to_string())
            } else if let Some(k) = extract_bytes(&frames[1]) {
                if db.get(&k).is_none() {
                    Frame::Integer(-2) // Key does not exist
                } else if let Some(ms) = db.get_ttl(&k) {
                    Frame::Integer((ms / 1000) as i64)
                } else {
                    Frame::Integer(-1) // Key exists but has no TTL
                }
            } else {
                Frame::Error("ERR invalid arguments".to_string())
            }
        }
        "RENAME" => {
            if frames.len() != 3 {
                Frame::Error("ERR wrong number of arguments for 'rename' command".to_string())
            } else if let (Some(old_k), Some(new_k)) = (extract_bytes(&frames[1]), extract_bytes(&frames[2])) {
                if db.rename(&old_k, &new_k) {
                    Frame::Simple("OK".to_string())
                } else {
                    Frame::Error("ERR no such key".to_string())
                }
            } else {
                Frame::Error("ERR invalid arguments".to_string())
            }
        }
        "TYPE" => {
            if frames.len() != 2 {
                Frame::Error("ERR wrong number of arguments for 'type' command".to_string())
            } else if let Some(k) = extract_bytes(&frames[1]) {
                if let Some(val) = db.get(&k) {
                    let type_str = match val {
                        radish_storage::value::Value::String(_) => "string",
                        radish_storage::value::Value::List(_) => "list",
                        radish_storage::value::Value::Set(_) => "set",
                        radish_storage::value::Value::Hash(_) => "hash",
                    };
                    Frame::Simple(type_str.to_string())
                } else {
                    Frame::Simple("none".to_string())
                }
            } else {
                Frame::Error("ERR invalid arguments".to_string())
            }
        }
        "SHUTDOWN" => {
            let snapshot = db.snapshot();
            let dump_path = config.get_resolved_dump_path().to_string_lossy().to_string();
            let _ = snapshot.save_to_disk(&dump_path);
            Frame::Simple("OK".to_string())
        }
        "MEMORY" => {
            if frames.len() < 2 {
                Frame::Error("ERR wrong number of arguments for 'memory' command".to_string())
            } else if let Some(subcmd) = extract_bytes(&frames[1]) {
                let subcmd_str = String::from_utf8_lossy(&subcmd).to_uppercase();
                match subcmd_str.as_str() {
                    "USAGE" => {
                        if frames.len() != 3 {
                            Frame::Error("ERR wrong number of arguments for 'memory usage' command".to_string())
                        } else if let Some(k) = extract_bytes(&frames[2]) {
                            match db.get(&k) {
                                Some(val) => Frame::Integer(val.deep_size_of() as i64),
                                None => Frame::Null,
                            }
                        } else {
                            Frame::Error("ERR invalid arguments".to_string())
                        }
                    }
                    _ => Frame::Error(format!("ERR unknown subcommand '{}'", subcmd_str)),
                }
            } else {
                Frame::Error("ERR invalid arguments".to_string())
            }
        }
        _ => Frame::Error(format!("ERR unknown command '{}'", cmd)),
    }
}
