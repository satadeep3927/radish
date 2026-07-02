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
            if frames.len() < 3 {
                Frame::Error("ERR wrong number of arguments for 'set' command".to_string())
            } else if let (Some(k), Some(v)) = (extract_bytes(&frames[1]), extract_bytes(&frames[2])) {
                let mut expire = None;
                let mut nx = false;
                let mut xx = false;
                let mut keepttl = false;
                let mut get = false;
                let mut syntax_error = false;
                let mut type_error = false;

                let mut i = 3;
                while i < frames.len() {
                    if let Some(arg) = extract_bytes(&frames[i]) {
                        let arg_str = String::from_utf8_lossy(&arg).to_uppercase();
                        match arg_str.as_str() {
                            "NX" => { nx = true; }
                            "XX" => { xx = true; }
                            "KEEPTTL" => { keepttl = true; }
                            "GET" => { get = true; }
                            "EX" => {
                                if i + 1 >= frames.len() { syntax_error = true; break; }
                                if let Some(val_b) = extract_bytes(&frames[i+1]) {
                                    if let Ok(sec) = String::from_utf8_lossy(&val_b).parse::<u64>() {
                                        expire = Some(radish_storage::keyspace::now_ms() + sec * 1000);
                                    } else { type_error = true; break; }
                                } else { syntax_error = true; break; }
                                i += 1;
                            }
                            "PX" => {
                                if i + 1 >= frames.len() { syntax_error = true; break; }
                                if let Some(val_b) = extract_bytes(&frames[i+1]) {
                                    if let Ok(ms) = String::from_utf8_lossy(&val_b).parse::<u64>() {
                                        expire = Some(radish_storage::keyspace::now_ms() + ms);
                                    } else { type_error = true; break; }
                                } else { syntax_error = true; break; }
                                i += 1;
                            }
                            "EXAT" => {
                                if i + 1 >= frames.len() { syntax_error = true; break; }
                                if let Some(val_b) = extract_bytes(&frames[i+1]) {
                                    if let Ok(sec) = String::from_utf8_lossy(&val_b).parse::<u64>() {
                                        expire = Some(sec * 1000);
                                    } else { type_error = true; break; }
                                } else { syntax_error = true; break; }
                                i += 1;
                            }
                            "PXAT" => {
                                if i + 1 >= frames.len() { syntax_error = true; break; }
                                if let Some(val_b) = extract_bytes(&frames[i+1]) {
                                    if let Ok(ms) = String::from_utf8_lossy(&val_b).parse::<u64>() {
                                        expire = Some(ms);
                                    } else { type_error = true; break; }
                                } else { syntax_error = true; break; }
                                i += 1;
                            }
                            _ => { syntax_error = true; break; }
                        }
                    } else {
                        syntax_error = true; break;
                    }
                    i += 1;
                }

                if syntax_error {
                    Frame::Error("ERR syntax error".to_string())
                } else if type_error {
                    Frame::Error("ERR value is not an integer or out of range".to_string())
                } else if nx && xx {
                    Frame::Error("ERR syntax error".to_string())
                } else {
                    let exists = db.get(&k).is_some();
                    if (nx && exists) || (xx && !exists) {
                        Frame::Null
                    } else {
                        let old_val = if get {
                            match db.get(&k) {
                                Some(Value::String(val)) => Frame::Bulk(val.clone()),
                                Some(_) => return Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
                                None => Frame::Null,
                            }
                        } else {
                            Frame::Null
                        };

                        let retained_ttl = if keepttl { db.get_deadline(&k) } else { None };

                        db.set(k.clone(), Value::String(v));

                        if let Some(deadline) = expire {
                            db.set_ttl(&k, deadline);
                        } else if let Some(deadline) = retained_ttl {
                            db.set_ttl(&k, deadline);
                        }

                        if get {
                            old_val
                        } else {
                            Frame::Simple("OK".to_string())
                        }
                    }
                }
            } else {
                Frame::Error("ERR invalid arguments".to_string())
            }
        }
        "SETNX" => {
            if frames.len() != 3 {
                Frame::Error("ERR wrong number of arguments for 'setnx' command".to_string())
            } else if let (Some(k), Some(v)) = (extract_bytes(&frames[1]), extract_bytes(&frames[2])) {
                if db.get(&k).is_none() {
                    db.set(k, Value::String(v));
                    Frame::Integer(1)
                } else {
                    Frame::Integer(0)
                }
            } else {
                Frame::Error("ERR invalid arguments".to_string())
            }
        }
        "GETSET" => {
            if frames.len() != 3 {
                Frame::Error("ERR wrong number of arguments for 'getset' command".to_string())
            } else if let (Some(k), Some(v)) = (extract_bytes(&frames[1]), extract_bytes(&frames[2])) {
                let old_val = match db.get(&k) {
                    Some(Value::String(val)) => Frame::Bulk(val.clone()),
                    Some(_) => return Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
                    None => Frame::Null,
                };
                db.set(k, Value::String(v));
                old_val
            } else {
                Frame::Error("ERR invalid arguments".to_string())
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
