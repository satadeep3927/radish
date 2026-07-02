pub mod connection;
pub mod engine;
pub mod commands;
pub mod config;
pub mod shared;

use tokio::net::TcpListener;
use connection::Connection;
use engine::background_worker;
use config::RadishConfig;
use shared::SharedState;

use std::sync::atomic::Ordering;
use std::sync::Arc;

/// Starts the Radish Server
pub async fn start(config: RadishConfig, mut cancel_rx: tokio::sync::oneshot::Receiver<()>) {
    log::info!("Radish Server booting up...");
    
    let dump_path = config.get_resolved_dump_path().to_string_lossy().to_string();
    let state = Arc::new(SharedState::new(config.clone(), &dump_path));
    
    // Spawn background worker for active sweeping and snapshots
    let worker_state = state.clone();
    tokio::spawn(async move {
        background_worker(worker_state).await;
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

    // Accept incoming client connections
    loop {
        tokio::select! {
            _ = &mut cancel_rx => {
                log::info!("Shutdown signal received via CLI, stopping listener loop...");
                break;
            }
            _ = state.shutdown.notified() => {
                log::info!("Engine executed SHUTDOWN command, stopping listener loop...");
                break;
            }
            accept_res = listener.accept() => {
                match accept_res {
                    Ok((stream, _addr)) => {
                        // Disable Nagle's algorithm to drastically reduce latency for unpipelined requests
                        let _ = stream.set_nodelay(true);
                        
                        let state_clone = state.clone();
                        let conn_id = state.next_id.fetch_add(1, Ordering::SeqCst);
                        
                        // Spawn a dedicated task for this TCP connection
                        tokio::spawn(async move {
                            let conn = Connection::new(conn_id, stream);
                            conn.process(state_clone).await;
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

