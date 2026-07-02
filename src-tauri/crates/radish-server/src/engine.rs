use std::sync::Arc;
use tokio::time::{interval, Duration};
use radish_storage::eviction::active_sweep;
use crate::shared::SharedState;

pub async fn background_worker(state: Arc<SharedState>) {
    let mut ticker = state.config.save_interval.map(|secs| interval(Duration::from_secs(secs)));

    loop {
        if let Some(ref mut t) = ticker {
            tokio::select! {
                _ = state.shutdown.notified() => break,
                _ = t.tick() => {
                    // Active Sweeper
                    {
                        let mut db = state.db.write().unwrap();
                        active_sweep(&mut *db);
                    }

                    // Snapshot
                    let snapshot = {
                        let db = state.db.read().unwrap();
                        db.snapshot()
                    };
                    
                    let dump_path = state.config.get_resolved_dump_path().to_string_lossy().to_string();
                    tokio::task::spawn_blocking(move || {
                        if let Err(e) = snapshot.save_to_disk(&dump_path) {
                            log::error!("Failed to save snapshot: {}", e);
                        } else {
                            log::info!("Saved database snapshot to {}", dump_path);
                        }
                    });
                }
            }
        } else {
            // No ticker configured, just wait for shutdown
            state.shutdown.notified().await;
            break;
        }
    }

    log::info!("Engine shutting down, forcing final snapshot save...");
    let snapshot = {
        let db = state.db.read().unwrap();
        db.snapshot()
    };
    let dump_path = state.config.get_resolved_dump_path().to_string_lossy().to_string();
    if let Err(e) = snapshot.save_to_disk(&dump_path) {
        log::error!("Failed to save snapshot on shutdown: {}", e);
    } else {
        log::info!("Saved database snapshot to {} on shutdown successfully.", dump_path);
    }
}
