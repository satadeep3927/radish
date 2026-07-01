use radish_proto::Frame;
use radish_storage::{Keyspace, Value};
use bytes::Bytes;
use super::extract_bytes;

pub fn handle(cmd: &str, frames: &[Frame], db: &mut Keyspace) -> Frame {
    match cmd {
        "MSET" => {
            if frames.len() < 3 || frames.len() % 2 == 0 {
                Frame::Error("ERR wrong number of arguments for 'mset' command".to_string())
            } else {
                let mut success = true;
                for i in (1..frames.len()).step_by(2) {
                    if let (Some(k), Some(v)) = (extract_bytes(&frames[i]), extract_bytes(&frames[i+1])) {
                        db.set(k, Value::String(v));
                    } else {
                        success = false;
                        break;
                    }
                }
                if success { Frame::Simple("OK".to_string()) } else { Frame::Error("ERR invalid arguments".to_string()) }
            }
        }
        "MGET" => {
            if frames.len() < 2 {
                Frame::Error("ERR wrong number of arguments for 'mget' command".to_string())
            } else {
                let mut res = Vec::new();
                for f in &frames[1..] {
                    if let Some(k) = extract_bytes(f) {
                        match db.get(&k) {
                            Some(Value::String(val)) => res.push(Frame::Bulk(val.clone())),
                            _ => res.push(Frame::Null),
                        }
                    } else {
                        res.push(Frame::Null);
                    }
                }
                Frame::Array(res)
            }
        }
        "SET" => {
            if frames.len() != 3 {
                Frame::Error("ERR wrong number of arguments for 'set' command".to_string())
            } else {
                if let (Some(k), Some(v)) = (extract_bytes(&frames[1]), extract_bytes(&frames[2])) {
                    db.set(k, Value::String(v));
                    Frame::Simple("OK".to_string())
                } else {
                    Frame::Error("ERR invalid arguments".to_string())
                }
            }
        }
        "SETEX" => {
            if frames.len() != 4 {
                Frame::Error("ERR wrong number of arguments for 'setex' command".to_string())
            } else if let (Some(k), Some(sec_b), Some(v)) = (extract_bytes(&frames[1]), extract_bytes(&frames[2]), extract_bytes(&frames[3])) {
                let sec_str = String::from_utf8_lossy(&sec_b);
                if let Ok(sec) = sec_str.parse::<u64>() {
                    db.set(k.clone(), Value::String(v));
                    let deadline = radish_storage::keyspace::now_ms() + (sec * 1000);
                    db.set_ttl(&k, deadline);
                    Frame::Simple("OK".to_string())
                } else {
                    Frame::Error("ERR value is not an integer or out of range".to_string())
                }
            } else {
                Frame::Error("ERR invalid arguments".to_string())
            }
        }
        "GET" => {
            if frames.len() != 2 {
                Frame::Error("ERR wrong number of arguments for 'get' command".to_string())
            } else {
                if let Some(k) = extract_bytes(&frames[1]) {
                    match db.get(&k) {
                        Some(Value::String(val)) => Frame::Bulk(val.clone()),
                        Some(_) => Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
                        None => Frame::Null,
                    }
                } else {
                    Frame::Error("ERR invalid key".to_string())
                }
            }
        }
        "GETRANGE" => {
            if frames.len() != 4 {
                Frame::Error("ERR wrong number of arguments for 'getrange' command".to_string())
            } else if let (Some(k), Some(start_b), Some(end_b)) = (extract_bytes(&frames[1]), extract_bytes(&frames[2]), extract_bytes(&frames[3])) {
                let start_str = String::from_utf8_lossy(&start_b);
                let end_str = String::from_utf8_lossy(&end_b);
                if let (Ok(mut start), Ok(mut end)) = (start_str.parse::<i64>(), end_str.parse::<i64>()) {
                    match db.get(&k) {
                        Some(Value::String(val)) => {
                            let len = val.len() as i64;
                            if start < 0 { start += len; }
                            if end < 0 { end += len; }
                            if start < 0 { start = 0; }
                            if end < 0 { end = 0; }
                            if end >= len { end = len - 1; }
                            
                            if start > end || start >= len {
                                Frame::Bulk(Bytes::new())
                            } else {
                                let slice = val.slice((start as usize)..(end as usize + 1));
                                Frame::Bulk(slice)
                            }
                        },
                        Some(_) => Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
                        None => Frame::Bulk(Bytes::new()),
                    }
                } else {
                    Frame::Error("ERR value is not an integer or out of range".to_string())
                }
            } else {
                Frame::Error("ERR invalid arguments".to_string())
            }
        }
        "INCR" | "DECR" => {
            if frames.len() != 2 {
                Frame::Error(format!("ERR wrong number of arguments for '{}' command", cmd.to_lowercase()))
            } else if let Some(k) = extract_bytes(&frames[1]) {
                let is_incr = cmd == "INCR";
                let new_val = match db.get_mut(&k) {
                    Some(Value::String(val)) => {
                        let s = std::str::from_utf8(val).unwrap_or("");
                        if let Ok(mut num) = s.parse::<i64>() {
                            if is_incr { num += 1; } else { num -= 1; }
                            *val = Bytes::from(num.to_string());
                            Ok(num)
                        } else {
                            Err("ERR value is not an integer or out of range")
                        }
                    },
                    Some(_) => Err("WRONGTYPE Operation against a key holding the wrong kind of value"),
                    None => {
                        let num = if is_incr { 1 } else { -1 };
                        db.set(k.clone(), Value::String(Bytes::from(num.to_string())));
                        Ok(num)
                    }
                };
                match new_val {
                    Ok(n) => Frame::Integer(n),
                    Err(e) => Frame::Error(e.to_string()),
                }
            } else {
                Frame::Error("ERR invalid arguments".to_string())
            }
        }
        _ => Frame::Error(format!("ERR unknown command '{}'", cmd)),
    }
}
