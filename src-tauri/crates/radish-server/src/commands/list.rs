use radish_proto::Frame;
use radish_storage::{Keyspace, Value};
use bytes::Bytes;
use im::Vector;
use super::extract_bytes;

pub fn handle(cmd: &str, frames: &[Frame], db: &mut Keyspace) -> Frame {
    match cmd {
        "LPUSH" | "RPUSH" => {
            if frames.len() < 3 {
                Frame::Error(format!("ERR wrong number of arguments for '{}' command", cmd.to_lowercase()))
            } else if let Some(k) = extract_bytes(&frames[1]) {
                let is_lpush = cmd == "LPUSH";
                let elements: Vec<Bytes> = frames[2..].iter().filter_map(extract_bytes).collect();
                if elements.len() != frames.len() - 2 {
                    return Frame::Error("ERR invalid arguments".to_string());
                }
                
                let len = match db.get_mut(&k) {
                    Some(Value::List(l)) => {
                        for el in elements {
                            if is_lpush { l.push_front(el); } else { l.push_back(el); }
                        }
                        l.len()
                    },
                    Some(_) => return Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
                    None => {
                        let mut l = Vector::new();
                        for el in elements {
                            if is_lpush { l.push_front(el); } else { l.push_back(el); }
                        }
                        let len = l.len();
                        db.set(k, Value::List(l));
                        len
                    }
                };
                Frame::Integer(len as i64)
            } else {
                Frame::Error("ERR invalid arguments".to_string())
            }
        }
        "LPOP" | "RPOP" => {
            if frames.len() != 2 {
                Frame::Error(format!("ERR wrong number of arguments for '{}' command", cmd.to_lowercase()))
            } else if let Some(k) = extract_bytes(&frames[1]) {
                let is_lpop = cmd == "LPOP";
                match db.get_mut(&k) {
                    Some(Value::List(l)) => {
                        let val = if is_lpop { l.pop_front() } else { l.pop_back() };
                        if l.is_empty() {
                            db.del(&k);
                        }
                        match val {
                            Some(v) => Frame::Bulk(v),
                            None => Frame::Null,
                        }
                    },
                    Some(_) => Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
                    None => Frame::Null,
                }
            } else {
                Frame::Error("ERR invalid arguments".to_string())
            }
        }
        "LLEN" => {
            if frames.len() != 2 {
                Frame::Error("ERR wrong number of arguments for 'llen' command".to_string())
            } else if let Some(k) = extract_bytes(&frames[1]) {
                match db.get(&k) {
                    Some(Value::List(l)) => Frame::Integer(l.len() as i64),
                    Some(_) => Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
                    None => Frame::Integer(0),
                }
            } else {
                Frame::Error("ERR invalid arguments".to_string())
            }
        }
        "LRANGE" => {
            if frames.len() != 4 {
                Frame::Error("ERR wrong number of arguments for 'lrange' command".to_string())
            } else if let (Some(k), Some(start_b), Some(end_b)) = (extract_bytes(&frames[1]), extract_bytes(&frames[2]), extract_bytes(&frames[3])) {
                let start_str = String::from_utf8_lossy(&start_b);
                let end_str = String::from_utf8_lossy(&end_b);
                if let (Ok(start), Ok(end)) = (start_str.parse::<i64>(), end_str.parse::<i64>()) {
                    match db.get(&k) {
                        Some(Value::List(l)) => {
                            let len = l.len() as i64;
                            let s = if start < 0 { std::cmp::max(0, len + start) } else { start };
                            let e = if end < 0 { std::cmp::max(0, len + end) } else { end };
                            
                            let mut res = Vec::new();
                            if s <= e && s < len {
                                let actual_end = std::cmp::min(len - 1, e);
                                for i in s..=actual_end {
                                    res.push(Frame::Bulk(l[i as usize].clone()));
                                }
                            }
                            Frame::Array(res)
                        },
                        Some(_) => Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
                        None => Frame::Array(vec![]),
                    }
                } else {
                    Frame::Error("ERR value is not an integer or out of range".to_string())
                }
            } else {
                Frame::Error("ERR invalid arguments".to_string())
            }
        }
        "LSET" => {
            if frames.len() != 4 {
                Frame::Error("ERR wrong number of arguments for 'lset' command".to_string())
            } else if let (Some(k), Some(idx_b), Some(v)) = (extract_bytes(&frames[1]), extract_bytes(&frames[2]), extract_bytes(&frames[3])) {
                let idx_str = String::from_utf8_lossy(&idx_b);
                if let Ok(idx) = idx_str.parse::<i64>() {
                    match db.get_mut(&k) {
                        Some(Value::List(l)) => {
                            let len = l.len() as i64;
                            let i = if idx < 0 { len + idx } else { idx };
                            if i < 0 || i >= len {
                                Frame::Error("ERR index out of range".to_string())
                            } else {
                                l[i as usize] = v;
                                Frame::Simple("OK".to_string())
                            }
                        },
                        Some(_) => Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
                        None => Frame::Error("ERR no such key".to_string()),
                    }
                } else {
                    Frame::Error("ERR value is not an integer or out of range".to_string())
                }
            } else {
                Frame::Error("ERR invalid arguments".to_string())
            }
        }
        _ => Frame::Error(format!("ERR unknown command '{}'", cmd)),
    }
}
