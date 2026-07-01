use radish_proto::Frame;
use radish_storage::{Keyspace, Value};
use im::HashMap;
use super::extract_bytes;

pub fn handle(cmd: &str, frames: &[Frame], db: &mut Keyspace) -> Frame {
    match cmd {
        "HDEL" => {
            if frames.len() < 3 {
                Frame::Error("ERR wrong number of arguments for 'hdel' command".to_string())
            } else if let Some(k) = extract_bytes(&frames[1]) {
                let mut count = 0;
                let mut empty_hash = false;
                
                if let Some(Value::Hash(h)) = db.get_mut(&k) {
                    for f in &frames[2..] {
                        if let Some(field) = extract_bytes(f) {
                            if h.remove(&field).is_some() {
                                count += 1;
                            }
                        }
                    }
                    empty_hash = h.is_empty();
                } else if db.get(&k).is_some() {
                    return Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string());
                }
                
                if empty_hash {
                    db.del(&k);
                }
                Frame::Integer(count)
            } else {
                Frame::Error("ERR invalid arguments".to_string())
            }
        }
        "HEXISTS" => {
            if frames.len() != 3 {
                Frame::Error("ERR wrong number of arguments for 'hexists' command".to_string())
            } else if let (Some(k), Some(f)) = (extract_bytes(&frames[1]), extract_bytes(&frames[2])) {
                match db.get(&k) {
                    Some(Value::Hash(h)) => {
                        Frame::Integer(if h.contains_key(&f) { 1 } else { 0 })
                    }
                    Some(_) => Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
                    None => Frame::Integer(0),
                }
            } else {
                Frame::Error("ERR invalid arguments".to_string())
            }
        }
        "HSET" => {
            if frames.len() < 4 || frames.len() % 2 != 0 {
                Frame::Error("ERR wrong number of arguments for 'hset' command".to_string())
            } else if let Some(k) = extract_bytes(&frames[1]) {
                // Check wrong type before mutating
                if let Some(existing) = db.get(&k) {
                    if !matches!(existing, Value::Hash(_)) {
                        return Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string());
                    }
                }

                let mut new_fields_count = 0;
                match db.get_mut(&k) {
                    Some(Value::Hash(h)) => {
                        let mut i = 2;
                        while i < frames.len() {
                            if let (Some(f), Some(v)) = (extract_bytes(&frames[i]), extract_bytes(&frames[i+1])) {
                                if h.insert(f, v).is_none() {
                                    new_fields_count += 1;
                                }
                            }
                            i += 2;
                        }
                    }
                    None => {
                        let mut h = HashMap::new();
                        let mut i = 2;
                        while i < frames.len() {
                            if let (Some(f), Some(v)) = (extract_bytes(&frames[i]), extract_bytes(&frames[i+1])) {
                                h.insert(f, v);
                                new_fields_count += 1;
                            }
                            i += 2;
                        }
                        db.set(k, Value::Hash(h));
                    }
                    _ => unreachable!(),
                };
                Frame::Integer(new_fields_count)
            } else {
                Frame::Error("ERR invalid arguments".to_string())
            }
        }
        "HGET" => {
            if frames.len() != 3 {
                Frame::Error("ERR wrong number of arguments for 'hget' command".to_string())
            } else if let (Some(k), Some(f)) = (extract_bytes(&frames[1]), extract_bytes(&frames[2])) {
                match db.get(&k) {
                    Some(Value::Hash(h)) => match h.get(&f) {
                        Some(val) => Frame::Bulk(val.clone()),
                        None => Frame::Null,
                    },
                    Some(_) => Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
                    None => Frame::Null,
                }
            } else {
                Frame::Error("ERR invalid arguments".to_string())
            }
        }
        "HGETALL" => {
            if frames.len() != 2 {
                Frame::Error("ERR wrong number of arguments for 'hgetall' command".to_string())
            } else if let Some(k) = extract_bytes(&frames[1]) {
                match db.get(&k) {
                    Some(Value::Hash(h)) => {
                        let mut res = Vec::with_capacity(h.len() * 2);
                        for (f, v) in h.iter() {
                            res.push(Frame::Bulk(f.clone()));
                            res.push(Frame::Bulk(v.clone()));
                        }
                        Frame::Array(res)
                    }
                    Some(_) => Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
                    None => Frame::Array(vec![]),
                }
            } else {
                Frame::Error("ERR invalid arguments".to_string())
            }
        }
        "HLEN" => {
            if frames.len() != 2 {
                Frame::Error("ERR wrong number of arguments for 'hlen' command".to_string())
            } else if let Some(k) = extract_bytes(&frames[1]) {
                match db.get(&k) {
                    Some(Value::Hash(h)) => Frame::Integer(h.len() as i64),
                    Some(_) => Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
                    None => Frame::Integer(0),
                }
            } else {
                Frame::Error("ERR invalid arguments".to_string())
            }
        }
        "HSCAN" => {
            if frames.len() < 3 {
                Frame::Error("ERR wrong number of arguments for 'hscan' command".to_string())
            } else if let Some(k) = extract_bytes(&frames[1]) {
                match db.get(&k) {
                    Some(Value::Hash(h)) => {
                        let mut res = Vec::with_capacity(h.len() * 2);
                        for (f, v) in h.iter() {
                            res.push(Frame::Bulk(f.clone()));
                            res.push(Frame::Bulk(v.clone()));
                        }
                        
                        let mut scan_result = Vec::new();
                        scan_result.push(Frame::Bulk(bytes::Bytes::from("0"))); // cursor 0 = done
                        scan_result.push(Frame::Array(res));
                        
                        Frame::Array(scan_result)
                    }
                    Some(_) => Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
                    None => {
                        let mut scan_result = Vec::new();
                        scan_result.push(Frame::Bulk(bytes::Bytes::from("0")));
                        scan_result.push(Frame::Array(vec![]));
                        Frame::Array(scan_result)
                    }
                }
            } else {
                Frame::Error("ERR invalid arguments".to_string())
            }
        }
        _ => Frame::Error(format!("ERR unknown command '{}'", cmd)),
    }
}
