use bytes::BytesMut;
use radish_proto::{parse, ParseError, Frame};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

use crate::engine::EngineMessage;

/// Handles a single client's TCP connection.
pub struct Connection {
    id: u64,
    stream: TcpStream,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(id: u64, stream: TcpStream) -> Self {
        Self {
            id,
            stream,
            buffer: BytesMut::with_capacity(4096),
        }
    }

    /// Process the connection loop. Reads requests, forwards them to the engine,
    /// and writes responses back.
    pub async fn process(mut self, tx: mpsc::Sender<EngineMessage>) {
        // Notify the engine that we connected
        let _ = tx.send(EngineMessage::Connect { conn_id: self.id }).await;

        // Create an outgoing channel for the engine to send responses to this connection.
        // We use an mpsc channel so the engine can send multiple frames (for Pub/Sub).
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

                    // Parse all complete frames from the buffer
                    while let Some(frame) = self.try_parse_frame() {
                        let req = EngineMessage::Command {
                            conn_id: self.id,
                            frame,
                            responder: resp_tx.clone(),
                        };
                        
                        if tx.send(req).await.is_err() {
                            eprintln!("Server engine has shut down.");
                            return;
                        }
                    }
                }
                
                // 2. Receive responses from the Engine and write them to the TCP socket
                response_opt = resp_rx.recv() => {
                    match response_opt {
                        Some(response) => {
                            let mut out_buf = BytesMut::new();
                            response.serialize(&mut out_buf);
                            
                            if let Err(e) = self.stream.write_all(&out_buf).await {
                                eprintln!("Failed to write to client: {}", e);
                                break;
                            }
                        }
                        None => {
                            // The engine dropped our responder channel (server shutting down)
                            break;
                        }
                    }
                }
            }
        }
        
        // Notify the engine that we disconnected so it can clean up ACL state
        let _ = tx.send(EngineMessage::Disconnect { conn_id: self.id }).await;
    }

    /// Reads more bytes from the TCP socket into the buffer.
    async fn read_frame_or_wait(&mut self) -> Result<(), String> {
        let mut temp_buf = [0u8; 4096];
        let n = self.stream.read(&mut temp_buf).await.map_err(|e| e.to_string())?;
        
        if n == 0 {
            // Connection closed by peer
            return Err("".to_string());
        }
        
        self.buffer.extend_from_slice(&temp_buf[0..n]);
        Ok(())
    }

    /// Attempts to parse one complete frame from the buffer.
    /// If parsed, advances the buffer and returns the Frame.
    fn try_parse_frame(&mut self) -> Option<Frame> {
        match parse(&mut self.buffer) {
            Ok((frame, consumed)) => {
                let _ = self.buffer.split_to(consumed);
                Some(frame)
            }
            Err(ParseError::Incomplete) => None,
            Err(ParseError::InvalidProtocol(e)) => {
                eprintln!("Protocol error: {}", e);
                // In a robust implementation, we would send an error back and close the connection.
                // For now, we just clear the buffer.
                self.buffer.clear();
                None
            }
        }
    }
}
