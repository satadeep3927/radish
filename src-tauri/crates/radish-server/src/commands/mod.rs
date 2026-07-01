pub mod connection;
pub mod generic;
pub mod hash;
pub mod list;
pub mod pubsub;
pub mod set;
pub mod string;
pub mod acl;

use bytes::Bytes;
use radish_proto::Frame;

/// Helper to safely extract Bytes from Simple or Bulk strings
pub fn extract_bytes(frame: &Frame) -> Option<Bytes> {
    match frame {
        Frame::Bulk(b) => Some(b.clone()),
        Frame::Simple(s) => Some(Bytes::from(s.clone())),
        _ => None,
    }
}

/// Iteratively matches a glob pattern against a string to prevent stack overflow
pub fn match_glob(pattern: &[u8], string: &[u8]) -> bool {
    let mut p_idx = 0;
    let mut s_idx = 0;
    let mut star_idx = None;
    let mut s_recall = 0;

    while s_idx < string.len() {
        if p_idx < pattern.len() && (pattern[p_idx] == b'?' || pattern[p_idx] == string[s_idx]) {
            p_idx += 1;
            s_idx += 1;
        } else if p_idx < pattern.len() && pattern[p_idx] == b'*' {
            star_idx = Some(p_idx);
            s_recall = s_idx;
            p_idx += 1;
        } else if let Some(last_star) = star_idx {
            p_idx = last_star + 1;
            s_recall += 1;
            s_idx = s_recall;
        } else {
            return false;
        }
    }

    while p_idx < pattern.len() && pattern[p_idx] == b'*' {
        p_idx += 1;
    }

    p_idx == pattern.len()
}
