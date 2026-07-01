use bytes::Bytes;
use radish_proto::Frame;
use radish_server::config::RadishConfig;
use radish_server::commands::match_glob;
use radish_storage::{Keyspace, Value};
use tokio::sync::mpsc;
use std::time::Duration;

#[test]
fn test_config_defaults() {
    let config = RadishConfig::default();
    assert_eq!(config.port, 6379);
    assert_eq!(config.bind, "127.0.0.1");
    assert_eq!(config.requires_auth, false);
    assert_eq!(config.dump_path, "dump.radish");
    assert_eq!(config.maxmemory, "0");
}

#[test]
fn test_config_toml_parsing() {
    let toml_str = r#"
        port = 6385
        bind = "0.0.0.0"
        requires_auth = true
        password = "testpassword"
        dump_path = "custom.dump"
        maxmemory = "512mb"
    "#;
    let config: RadishConfig = toml::from_str(toml_str).unwrap();
    assert_eq!(config.port, 6385);
    assert_eq!(config.bind, "0.0.0.0");
    assert_eq!(config.requires_auth, true);
    assert_eq!(config.password, "testpassword");
    assert_eq!(config.dump_path, "custom.dump");
    assert_eq!(config.maxmemory, "512mb");
}

#[test]
fn test_match_glob_iterative() {
    assert!(match_glob(b"a*b", b"ab"));
    assert!(match_glob(b"a*b", b"axb"));
    assert!(match_glob(b"a*b", b"axxxxxxxxxb"));
    assert!(match_glob(b"a?c", b"abc"));
    assert!(!match_glob(b"a?c", b"abbc"));
    assert!(match_glob(b"*a*b*c*", b"1a2b3c4"));
    assert!(!match_glob(b"*a*b*c*", b"1a2b3"));
    assert!(match_glob(b"", b""));
    assert!(!match_glob(b"abc", b""));
}

#[test]
fn test_keyspace_size_of_caching() {
    let mut ks = Keyspace::new();
    assert_eq!(ks.size_of(), 0);

    ks.set(Bytes::from("key1"), Value::String(Bytes::from("hello")));
    assert_eq!(ks.size_of(), 9);

    ks.set(Bytes::from("key2"), Value::String(Bytes::from("world")));
    assert_eq!(ks.size_of(), 18);

    ks.del(b"key1");
    assert_eq!(ks.size_of(), 9);

    ks.flush();
    assert_eq!(ks.size_of(), 0);
}

fn get_test_config(port: u16, name: &str) -> RadishConfig {
    let mut config = RadishConfig::default();
    config.port = port;
    let mut temp = std::env::temp_dir();
    temp.push(format!("radish_test_{}_{}.dump", name, port));
    config.dump_path = temp.to_string_lossy().to_string();
    config
}

#[tokio::test]
async fn test_scan_with_match() {
    let (tx, rx) = mpsc::channel(10);
    let config = get_test_config(6481, "scan");
    tokio::spawn(async move {
        radish_server::engine::run(rx, config).await;
    });

    let (resp_tx, mut resp_rx) = mpsc::channel(10);
    // Insert some keys
    for k in &["user:1", "user:2", "post:1"] {
        tx.send(radish_server::engine::EngineMessage::Command {
            conn_id: 1,
            frame: Frame::Array(vec![Frame::Bulk(Bytes::from("SET")), Frame::Bulk(Bytes::from(*k)), Frame::Bulk(Bytes::from("val"))]),
            responder: resp_tx.clone(),
        }).await.unwrap();
        let _ = resp_rx.recv().await.unwrap();
    }

    // SCAN with MATCH user:*
    tx.send(radish_server::engine::EngineMessage::Command {
        conn_id: 1,
        frame: Frame::Array(vec![Frame::Bulk(Bytes::from("SCAN")), Frame::Bulk(Bytes::from("0")), Frame::Bulk(Bytes::from("MATCH")), Frame::Bulk(Bytes::from("user:*"))]),
        responder: resp_tx.clone(),
    }).await.unwrap();
    let resp = resp_rx.recv().await.unwrap();
    if let Frame::Array(arr) = resp {
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0], Frame::Bulk(Bytes::from("0")));
        if let Frame::Array(ref keys) = arr[1] {
            assert_eq!(keys.len(), 2);
            let s = format!("{:?}", keys);
            assert!(s.contains("user:1"));
            assert!(s.contains("user:2"));
        } else {
            panic!("Expected array of keys");
        }
    } else {
        panic!("Expected Array response from SCAN");
    }
}

#[tokio::test]
async fn test_setex_ttl() {
    let (tx, rx) = mpsc::channel(10);
    let config = get_test_config(6482, "setex");
    tokio::spawn(async move {
        radish_server::engine::run(rx, config).await;
    });
    let (resp_tx, mut resp_rx) = mpsc::channel(10);

    tx.send(radish_server::engine::EngineMessage::Command {
        conn_id: 1,
        frame: Frame::Array(vec![Frame::Bulk(Bytes::from("SETEX")), Frame::Bulk(Bytes::from("tempkey")), Frame::Bulk(Bytes::from("1")), Frame::Bulk(Bytes::from("tempval"))]),
        responder: resp_tx.clone(),
    }).await.unwrap();
    let resp = resp_rx.recv().await.unwrap();
    assert_eq!(resp, Frame::Simple("OK".to_string()));

    tx.send(radish_server::engine::EngineMessage::Command {
        conn_id: 1,
        frame: Frame::Array(vec![Frame::Bulk(Bytes::from("TTL")), Frame::Bulk(Bytes::from("tempkey"))]),
        responder: resp_tx.clone(),
    }).await.unwrap();
    let resp = resp_rx.recv().await.unwrap();
    if let Frame::Integer(ttl) = resp {
        assert!(ttl == 0 || ttl == 1);
    } else {
        panic!("Expected integer TTL");
    }

    tokio::time::sleep(Duration::from_millis(1100)).await;

    tx.send(radish_server::engine::EngineMessage::Command {
        conn_id: 1,
        frame: Frame::Array(vec![Frame::Bulk(Bytes::from("GET")), Frame::Bulk(Bytes::from("tempkey"))]),
        responder: resp_tx.clone(),
    }).await.unwrap();
    let resp = resp_rx.recv().await.unwrap();
    assert_eq!(resp, Frame::Null);
}

#[tokio::test]
async fn test_memory_usage_command() {
    let (tx, rx) = mpsc::channel(10);
    let config = get_test_config(6483, "memory");
    tokio::spawn(async move {
        radish_server::engine::run(rx, config).await;
    });
    let (resp_tx, mut resp_rx) = mpsc::channel(10);

    tx.send(radish_server::engine::EngineMessage::Command {
        conn_id: 1,
        frame: Frame::Array(vec![Frame::Bulk(Bytes::from("SET")), Frame::Bulk(Bytes::from("memkey")), Frame::Bulk(Bytes::from("somevalue"))]),
        responder: resp_tx.clone(),
    }).await.unwrap();
    let _ = resp_rx.recv().await.unwrap();

    tx.send(radish_server::engine::EngineMessage::Command {
        conn_id: 1,
        frame: Frame::Array(vec![Frame::Bulk(Bytes::from("MEMORY")), Frame::Bulk(Bytes::from("USAGE")), Frame::Bulk(Bytes::from("memkey"))]),
        responder: resp_tx.clone(),
    }).await.unwrap();
    let resp = resp_rx.recv().await.unwrap();
    if let Frame::Integer(usage) = resp {
        assert_eq!(usage, 9); // "somevalue" is 9 bytes
    } else {
        panic!("Expected Integer memory usage");
    }

    tx.send(radish_server::engine::EngineMessage::Command {
        conn_id: 1,
        frame: Frame::Array(vec![Frame::Bulk(Bytes::from("MEMORY")), Frame::Bulk(Bytes::from("USAGE")), Frame::Bulk(Bytes::from("non_existent"))]),
        responder: resp_tx.clone(),
    }).await.unwrap();
    let resp = resp_rx.recv().await.unwrap();
    assert_eq!(resp, Frame::Null);
}
