use radish_proto::Frame;
use bytes::Bytes;

pub fn handle(cmd: &str, frames: &[Frame]) -> Frame {
    match cmd {
        "PING" => {
            if frames.len() > 1 {
                match &frames[1] {
                    Frame::Bulk(b) => Frame::Bulk(b.clone()),
                    Frame::Simple(s) => Frame::Simple(s.clone()),
                    _ => Frame::Error("ERR wrong number of arguments for 'ping' command".to_string()),
                }
            } else {
                Frame::Simple("PONG".to_string())
            }
        }
        "HELLO" => {
            let mut res = Vec::new();
            res.push(Frame::Bulk(Bytes::from("server")));
            res.push(Frame::Bulk(Bytes::from("redis"))); // Spoof redis so strict clients don't panic
            res.push(Frame::Bulk(Bytes::from("version")));
            res.push(Frame::Bulk(Bytes::from("6.2.0")));
            res.push(Frame::Bulk(Bytes::from("proto")));
            res.push(Frame::Integer(2));
            res.push(Frame::Bulk(Bytes::from("id")));
            res.push(Frame::Integer(1));
            res.push(Frame::Bulk(Bytes::from("mode")));
            res.push(Frame::Bulk(Bytes::from("standalone")));
            res.push(Frame::Bulk(Bytes::from("role")));
            res.push(Frame::Bulk(Bytes::from("master")));
            res.push(Frame::Bulk(Bytes::from("modules")));
            res.push(Frame::Array(Vec::new()));
            Frame::Array(res)
        }
        _ => Frame::Error(format!("ERR unknown command '{}'", cmd)),
    }
}
