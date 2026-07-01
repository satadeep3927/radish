use serde_json::{Value, json};
use bytes::{Bytes, BytesMut, Buf};
use radish_proto::Frame;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use tauri::{AppHandle, State, Emitter};
use tokio::sync::oneshot;
use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref SERVER_CANCEL_TX: Mutex<Option<oneshot::Sender<()>>> = Mutex::new(None);
    static ref APP_HANDLE: Mutex<Option<AppHandle>> = Mutex::new(None);
}

struct TauriLogger;

impl log::Log for TauriLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let msg = format!("[{}] {}", record.level(), record.args());
            println!("{}", msg);
            
            if let Some(app) = APP_HANDLE.lock().ok().and_then(|guard| guard.as_ref().cloned()) {
                let _ = app.emit("server-log", msg);
            }
        }
    }

    fn flush(&self) {}
}

static LOGGER: TauriLogger = TauriLogger;

#[tauri::command]
fn read_config() -> Result<radish_server::config::RadishConfig, String> {
    Ok(radish_server::config::RadishConfig::load())
}

#[tauri::command]
fn write_config(config: radish_server::config::RadishConfig) -> Result<(), String> {
    let mut path = radish_server::config::get_config_dir();
    path.push("radish.toml");
    let content = toml::to_string(&config).map_err(|e| e.to_string())?;
    std::fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn start_server() -> Result<(), String> {
    let mut cancel_slot = SERVER_CANCEL_TX.lock().unwrap();
    if cancel_slot.is_some() {
        return Err("Server is already running".to_string());
    }
    
    let config = radish_server::config::RadishConfig::load();
    let (tx, rx) = oneshot::channel();
    *cancel_slot = Some(tx);

    tauri::async_runtime::spawn(async move {
        radish_server::start(config, rx).await;
        // Clear the cancel slot when the server exits (e.g. bind failure or normal shutdown)
        let mut cancel_slot = SERVER_CANCEL_TX.lock().unwrap();
        *cancel_slot = None;
    });
    Ok(())
}

#[tauri::command]
fn stop_server() -> Result<(), String> {
    let mut cancel_slot = SERVER_CANCEL_TX.lock().unwrap();
    if let Some(tx) = cancel_slot.take() {
        let _ = tx.send(());
        Ok(())
    } else {
        Err("Server is not running".to_string())
    }
}

#[tauri::command]
async fn restart_server() -> Result<(), String> {
    let _ = stop_server();
    // Wait briefly for the server port to free up
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    start_server()
}

#[tauri::command]
async fn execute_command(connection_string: String, args: Vec<String>, password: Option<String>) -> Result<Value, String> {
    if args.is_empty() {
        return Err("No command provided".into());
    }

    let connect_future = TcpStream::connect(&connection_string);
    let mut stream = tokio::time::timeout(std::time::Duration::from_secs(2), connect_future)
        .await
        .map_err(|_| "Connection timed out".to_string())?
        .map_err(|e| e.to_string())?;

    // If a password is provided, send AUTH before the actual command
    if let Some(ref pwd) = password {
        if !pwd.is_empty() {
            let auth_frame = Frame::Array(vec![
                Frame::Bulk(Bytes::from("AUTH")),
                Frame::Bulk(Bytes::from(pwd.clone())),
            ]);
            let mut auth_data = BytesMut::new();
            auth_frame.serialize(&mut auth_data);
            stream.write_all(&auth_data).await.map_err(|e| e.to_string())?;

            // Read and discard the AUTH response
            let mut auth_buf = BytesMut::with_capacity(64);
            loop {
                match radish_proto::parse(&mut auth_buf) {
                    Ok(_) => break,
                    Err(radish_proto::ParseError::Incomplete) => {}
                    Err(e) => return Err(format!("Auth parse error: {:?}", e)),
                }
                let mut tmp = [0u8; 64];
                let n = tokio::time::timeout(std::time::Duration::from_secs(2), stream.read(&mut tmp))
                    .await
                    .map_err(|_| "Auth read timed out".to_string())?
                    .map_err(|e| e.to_string())?;
                if n == 0 { return Err("Connection reset during AUTH".into()); }
                auth_buf.extend_from_slice(&tmp[..n]);
            }
        }
    }

    let frames: Vec<Frame> = args.into_iter().map(|arg| Frame::Bulk(Bytes::from(arg))).collect();
    let array_frame = Frame::Array(frames);

    let mut data = BytesMut::new();
    array_frame.serialize(&mut data);
    
    stream.write_all(&data).await.map_err(|e| e.to_string())?;

    let mut buffer = BytesMut::with_capacity(4096);
    
    loop {
        match radish_proto::parse(&mut buffer) {
            Ok((frame, _consumed)) => {
                return Ok(frame_to_json(&frame));
            }
            Err(radish_proto::ParseError::Incomplete) => {
                // Need more data
            }
            Err(e) => {
                return Err(format!("Parse error: {:?}", e));
            }
        }

        if buffer.capacity() == buffer.len() {
            buffer.reserve(4096);
        }

        let mut temp_buf = [0u8; 4096];
        let read_future = stream.read(&mut temp_buf);
        let n = tokio::time::timeout(std::time::Duration::from_secs(2), read_future)
            .await
            .map_err(|_| "Read timed out".to_string())?
            .map_err(|e| e.to_string())?;
            
        if n == 0 {
            return Err("Connection reset by peer".into());
        }
        buffer.extend_from_slice(&temp_buf[..n]);
    }
}

fn frame_to_json(frame: &Frame) -> Value {
    match frame {
        Frame::Simple(s) => json!(s),
        Frame::Error(e) => json!({ "error": e }),
        Frame::Integer(i) => json!(i),
        Frame::Bulk(b) => {
            if let Ok(s) = std::str::from_utf8(b) {
                json!(s)
            } else {
                // If it's not valid utf8, return an array of bytes
                json!(b.as_ref())
            }
        },
        Frame::Null => json!(null),
        Frame::Array(arr) => {
            let json_arr: Vec<Value> = arr.iter().map(frame_to_json).collect();
            json!(json_arr)
        }
    }
}

use serde::Serialize;
use std::collections::BTreeMap;

/// A node in the key tree. Leaf nodes have `full_key` set; folder nodes do not.
#[derive(Serialize, Clone)]
pub struct KeyNode {
    pub name: String,
    /// The full Redis key — only present if this node is a real key (leaf).
    pub full_key: Option<String>,
    /// Child nodes, sorted alphabetically.
    pub children: Vec<KeyNode>,
}

/// Build a tree from a flat list of key strings, splitting on `separator`.
#[tauri::command]
fn group_keys(keys: Vec<String>, separator: String) -> Vec<KeyNode> {
    let sep = separator.chars().next().unwrap_or(':');

    // Use a recursive BTreeMap trie: BTreeMap<segment, (full_key_if_leaf, children)>
    type Trie = BTreeMap<String, TrieNode>;
    struct TrieNode {
        full_key: Option<String>,
        children: Trie,
    }

    fn insert_key(trie: &mut Trie, segments: &[&str], full_key: &str) {
        if segments.is_empty() { return; }
        let head = segments[0].to_string();
        let tail = &segments[1..];
        let node = trie.entry(head).or_insert_with(|| TrieNode { full_key: None, children: BTreeMap::new() });
        if tail.is_empty() {
            // This segment IS the leaf — record the full key
            node.full_key = Some(full_key.to_string());
        } else {
            insert_key(&mut node.children, tail, full_key);
        }
    }

    fn trie_to_nodes(trie: Trie) -> Vec<KeyNode> {
        trie.into_iter().map(|(name, node)| {
            let children = trie_to_nodes(node.children);
            KeyNode { name, full_key: node.full_key, children }
        }).collect()
    }

    let mut trie: Trie = BTreeMap::new();
    for key in &keys {
        let segments: Vec<&str> = key.split(sep).collect();
        insert_key(&mut trie, &segments, key);
    }

    trie_to_nodes(trie)
}

#[derive(Default)]
pub struct PubSubState {
    pub active_subs: Mutex<HashMap<String, oneshot::Sender<()>>>,
}

#[tauri::command]
async fn subscribe_channel(
    app: AppHandle,
    state: State<'_, PubSubState>,
    connection_string: String,
    channel: String,
    password: Option<String>,
) -> Result<(), String> {
    let mut subs = state.active_subs.lock().unwrap();
    if subs.contains_key(&channel) {
        return Ok(()); // Already subscribed
    }

    let (cancel_tx, mut cancel_rx) = oneshot::channel::<()>();
    subs.insert(channel.clone(), cancel_tx);

    let channel_clone = channel.clone();
    tokio::spawn(async move {
        // 1. Connect to TCP socket
        let mut stream = match TcpStream::connect(&connection_string).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("PubSub failed to connect: {}", e);
                return;
            }
        };

        // 2. If password provided, send AUTH first
        if let Some(ref pwd) = password {
            if !pwd.is_empty() {
                let auth_frame = Frame::Array(vec![
                    Frame::Bulk(Bytes::from("AUTH")),
                    Frame::Bulk(Bytes::from(pwd.clone())),
                ]);
                let mut auth_data = BytesMut::new();
                auth_frame.serialize(&mut auth_data);
                if stream.write_all(&auth_data).await.is_err() { return; }

                // Drain the AUTH response
                let mut auth_buf = BytesMut::with_capacity(64);
                let mut tmp = [0u8; 64];
                loop {
                    match radish_proto::parse(&mut auth_buf) {
                        Ok(_) => break,
                        Err(radish_proto::ParseError::Incomplete) => {}
                        Err(_) => return,
                    }
                    match stream.read(&mut tmp).await {
                        Ok(n) if n > 0 => auth_buf.extend_from_slice(&tmp[..n]),
                        _ => return,
                    }
                }
            }
        }

        // 3. Format and serialize SUBSCRIBE or PSUBSCRIBE command
        let is_pattern = channel_clone.contains('*') || channel_clone.contains('?');
        let cmd_verb = if is_pattern { "PSUBSCRIBE" } else { "SUBSCRIBE" };
        let frames = vec![
            Frame::Bulk(Bytes::from(cmd_verb)),
            Frame::Bulk(Bytes::from(channel_clone.clone())),
        ];
        let array_frame = Frame::Array(frames);
        let mut data = BytesMut::new();
        array_frame.serialize(&mut data);

        if stream.write_all(&data).await.is_err() {
            return;
        }

        let mut buffer = BytesMut::with_capacity(4096);
        let mut temp_buf = [0u8; 4096];

        loop {
            tokio::select! {
                _ = &mut cancel_rx => {
                    break; // Unsubscribed
                }
                read_res = stream.read(&mut temp_buf) => {
                    let n = match read_res {
                        Ok(n) if n > 0 => n,
                        _ => break, // Connection closed or error
                    };
                    buffer.extend_from_slice(&temp_buf[..n]);

                    // Parse frames
                    loop {
                        match radish_proto::parse(&mut buffer) {
                            Ok((frame, consumed)) => {
                                buffer.advance(consumed);
                                let json_val = frame_to_json(&frame);
                                let _ = app.emit("pubsub-message", json_val);
                            }
                            Err(radish_proto::ParseError::Incomplete) => {
                                break; // Need more data from socket
                            }
                            Err(e) => {
                                eprintln!("Subscription parse error: {:?}", e);
                                return;
                            }
                        }
                    }
                }
            }
        }
    });

    Ok(())
}

#[tauri::command]
fn unsubscribe_channel(state: State<'_, PubSubState>, channel: String) -> Result<(), String> {
    let mut subs = state.active_subs.lock().unwrap();
    if let Some(cancel_tx) = subs.remove(&channel) {
        let _ = cancel_tx.send(());
    }
    Ok(())
}

pub fn run() {
    let _ = log::set_logger(&LOGGER).map(|()| log::set_max_level(log::LevelFilter::Info));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(PubSubState::default())
        .setup(|app| {
            let handle = app.handle().clone();
            *APP_HANDLE.lock().unwrap() = Some(handle);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_server,
            stop_server,
            restart_server,
            execute_command,
            group_keys,
            subscribe_channel,
            unsubscribe_channel,
            read_config,
            write_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
