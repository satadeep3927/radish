use crate::frame::Frame;
use bytes::BytesMut;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    Incomplete,
    InvalidProtocol(String),
}

/// Attempts to parse a Frame from the given buffer.
/// Returns Ok(Some((Frame, bytes_consumed))) if a complete frame was parsed.
/// Returns Err(ParseError::Incomplete) if more bytes are needed.
pub fn parse(buf: &mut BytesMut) -> Result<(Frame, usize), ParseError> {
    if buf.is_empty() {
        return Err(ParseError::Incomplete);
    }
    
    match buf[0] {
        b'+' => parse_simple_string(buf),
        b'-' => parse_error(buf),
        b':' => parse_integer(buf),
        b'$' => parse_bulk_string(buf),
        b'*' => parse_array(buf),
        _ => Err(ParseError::InvalidProtocol(format!(
            "Invalid first byte: {}",
            buf[0]
        ))),
    }
}

fn read_line(buf: &[u8]) -> Result<(&[u8], usize), ParseError> {
    for i in 0..buf.len() {
        if buf[i] == b'\r' && i + 1 < buf.len() && buf[i + 1] == b'\n' {
            return Ok((&buf[0..i], i + 2)); // i+2 to consume \r\n
        }
    }
    Err(ParseError::Incomplete)
}

fn parse_simple_string(buf: &[u8]) -> Result<(Frame, usize), ParseError> {
    let (line, len) = read_line(&buf[1..])?;
    let s = String::from_utf8_lossy(line).to_string();
    Ok((Frame::Simple(s), len + 1)) // +1 for the initial byte
}

fn parse_error(buf: &[u8]) -> Result<(Frame, usize), ParseError> {
    let (line, len) = read_line(&buf[1..])?;
    let s = String::from_utf8_lossy(line).to_string();
    Ok((Frame::Error(s), len + 1))
}

fn parse_integer(buf: &[u8]) -> Result<(Frame, usize), ParseError> {
    let (line, len) = read_line(&buf[1..])?;
    let s = std::str::from_utf8(line)
        .map_err(|_| ParseError::InvalidProtocol("Invalid UTF-8 in integer".into()))?;
    let i: i64 = s
        .parse()
        .map_err(|_| ParseError::InvalidProtocol("Invalid integer format".into()))?;
    Ok((Frame::Integer(i), len + 1))
}

fn parse_bulk_string(buf: &[u8]) -> Result<(Frame, usize), ParseError> {
    let (line, consumed_so_far) = read_line(&buf[1..])?;
    let s = std::str::from_utf8(line)
        .map_err(|_| ParseError::InvalidProtocol("Invalid UTF-8 in bulk string length".into()))?;
    let len: i64 = s
        .parse()
        .map_err(|_| ParseError::InvalidProtocol("Invalid bulk string length format".into()))?;
        
    if len == -1 {
        return Ok((Frame::Null, consumed_so_far + 1));
    }
    
    if len < -1 {
        return Err(ParseError::InvalidProtocol("Bulk string length cannot be less than -1".into()));
    }
    
    let len = len as usize;
    let total_len = 1 + consumed_so_far + len + 2; // initial '$' + length line + string + \r\n
    
    if buf.len() < total_len {
        return Err(ParseError::Incomplete);
    }
    
    let data_start = 1 + consumed_so_far;
    let data = &buf[data_start..data_start + len];
    
    // Check for trailing \r\n
    if &buf[data_start + len..total_len] != b"\r\n" {
        return Err(ParseError::InvalidProtocol("Expected \\r\\n after bulk string".into()));
    }
    
    Ok((Frame::Bulk(bytes::Bytes::copy_from_slice(data)), total_len))
}

fn parse_array(buf: &[u8]) -> Result<(Frame, usize), ParseError> {
    let (line, consumed_so_far) = read_line(&buf[1..])?;
    let s = std::str::from_utf8(line)
        .map_err(|_| ParseError::InvalidProtocol("Invalid UTF-8 in array length".into()))?;
    let len: i64 = s
        .parse()
        .map_err(|_| ParseError::InvalidProtocol("Invalid array length format".into()))?;
        
    if len == -1 {
        return Ok((Frame::Null, consumed_so_far + 1));
    }
    
    if len < -1 {
        return Err(ParseError::InvalidProtocol("Array length cannot be less than -1".into()));
    }
    
    let len = len as usize;
    let mut current_offset = 1 + consumed_so_far;
    let mut frames = Vec::with_capacity(len);
    
    for _ in 0..len {
        // We need to parse recursively. But we must pass a slice of the remaining buffer.
        let remaining = &buf[current_offset..];
        if remaining.is_empty() {
            return Err(ParseError::Incomplete);
        }
        
        // Parse recursively using the remaining buffer slice.
        let (frame, consumed) = parse_internal(remaining)?;
        frames.push(frame);
        current_offset += consumed;
    }
    
    Ok((Frame::Array(frames), current_offset))
}

// Internal version that takes a slice instead of BytesMut to allow recursive slicing.
fn parse_internal(buf: &[u8]) -> Result<(Frame, usize), ParseError> {
    if buf.is_empty() {
        return Err(ParseError::Incomplete);
    }
    
    match buf[0] {
        b'+' => parse_simple_string(buf),
        b'-' => parse_error(buf),
        b':' => parse_integer(buf),
        b'$' => parse_bulk_string(buf),
        b'*' => parse_array(buf),
        _ => Err(ParseError::InvalidProtocol(format!(
            "Invalid first byte: {}",
            buf[0]
        ))),
    }
}

