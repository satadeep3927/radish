use std::sync::RwLock;
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use tokio::sync::Notify;
use radish_proto::Frame;
use radish_storage::Keyspace;
use tokio::sync::mpsc;
use crate::config::RadishConfig;

pub struct AuthState {
    pub connected_clients: HashMap<u64, String>,
    pub users: HashMap<String, String>,
}

pub struct SharedState {
    pub db: RwLock<Keyspace>,
    pub pubsub: RwLock<HashMap<String, Vec<mpsc::Sender<Frame>>>>,
    pub auth: RwLock<AuthState>,
    pub config: RadishConfig,
    pub start_time: u64,
    pub next_id: AtomicU64,
    pub active_connections: AtomicU64,
    pub shutdown: Notify,
}

impl SharedState {
    pub fn new(config: RadishConfig, dump_path: &str) -> Self {
        let db = Keyspace::load_from_disk(dump_path).unwrap_or_else(|_| Keyspace::new());
        let start_time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
        
        let mut users = HashMap::new();
        if !config.password.is_empty() {
            users.insert("default".to_string(), config.password.clone());
        } else {
            users.insert("default".to_string(), "radish".to_string());
        }
        
        Self {
            db: RwLock::new(db),
            pubsub: RwLock::new(HashMap::new()),
            auth: RwLock::new(AuthState {
                connected_clients: HashMap::new(),
                users,
            }),
            config,
            start_time,
            next_id: AtomicU64::new(1),
            active_connections: AtomicU64::new(0),
            shutdown: Notify::new(),
        }
    }
}
