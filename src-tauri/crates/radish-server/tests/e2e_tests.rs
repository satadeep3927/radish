use tokio::time::Duration;
use radish_cli::client::Client;
use radish_server::config::RadishConfig;
use radish_proto::Frame;
use bytes::Bytes;
use std::sync::atomic::{AtomicU16, Ordering};

// Use atomic port counter so parallel tests don't collide
static PORT_COUNTER: AtomicU16 = AtomicU16::new(7000);

async fn start_test_server() -> (u16, tokio::sync::oneshot::Sender<()>) {
    let port = PORT_COUNTER.fetch_add(1, Ordering::SeqCst);
    let mut config = RadishConfig::default();
    config.port = port;
    config.requires_auth = false;
    let mut temp = std::env::temp_dir();
    temp.push(format!("radish_test_{}.rdb", port));
    config.dump_path = temp.to_string_lossy().to_string();
    
    let (tx, rx) = tokio::sync::oneshot::channel();
    
    tokio::spawn(async move {
        radish_server::start(config, rx).await;
    });

    // Give the server a moment to bind to the port
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    (port, tx)
}

#[tokio::test]
async fn test_ping() {
    let (port, shutdown_tx) = start_test_server().await;
    let mut client = Client::connect("127.0.0.1", port).await.expect("Failed to connect");
    
    client.send_command("PING").await.unwrap();
    let resp = client.receive_response().await.unwrap();
    assert_eq!(resp, Frame::Simple("PONG".to_string()));
    
    let _ = shutdown_tx.send(());
}

#[tokio::test]
async fn test_set_get() {
    let (port, shutdown_tx) = start_test_server().await;
    let mut client = Client::connect("127.0.0.1", port).await.expect("Failed to connect");
    
    client.send_command("SET test_key test_val").await.unwrap();
    let resp = client.receive_response().await.unwrap();
    assert_eq!(resp, Frame::Simple("OK".to_string()));
    
    client.send_command("GET test_key").await.unwrap();
    let resp = client.receive_response().await.unwrap();
    assert_eq!(resp, Frame::Bulk(Bytes::from("test_val")));
    
    let _ = shutdown_tx.send(());
}

#[tokio::test]
async fn test_mset_mget() {
    let (port, shutdown_tx) = start_test_server().await;
    let mut client = Client::connect("127.0.0.1", port).await.expect("Failed to connect");
    
    client.send_command("MSET k1 v1 k2 v2 k3 v3").await.unwrap();
    let resp = client.receive_response().await.unwrap();
    assert_eq!(resp, Frame::Simple("OK".to_string()));
    
    client.send_command("MGET k1 k3").await.unwrap();
    let resp = client.receive_response().await.unwrap();
    assert_eq!(resp, Frame::Array(vec![
        Frame::Bulk(Bytes::from("v1")),
        Frame::Bulk(Bytes::from("v3"))
    ]));
    
    let _ = shutdown_tx.send(());
}

#[tokio::test]
async fn test_hash_commands() {
    let (port, shutdown_tx) = start_test_server().await;
    let mut client = Client::connect("127.0.0.1", port).await.expect("Failed to connect");
    
    client.send_command("HSET myhash f1 v1 f2 v2").await.unwrap();
    let resp = client.receive_response().await.unwrap();
    assert_eq!(resp, Frame::Integer(2));
    
    client.send_command("HGET myhash f1").await.unwrap();
    let resp = client.receive_response().await.unwrap();
    assert_eq!(resp, Frame::Bulk(Bytes::from("v1")));
    
    let _ = shutdown_tx.send(());
}
