use radish_proto::Frame;
use radish_storage::{Keyspace, Value};
use bytes::Bytes;
use im::HashSet;
use super::extract_bytes;

pub fn handle(cmd: &str, frames: &[Frame], db: &mut Keyspace) -> Frame {
    match cmd {
        "SADD" => {
            if frames.len() < 3 {
                Frame::Error("ERR wrong number of arguments for 'sadd' command".to_string())
            } else if let Some(k) = extract_bytes(&frames[1]) {
                let elements: Vec<Bytes> = frames[2..].iter().filter_map(extract_bytes).collect();
                if elements.len() != frames.len() - 2 {
                    return Frame::Error("ERR invalid arguments".to_string());
                }
                
                let mut added = 0;
                match db.get_mut(&k) {
                    Some(Value::Set(s)) => {
                        for el in elements {
                            if s.insert(el).is_none() { added += 1; }
                        }
                    },
                    Some(_) => return Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
                    None => {
                        let mut s = HashSet::new();
                        for el in elements {
                            if s.insert(el).is_none() { added += 1; }
                        }
                        db.set(k, Value::Set(s));
                    }
                }
                Frame::Integer(added)
            } else {
                Frame::Error("ERR invalid arguments".to_string())
            }
        }
        "SREM" => {
            if frames.len() < 3 {
                Frame::Error("ERR wrong number of arguments for 'srem' command".to_string())
            } else if let Some(k) = extract_bytes(&frames[1]) {
                let elements: Vec<Bytes> = frames[2..].iter().filter_map(extract_bytes).collect();
                if elements.len() != frames.len() - 2 {
                    return Frame::Error("ERR invalid arguments".to_string());
                }
                
                match db.get_mut(&k) {
                    Some(Value::Set(s)) => {
                        let mut removed = 0;
                        for el in elements {
                            if s.remove(&el).is_some() { removed += 1; }
                        }
                        if s.is_empty() {
                            db.del(&k);
                        }
                        Frame::Integer(removed)
                    },
                    Some(_) => Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
                    None => Frame::Integer(0),
                }
            } else {
                Frame::Error("ERR invalid arguments".to_string())
            }
        }
        "SMEMBERS" => {
            if frames.len() != 2 {
                Frame::Error("ERR wrong number of arguments for 'smembers' command".to_string())
            } else if let Some(k) = extract_bytes(&frames[1]) {
                match db.get(&k) {
                    Some(Value::Set(s)) => {
                        let mut res = Vec::with_capacity(s.len());
                        for v in s.iter() {
                            res.push(Frame::Bulk(v.clone()));
                        }
                        Frame::Array(res)
                    },
                    Some(_) => Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
                    None => Frame::Array(vec![]),
                }
            } else {
                Frame::Error("ERR invalid arguments".to_string())
            }
        }
        "SISMEMBER" => {
            if frames.len() != 3 {
                Frame::Error("ERR wrong number of arguments for 'sismember' command".to_string())
            } else if let (Some(k), Some(v)) = (extract_bytes(&frames[1]), extract_bytes(&frames[2])) {
                match db.get(&k) {
                    Some(Value::Set(s)) => {
                        Frame::Integer(if s.contains(&v) { 1 } else { 0 })
                    },
                    Some(_) => Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
                    None => Frame::Integer(0),
                }
            } else {
                Frame::Error("ERR invalid arguments".to_string())
            }
        }
        "SCARD" => {
            if frames.len() != 2 {
                Frame::Error("ERR wrong number of arguments for 'scard' command".to_string())
            } else if let Some(k) = extract_bytes(&frames[1]) {
                match db.get(&k) {
                    Some(Value::Set(s)) => Frame::Integer(s.len() as i64),
                    Some(_) => Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
                    None => Frame::Integer(0),
                }
            } else {
                Frame::Error("ERR invalid arguments".to_string())
            }
        }
        "SSCAN" => {
            if frames.len() < 3 {
                Frame::Error("ERR wrong number of arguments for 'sscan' command".to_string())
            } else if let Some(k) = extract_bytes(&frames[1]) {
                match db.get(&k) {
                    Some(Value::Set(s)) => {
                        let mut res = Vec::with_capacity(s.len());
                        // Simple implementation: return all elements and cursor 0
                        for v in s.iter() {
                            res.push(Frame::Bulk(v.clone()));
                        }
                        
                        let mut scan_result = Vec::new();
                        scan_result.push(Frame::Bulk(bytes::Bytes::from("0"))); // cursor 0 = done
                        scan_result.push(Frame::Array(res));
                        
                        Frame::Array(scan_result)
                    },
                    Some(_) => Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
                    None => {
                        let mut scan_result = Vec::new();
                        scan_result.push(Frame::Bulk(bytes::Bytes::from("0")));
                        scan_result.push(Frame::Array(vec![]));
                        Frame::Array(scan_result)
                    },
                }
            } else {
                Frame::Error("ERR invalid arguments".to_string())
            }
        }
        _ => Frame::Error(format!("ERR unknown command '{}'", cmd)),
    }
}
