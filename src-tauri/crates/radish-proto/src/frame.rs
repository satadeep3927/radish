use bytes::{BufMut, Bytes, BytesMut};
use std::fmt;

/// Represents a parsed RESP frame (Redis Serialization Protocol).
#[derive(Clone, Debug, PartialEq)]
pub enum Frame {
    Simple(String),
    Error(String),
    Integer(i64),
    Bulk(Bytes),
    Null,
    Array(Vec<Frame>),
}

impl Frame {
    /// Creates a new Array frame builder
    pub fn array() -> Self {
        Frame::Array(Vec::new())
    }

    /// Appends a Bulk string frame to the array (only works if this is an Array frame)
    pub fn push_bulk(mut self, s: &str) -> Self {
        if let Frame::Array(ref mut arr) = self {
            arr.push(Frame::Bulk(Bytes::from(s.to_string())));
        }
        self
    }

    /// Appends an Integer frame to the array
    pub fn push_int(mut self, i: i64) -> Self {
        if let Frame::Array(ref mut arr) = self {
            arr.push(Frame::Integer(i));
        }
        self
    }

    /// Serializes the frame back into a RESP byte buffer.
    pub fn serialize(&self, buf: &mut BytesMut) {
        match self {
            Frame::Simple(s) => {
                buf.put_u8(b'+');
                buf.put_slice(s.as_bytes());
                buf.put_slice(b"\r\n");
            }
            Frame::Error(e) => {
                buf.put_u8(b'-');
                buf.put_slice(e.as_bytes());
                buf.put_slice(b"\r\n");
            }
            Frame::Integer(i) => {
                buf.put_u8(b':');
                buf.put_slice(i.to_string().as_bytes());
                buf.put_slice(b"\r\n");
            }
            Frame::Bulk(b) => {
                buf.put_u8(b'$');
                buf.put_slice(b.len().to_string().as_bytes());
                buf.put_slice(b"\r\n");
                buf.put_slice(b);
                buf.put_slice(b"\r\n");
            }
            Frame::Null => {
                buf.put_slice(b"$-1\r\n");
            }
            Frame::Array(frames) => {
                buf.put_u8(b'*');
                buf.put_slice(frames.len().to_string().as_bytes());
                buf.put_slice(b"\r\n");
                for frame in frames {
                    frame.serialize(buf);
                }
            }
        }
    }
}

impl fmt::Display for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Frame::Simple(s) => write!(f, "{}", s),
            Frame::Error(e) => write!(f, "(error) {}", e),
            Frame::Integer(i) => write!(f, "(integer) {}", i),
            Frame::Bulk(b) => write!(f, "{}", String::from_utf8_lossy(b)),
            Frame::Null => write!(f, "(nil)"),
            Frame::Array(a) => {
                for (i, frame) in a.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", frame)?;
                }
                Ok(())
            }
        }
    }
}
