use tokio::sync::mpsc;
use radish_cli::client::Client;
use radish_server::config::RadishConfig;
use radish_proto::Frame;
use bytes::Bytes;
use std::time::Duration;

#[tokio::test]
async fn test_cli_client_basic() {
    let (tx, rx) = mpsc::channel(10);
    // Bind to a custom port to avoid conflict with default port in parallel tests
    let mut config = RadishConfig::default();
    config.port = 6480;
    
    let engine_config = config.clone();
    tokio::spawn(async move {
        radish_server::engine::run(rx, engine_config).await;
    });

    let addr = format!("{}:{}", config.bind, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    
    // Spawn a connection acceptor
    tokio::spawn(async move {
        if let Ok((stream, _)) = listener.accept().await {
            let conn = radish_server::connection::Connection::new(1, stream);
            conn.process(tx).await;
        }
    });

    // Wait briefly for connection handler to start listening
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect using CLI Client
    let mut client = Client::connect("127.0.0.1", 6480).await.unwrap();

    // Send PING
    client.send_command("PING").await.unwrap();
    let resp = client.receive_response().await.unwrap();
    assert_eq!(resp, Frame::Simple("PONG".to_string()));

    // Send SET mykey "hello world"
    client.send_command("SET mykey \"hello world\"").await.unwrap();
    let resp = client.receive_response().await.unwrap();
    assert_eq!(resp, Frame::Simple("OK".to_string()));

    // Send GET mykey
    client.send_command("GET mykey").await.unwrap();
    let resp = client.receive_response().await.unwrap();
    assert_eq!(resp, Frame::Bulk(Bytes::from("hello world")));
}
