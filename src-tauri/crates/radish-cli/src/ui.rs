use colored::Colorize;
use radish_proto::Frame;

/// Recursively formats a RESP frame with syntax highlighting for the CLI.
pub fn format_frame(frame: &Frame, indent: usize) -> String {
    let prefix = " ".repeat(indent);
    match frame {
        Frame::Simple(s) => format!("{}{}", prefix, s.green()),
        Frame::Error(e) => format!("{}{}", prefix, format!("(error) {}", e).red().bold()),
        Frame::Integer(i) => format!("{}{}", prefix, format!("(integer) {}", i).yellow()),
        Frame::Bulk(b) => format!("{}\"{}\"", prefix, String::from_utf8_lossy(b).cyan()),
        Frame::Null => format!("{}{}", prefix, "(nil)".bright_black()),
        Frame::Array(arr) => {
            if arr.is_empty() {
                format!("{}{}", prefix, "(empty array)".bright_black())
            } else {
                let mut res = String::new();
                for (i, f) in arr.iter().enumerate() {
                    let index_str = format!("{}) ", i + 1).bright_blue();
                    let inner = format_frame(f, indent + 3).trim_start().to_string();
                    res.push_str(&format!("{}{}", index_str, inner));
                    if i < arr.len() - 1 {
                        res.push('\n');
                        res.push_str(&prefix); // Fix alignment for nested arrays if any
                    }
                }
                res
            }
        }
    }
}
