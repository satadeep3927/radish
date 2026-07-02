use bytes::BytesMut;
use radish_proto::{parse, ParseError, Frame};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use std::sync::Arc;
use crate::shared::SharedState;

/// Handles a single client's TCP connection.
pub struct Connection {
    id: u64,
    stream: TcpStream,
    buffer: BytesMut,
    out_buf: BytesMut,
}

impl Connection {
    pub fn new(id: u64, stream: TcpStream) -> Self {
        Self {
            id,
            stream,
            buffer: BytesMut::with_capacity(4096),
            out_buf: BytesMut::with_capacity(4096),
        }
    }

    /// Process the connection loop. Reads requests, executes them directly against SharedState,
    /// and writes responses back.
    pub async fn process(mut self, state: Arc<SharedState>) {
        state.active_connections.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // Create an outgoing channel for the PubSub engine to send responses to this connection.
        let (resp_tx, mut resp_rx) = mpsc::channel::<Frame>(1024);

        loop {
            tokio::select! {
                // 1. Read data from the TCP socket
                result = self.read_frame_or_wait() => {
                    if let Err(e) = result {
                        if !e.is_empty() {
                            eprintln!("Connection error: {}", e);
                        }
                        break; // Client disconnected
                    }

                    self.out_buf.clear();

                    // Parse and execute all complete frames from the buffer
                    while let Some(frame) = self.try_parse_frame() {
                        let response = self.execute_frame(frame, &state, &resp_tx);
                        response.serialize(&mut self.out_buf);
                        
                        if self.out_buf.len() > 65536 {
                            break; // Prevent starvation/buffer bloat
                        }
                    }

                    if !self.out_buf.is_empty() {
                        if let Err(e) = self.stream.write_all(&self.out_buf).await {
                            eprintln!("Failed to write to client: {}", e);
                            break;
                        }
                    }
                }
                
                // 2. Receive async responses (e.g., from PubSub)
                response_opt = resp_rx.recv() => {
                    match response_opt {
                        Some(response) => {
                            self.out_buf.clear();
                            response.serialize(&mut self.out_buf);
                            
                            // Auto-pipeline: drain pending responses
                            while let Ok(next_resp) = resp_rx.try_recv() {
                                next_resp.serialize(&mut self.out_buf);
                                if self.out_buf.len() > 65536 {
                                    break; 
                                }
                            }
                            
                            if let Err(e) = self.stream.write_all(&self.out_buf).await {
                                eprintln!("Failed to write to client: {}", e);
                                break;
                            }
                        }
                        None => {
                            break;
                        }
                    }
                }
            }
        }
        
        // Disconnect cleanup
        state.auth.write().unwrap().connected_clients.remove(&self.id);
        state.active_connections.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    }

    fn execute_frame(&self, frame: Frame, state: &SharedState, resp_tx: &mpsc::Sender<Frame>) -> Frame {
        let frames = match frame {
            Frame::Array(frames) if !frames.is_empty() => frames,
            _ => return Frame::Error("ERR syntax error".to_string()),
        };

        let cmd_name = match &frames[0] {
            Frame::Bulk(b) => String::from_utf8_lossy(b).to_uppercase(),
            Frame::Simple(s) => s.to_uppercase(),
            _ => "".to_string(),
        };

        let requires_auth = state.config.requires_auth;
        let is_authenticated = state.auth.read().unwrap().connected_clients.contains_key(&self.id);

        if requires_auth && !is_authenticated && !matches!(cmd_name.as_str(), "AUTH" | "PING" | "HELLO") {
            return Frame::Error("NOAUTH Authentication required.".to_string());
        }

        crate::commands::dispatch(&cmd_name, &frames, self.id, state, resp_tx)
    }

    /// Reads more bytes from the TCP socket into the buffer.
    async fn read_frame_or_wait(&mut self) -> Result<(), String> {
        let n = self.stream.read_buf(&mut self.buffer).await.map_err(|e| e.to_string())?;
        
        if n == 0 {
            // Connection closed by peer
            return Err("".to_string());
        }
        
        Ok(())
    }

    /// Attempts to parse one complete frame from the buffer.
    fn try_parse_frame(&mut self) -> Option<Frame> {
        match parse(&mut self.buffer) {
            Ok((frame, consumed)) => {
                let _ = self.buffer.split_to(consumed);
                Some(frame)
            }
            Err(ParseError::Incomplete) => None,
            Err(ParseError::InvalidProtocol(e)) => {
                eprintln!("Protocol error: {}", e);
                self.buffer.clear();
                None
            }
        }
    }
}
