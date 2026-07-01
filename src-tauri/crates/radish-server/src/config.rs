use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RadishConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    
    #[serde(default)]
    pub save_interval: Option<u64>,
    
    #[serde(default)]
    pub requires_auth: bool,
    
    #[serde(default = "default_password")]
    pub password: String,
    
    #[serde(default = "default_dump_path")]
    pub dump_path: String,
    
    #[serde(default = "default_bind")]
    pub bind: String,

    #[serde(default = "default_maxmemory")]
    pub maxmemory: String,
}

fn default_port() -> u16 { 6379 }
fn default_password() -> String { "".to_string() }
fn default_dump_path() -> String { "dump.radish".to_string() }
fn default_bind() -> String { "127.0.0.1".to_string() }
fn default_maxmemory() -> String { "0".to_string() }

impl Default for RadishConfig {
    fn default() -> Self {
        RadishConfig {
            port: default_port(),
            save_interval: None,
            requires_auth: false,
            password: default_password(),
            dump_path: default_dump_path(),
            bind: default_bind(),
            maxmemory: default_maxmemory(),
        }
    }
}

use std::path::PathBuf;

pub fn get_config_dir() -> PathBuf {
    let home = std::env::var("USERPROFILE") // Windows
        .or_else(|_| std::env::var("HOME")) // Unix/macOS
        .unwrap_or_else(|_| ".".to_string());
    
    let mut path = PathBuf::from(home);
    path.push(".radish");
    
    // Ensure the folder exists
    let _ = std::fs::create_dir_all(&path);
    path
}

impl RadishConfig {
    /// Load from `radish.toml` in `~/.radish/`, or return defaults if missing.
    pub fn load() -> Self {
        let mut path = get_config_dir();
        path.push("radish.toml");
        if path.exists() {
            let content = std::fs::read_to_string(path).unwrap_or_default();
            toml::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Resolves the dump path to absolute. If relative, resolves inside `~/.radish/`.
    pub fn get_resolved_dump_path(&self) -> PathBuf {
        let path = std::path::Path::new(&self.dump_path);
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            let mut dir = get_config_dir();
            dir.push(&self.dump_path);
            dir
        }
    }
}
