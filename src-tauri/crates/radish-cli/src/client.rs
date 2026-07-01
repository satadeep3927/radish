use bytes::{Bytes, BytesMut};
use radish_proto::Frame;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use std::error::Error;

pub struct Client {
    stream: TcpStream,
    buffer: BytesMut,
}

impl Client {
    pub async fn connect(host: &str, port: u16) -> Result<Self, Box<dyn Error>> {
        let addr = format!("{}:{}", host, port);
        let stream = TcpStream::connect(&addr).await?;
        Ok(Self {
            stream,
            buffer: BytesMut::with_capacity(4096),
        })
    }

    pub async fn send_command(&mut self, input: &str) -> Result<(), Box<dyn Error>> {
        let mut args = Vec::new();
        let mut current_arg = String::new();
        let mut in_quotes = false;

        for c in input.chars() {
            match c {
                '"' => in_quotes = !in_quotes,
                ' ' if !in_quotes => {
                    if !current_arg.is_empty() {
                        args.push(current_arg.clone());
                        current_arg.clear();
                    }
                }
                _ => current_arg.push(c),
            }
        }
        if !current_arg.is_empty() {
            args.push(current_arg);
        }

        if args.is_empty() {
            return Ok(());
        }

        let mut frames = Vec::new();
        for arg in args {
            frames.push(Frame::Bulk(Bytes::from(arg)));
        }
        let array_frame = Frame::Array(frames);
        
        let mut data = BytesMut::new();
        array_frame.serialize(&mut data);
        
        self.stream.write_all(&data).await?;
        Ok(())
    }

    pub async fn receive_response(&mut self) -> Result<Frame, Box<dyn Error>> {
        loop {
            match radish_proto::parse(&mut self.buffer) {
                Ok((frame, consumed)) => {
                    let _ = self.buffer.split_to(consumed);
                    return Ok(frame);
                }
                Err(radish_proto::ParseError::Incomplete) => {
                    // Need more data
                }
                Err(e) => {
                    return Err(format!("Parse error: {:?}", e).into());
                }
            }

            if self.buffer.capacity() == self.buffer.len() {
                self.buffer.reserve(4096);
            }

            let mut temp_buf = [0u8; 4096];
            let n = self.stream.read(&mut temp_buf).await?;
            if n == 0 {
                return Err("Connection reset by peer".into());
            }
            self.buffer.extend_from_slice(&temp_buf[..n]);
        }
    }
}
