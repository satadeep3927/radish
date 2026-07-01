use radish_proto::Frame;
use radish_server::engine::{run, EngineMessage};
use tokio::sync::mpsc;
use bytes::Bytes;
use std::time::Duration;

async fn send_cmd(tx: &mpsc::Sender<EngineMessage>, args: Vec<&str>) -> Frame {
    let (resp_tx, mut resp_rx) = mpsc::channel(10);
    let frame = Frame::Array(args.into_iter().map(|s| Frame::Bulk(Bytes::from(s.to_string()))).collect());
    tx.send(EngineMessage::Command { conn_id: 1, frame, responder: resp_tx }).await.unwrap();
    resp_rx.recv().await.unwrap()
}

async fn auth(tx: &mpsc::Sender<EngineMessage>) {
    send_cmd(tx, vec!["AUTH", "radish"]).await;
}

fn run_test_engine(rx: mpsc::Receiver<EngineMessage>, name: &str) {
    let mut config = radish_server::config::RadishConfig::default();
    let mut temp = std::env::temp_dir();
    temp.push(format!("radish_engine_test_{}.dump", name));
    config.dump_path = temp.to_string_lossy().to_string();
    tokio::spawn(async move { run(rx, config).await; });
}

#[tokio::test]
async fn test_engine_del_exists() {
    let (tx, rx) = mpsc::channel(10);
    run_test_engine(rx, "del_exists");
    auth(&tx).await;

    send_cmd(&tx, vec!["SET", "a", "1"]).await;
    send_cmd(&tx, vec!["SET", "b", "2"]).await;

    let resp = send_cmd(&tx, vec!["EXISTS", "a", "b", "c"]).await;
    assert_eq!(resp, Frame::Integer(2));

    let resp = send_cmd(&tx, vec!["DEL", "a", "c"]).await;
    assert_eq!(resp, Frame::Integer(1));

    let resp = send_cmd(&tx, vec!["EXISTS", "a"]).await;
    assert_eq!(resp, Frame::Integer(0));
}

#[tokio::test]
async fn test_engine_incr_decr() {
    let (tx, rx) = mpsc::channel(10);
    run_test_engine(rx, "incr_decr");
    auth(&tx).await;

    let resp = send_cmd(&tx, vec!["INCR", "counter"]).await;
    assert_eq!(resp, Frame::Integer(1));

    let resp = send_cmd(&tx, vec!["INCR", "counter"]).await;
    assert_eq!(resp, Frame::Integer(2));

    let resp = send_cmd(&tx, vec!["DECR", "counter"]).await;
    assert_eq!(resp, Frame::Integer(1));

    send_cmd(&tx, vec!["SET", "str", "hello"]).await;
    let resp = send_cmd(&tx, vec!["INCR", "str"]).await;
    assert_eq!(resp, Frame::Error("ERR value is not an integer or out of range".to_string()));
}

#[tokio::test]
async fn test_engine_lrange() {
    let (tx, rx) = mpsc::channel(10);
    run_test_engine(rx, "lrange");
    auth(&tx).await;

    send_cmd(&tx, vec!["RPUSH", "mylist", "one", "two", "three"]).await;

    let resp = send_cmd(&tx, vec!["LRANGE", "mylist", "0", "1"]).await;
    assert_eq!(resp, Frame::Array(vec![Frame::Bulk(Bytes::from("one")), Frame::Bulk(Bytes::from("two"))]));

    let resp = send_cmd(&tx, vec!["LRANGE", "mylist", "-2", "-1"]).await;
    assert_eq!(resp, Frame::Array(vec![Frame::Bulk(Bytes::from("two")), Frame::Bulk(Bytes::from("three"))]));
}

#[tokio::test]
async fn test_engine_ttl() {
    let (tx, rx) = mpsc::channel(10);
    run_test_engine(rx, "ttl");
    auth(&tx).await;

    send_cmd(&tx, vec!["SET", "mykey", "hello"]).await;
    let resp = send_cmd(&tx, vec!["EXPIRE", "mykey", "1"]).await;
    assert_eq!(resp, Frame::Integer(1));

    let resp = send_cmd(&tx, vec!["TTL", "mykey"]).await;
    if let Frame::Integer(val) = resp {
        assert!(val == 0 || val == 1);
    } else {
        panic!("TTL did not return integer");
    }

    tokio::time::sleep(Duration::from_millis(1100)).await;

    let resp = send_cmd(&tx, vec!["GET", "mykey"]).await;
    assert_eq!(resp, Frame::Null); // Lazy expiration kicks in

    let resp = send_cmd(&tx, vec!["EXISTS", "mykey"]).await;
    assert_eq!(resp, Frame::Integer(0));
}

#[tokio::test]
async fn test_engine_ping() {
    let (tx, rx) = mpsc::channel(10);
    run_test_engine(rx, "ping");

    let resp = send_cmd(&tx, vec!["PING"]).await;
    assert_eq!(resp, Frame::Simple("PONG".to_string()));
}

#[tokio::test]
async fn test_engine_pubsub() {
    let (engine_tx, engine_rx) = mpsc::channel(10);
    run_test_engine(engine_rx, "pubsub");
    auth(&engine_tx).await;

    // Client 1: Subscriber
    let (sub_resp_tx, mut sub_resp_rx) = mpsc::channel(10);
    let sub_frame = Frame::Array(vec![
        Frame::Bulk(Bytes::from("SUBSCRIBE")),
        Frame::Bulk(Bytes::from("news")),
    ]);
    engine_tx.send(EngineMessage::Command { conn_id: 1, frame: sub_frame, responder: sub_resp_tx }).await.unwrap();

    // Verify subscription confirmation
    let confirm = sub_resp_rx.recv().await.unwrap();
    assert_eq!(confirm, Frame::Array(vec![
        Frame::Bulk(Bytes::from("subscribe")),
        Frame::Bulk(Bytes::from("news")),
        Frame::Integer(1),
    ]));

    // Client 2: Publisher
    let pub_resp = send_cmd(&engine_tx, vec!["PUBLISH", "news", "Radish v1 released!"]).await;
    assert_eq!(pub_resp, Frame::Integer(1)); // 1 receiver

    // Verify subscriber received the broadcast
    let broadcast = sub_resp_rx.recv().await.unwrap();
    assert_eq!(broadcast, Frame::Array(vec![
        Frame::Bulk(Bytes::from("message")),
        Frame::Bulk(Bytes::from("news")),
        Frame::Bulk(Bytes::from("Radish v1 released!")),
    ]));
}

#[tokio::test]
async fn test_engine_mget_mset() {
    let (tx, rx) = mpsc::channel(10);
    run_test_engine(rx, "mget_mset");
    auth(&tx).await;

    let resp = send_cmd(&tx, vec!["MSET", "key1", "val1", "key2", "val2"]).await;
    assert_eq!(resp, Frame::Simple("OK".to_string()));

    let resp = send_cmd(&tx, vec!["MGET", "key1", "keymissing", "key2"]).await;
    assert_eq!(resp, Frame::Array(vec![
        Frame::Bulk(Bytes::from("val1")),
        Frame::Null,
        Frame::Bulk(Bytes::from("val2"))
    ]));
}

#[tokio::test]
async fn test_engine_keys_flushdb() {
    let (tx, rx) = mpsc::channel(10);
    run_test_engine(rx, "keys_flushdb");
    auth(&tx).await;

    send_cmd(&tx, vec!["MSET", "user:1", "a", "user:2", "b", "post:1", "c"]).await;

    let resp = send_cmd(&tx, vec!["KEYS", "user:*"]).await;
    if let Frame::Array(mut arr) = resp {
        // Order is not guaranteed from a HashMap, so we just check length and elements
        assert_eq!(arr.len(), 2);
        let s = format!("{:?}", arr);
        assert!(s.contains("user:1"));
        assert!(s.contains("user:2"));
    } else {
        panic!("KEYS did not return an array");
    }

    let resp = send_cmd(&tx, vec!["FLUSHDB"]).await;
    assert_eq!(resp, Frame::Simple("OK".to_string()));

    let resp = send_cmd(&tx, vec!["KEYS", "*"]).await;
    assert_eq!(resp, Frame::Array(vec![]));
}

#[tokio::test]
async fn test_engine_hash_advanced() {
    let (tx, rx) = mpsc::channel(10);
    run_test_engine(rx, "hash_advanced");
    auth(&tx).await;

    send_cmd(&tx, vec!["HSET", "myhash", "field1", "val1"]).await;
    send_cmd(&tx, vec!["HSET", "myhash", "field2", "val2"]).await;

    let resp = send_cmd(&tx, vec!["HEXISTS", "myhash", "field1"]).await;
    assert_eq!(resp, Frame::Integer(1));

    let resp = send_cmd(&tx, vec!["HEXISTS", "myhash", "field3"]).await;
    assert_eq!(resp, Frame::Integer(0));

    let resp = send_cmd(&tx, vec!["HDEL", "myhash", "field1", "fieldmissing"]).await;
    assert_eq!(resp, Frame::Integer(1));

    let resp = send_cmd(&tx, vec!["HEXISTS", "myhash", "field1"]).await;
    assert_eq!(resp, Frame::Integer(0));
}

#[tokio::test]
async fn test_engine_acl() {
    let (tx, rx) = mpsc::channel(10);
    let mut config = radish_server::config::RadishConfig::default();
    config.requires_auth = true;
    let mut temp = std::env::temp_dir();
    temp.push("radish_engine_test_acl.dump");
    config.dump_path = temp.to_string_lossy().to_string();
    tokio::spawn(async move { run(rx, config).await; });

    // 1. Without AUTH, a command should fail with NOAUTH
    let resp = send_cmd(&tx, vec!["SET", "a", "1"]).await;
    assert_eq!(resp, Frame::Error("NOAUTH Authentication required.".to_string()));

    // 2. Wrong AUTH password
    let resp = send_cmd(&tx, vec!["AUTH", "wrongpass"]).await;
    assert_eq!(resp, Frame::Error("WRONGPASS invalid username-password pair".to_string()));

    // 3. Correct legacy AUTH password (default user)
    let resp = send_cmd(&tx, vec!["AUTH", "radish"]).await;
    assert_eq!(resp, Frame::Simple("OK".to_string()));

    // 4. Now commands should work
    let resp = send_cmd(&tx, vec!["SET", "a", "1"]).await;
    assert_eq!(resp, Frame::Simple("OK".to_string()));
    
    // 5. Test WHOAMI
    let resp = send_cmd(&tx, vec!["ACL", "WHOAMI"]).await;
    assert_eq!(resp, Frame::Bulk(Bytes::from("default")));

    // 6. Test SETUSER
    let resp = send_cmd(&tx, vec!["ACL", "SETUSER", "alice", ">secret"]).await;
    assert_eq!(resp, Frame::Simple("OK".to_string()));
}
