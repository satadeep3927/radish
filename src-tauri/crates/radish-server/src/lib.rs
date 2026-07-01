pub mod connection;
pub mod engine;
pub mod commands;
pub mod config;

use tokio::net::TcpListener;
use tokio::sync::mpsc;
use connection::Connection;
use engine::run;
use config::RadishConfig;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Starts the Radish Server
pub async fn start(config: RadishConfig, mut cancel_rx: tokio::sync::oneshot::Receiver<()>) {
    log::info!("Radish Server booting up...");
    
    let conn_counter = Arc::new(AtomicU64::new(1));

    // 1. Create the communication channel between connections and the engine
    let (tx, rx) = mpsc::channel(1024);

    // 2. Spawn the single-threaded Execution Engine task
    let engine_config = config.clone();
    tokio::spawn(async move {
        run(rx, engine_config).await;
    });

    let addr = format!("{}:{}", config.bind, config.port);
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            log::error!("Failed to bind to {}: {}. Server might already be running.", addr, e);
            return; // Gracefully exit the task instead of panicking
        }
    };
    log::info!("Listening on {}", addr);

    // 4. Accept incoming client connections
    loop {
        tokio::select! {
            _ = &mut cancel_rx => {
                log::info!("Shutdown signal received, stopping listener loop...");
                break;
            }
            accept_res = listener.accept() => {
                match accept_res {
                    Ok((stream, _addr)) => {
                        let tx = tx.clone();
                        let conn_id = conn_counter.fetch_add(1, Ordering::SeqCst);
                        
                        // Spawn a dedicated task for this TCP connection
                        tokio::spawn(async move {
                            let conn = Connection::new(conn_id, stream);
                            conn.process(tx).await;
                        });
                    }
                    Err(e) => {
                        log::error!("Failed to accept connection: {}", e);
                    }
                }
            }
        }
    }
    
    log::info!("Radish server shutdown sequence completed.");
}

