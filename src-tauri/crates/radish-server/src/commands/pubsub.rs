use radish_proto::Frame;
use tokio::sync::mpsc;
use std::collections::HashMap as StdHashMap;
use bytes::Bytes;
use super::{extract_bytes, match_glob};

pub fn handle(
    cmd: &str,
    frames: &[Frame],
    pubsub_channels: &mut StdHashMap<String, Vec<mpsc::Sender<Frame>>>,
    responder: &mpsc::Sender<Frame>,
) -> Frame {
    match cmd {
        "SUBSCRIBE" | "PSUBSCRIBE" => {
            let is_pattern = cmd == "PSUBSCRIBE";
            let reply_verb = if is_pattern { "psubscribe" } else { "subscribe" };
            if frames.len() < 2 {
                Frame::Error(format!("ERR wrong number of arguments for '{}' command", cmd.to_lowercase()))
            } else {
                let mut res = Vec::new();
                for (i, f) in frames[1..].iter().enumerate() {
                    if let Some(channel_b) = extract_bytes(f) {
                        let channel = String::from_utf8_lossy(&channel_b).into_owned();
                        pubsub_channels
                            .entry(channel.clone())
                            .or_insert_with(Vec::new)
                            .push(responder.clone());
                        
                        let mut confirm = Vec::new();
                        confirm.push(Frame::Bulk(Bytes::from(reply_verb)));
                        confirm.push(Frame::Bulk(Bytes::from(channel)));
                        confirm.push(Frame::Integer((i + 1) as i64));
                        res.push(Frame::Array(confirm));
                    }
                }
                for (i, msg) in res.iter().enumerate() {
                    if i < res.len() - 1 {
                        let _ = responder.try_send(msg.clone());
                    }
                }
                res.pop().unwrap_or(Frame::Null)
            }
        }
        "PUBLISH" => {
            if frames.len() != 3 {
                Frame::Error("ERR wrong number of arguments for 'publish' command".to_string())
            } else if let (Some(channel_b), Some(msg_b)) = (extract_bytes(&frames[1]), extract_bytes(&frames[2])) {
                let channel = String::from_utf8_lossy(&channel_b).into_owned();
                let mut receivers = 0;
                
                // Find all matching channels (exact or pattern)
                let mut targets = Vec::new();
                for (k, _) in pubsub_channels.iter() {
                    if k == &channel || match_glob(k.as_bytes(), channel.as_bytes()) {
                        targets.push(k.clone());
                    }
                }

                for target in targets {
                    let is_pattern = target.contains('*') || target.contains('?');
                    let mut msg_arr = Vec::new();
                    if is_pattern {
                        msg_arr.push(Frame::Bulk(Bytes::from("pmessage")));
                        msg_arr.push(Frame::Bulk(Bytes::from(target.clone())));
                        msg_arr.push(Frame::Bulk(Bytes::from(channel.clone())));
                        msg_arr.push(Frame::Bulk(msg_b.clone()));
                    } else {
                        msg_arr.push(Frame::Bulk(Bytes::from("message")));
                        msg_arr.push(Frame::Bulk(Bytes::from(channel.clone())));
                        msg_arr.push(Frame::Bulk(msg_b.clone()));
                    }
                    
                    let broadcast_frame = Frame::Array(msg_arr);

                    if let Some(subs) = pubsub_channels.get_mut(&target) {
                        subs.retain(|sender| {
                            if sender.try_send(broadcast_frame.clone()).is_ok() {
                                receivers += 1;
                                true
                            } else {
                                false
                            }
                        });
                        
                        if subs.is_empty() {
                            pubsub_channels.remove(&target);
                        }
                    }
                }
                Frame::Integer(receivers)
            } else {
                Frame::Error("ERR invalid arguments".to_string())
            }
        }
        "UNSUBSCRIBE" | "PUNSUBSCRIBE" => {
            let is_pattern = cmd == "PUNSUBSCRIBE";
            let reply_verb = if is_pattern { "punsubscribe" } else { "unsubscribe" };
            let channels_to_unsubscribe = if frames.len() >= 2 {
                frames[1..].iter().filter_map(extract_bytes).map(|b| String::from_utf8_lossy(&b).into_owned()).collect::<Vec<_>>()
            } else {
                let mut all_channels = Vec::new();
                for (channel, subs) in pubsub_channels.iter() {
                    if subs.iter().any(|s| s.same_channel(responder)) {
                        all_channels.push(channel.clone());
                    }
                }
                all_channels
            };

            let mut res = Vec::new();
            if channels_to_unsubscribe.is_empty() {
                let mut confirm = Vec::new();
                confirm.push(Frame::Bulk(Bytes::from(reply_verb)));
                confirm.push(Frame::Null);
                confirm.push(Frame::Integer(0));
                res.push(Frame::Array(confirm));
            } else {
                for channel in channels_to_unsubscribe.iter() {
                    if let Some(subs) = pubsub_channels.get_mut(channel) {
                        subs.retain(|s| !s.same_channel(responder));
                    }
                    if pubsub_channels.get(channel).map(|s| s.is_empty()).unwrap_or(false) {
                        pubsub_channels.remove(channel);
                    }
                    
                    let mut confirm = Vec::new();
                    confirm.push(Frame::Bulk(Bytes::from(reply_verb)));
                    confirm.push(Frame::Bulk(Bytes::from(channel.clone())));
                    confirm.push(Frame::Integer(1));
                    res.push(Frame::Array(confirm));
                }
            }
            
            for (i, msg) in res.iter().enumerate() {
                if i < res.len() - 1 {
                    let _ = responder.try_send(msg.clone());
                }
            }
            res.pop().unwrap_or(Frame::Null)
        }
        _ => Frame::Error(format!("ERR unknown command '{}'", cmd)),
    }
}
