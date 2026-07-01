use std::collections::HashMap;
use radish_proto::Frame;

pub fn handle(
    cmd_name: &str,
    frames: &[Frame],
    conn_id: u64,
    connected_clients: &mut HashMap<u64, String>,
    users: &mut HashMap<String, String>,
) -> Frame {
    match cmd_name {
        "AUTH" => handle_auth(frames, conn_id, connected_clients, users),
        "ACL" => handle_acl(frames, conn_id, connected_clients, users),
        _ => Frame::Error("ERR unknown command".to_string()),
    }
}

fn handle_auth(
    frames: &[Frame],
    conn_id: u64,
    connected_clients: &mut HashMap<u64, String>,
    users: &HashMap<String, String>,
) -> Frame {
    if users.is_empty() {
        return Frame::Simple("OK".to_string());
    }

    let (username, password) = match frames.len() {
        2 => {
            // Legacy AUTH <password> implies username "default"
            if let Frame::Bulk(pass) = &frames[1] {
                ("default".to_string(), String::from_utf8_lossy(pass).to_string())
            } else {
                return Frame::Error("ERR syntax error".to_string());
            }
        }
        3 => {
            // AUTH <username> <password>
            if let (Frame::Bulk(user), Frame::Bulk(pass)) = (&frames[1], &frames[2]) {
                (
                    String::from_utf8_lossy(user).to_string(),
                    String::from_utf8_lossy(pass).to_string(),
                )
            } else {
                return Frame::Error("ERR syntax error".to_string());
            }
        }
        _ => return Frame::Error("ERR wrong number of arguments for 'auth' command".to_string()),
    };

    if let Some(expected_pass) = users.get(&username) {
        if expected_pass == &password {
            connected_clients.insert(conn_id, username);
            return Frame::Simple("OK".to_string());
        }
    }
    
    Frame::Error("WRONGPASS invalid username-password pair".to_string())
}

fn handle_acl(
    frames: &[Frame],
    conn_id: u64,
    connected_clients: &HashMap<u64, String>,
    users: &mut HashMap<String, String>,
) -> Frame {
    if frames.len() < 2 {
        return Frame::Error("ERR wrong number of arguments for 'acl' command".to_string());
    }

    let subcmd = match &frames[1] {
        Frame::Bulk(b) => String::from_utf8_lossy(b).to_uppercase(),
        _ => return Frame::Error("ERR syntax error".to_string()),
    };

    match subcmd.as_str() {
        "WHOAMI" => {
            if let Some(username) = connected_clients.get(&conn_id) {
                Frame::Bulk(username.clone().into_bytes().into())
            } else {
                Frame::Bulk(b"default".to_vec().into())
            }
        }
        "SETUSER" => {
            if frames.len() < 4 {
                return Frame::Error("ERR wrong number of arguments for 'acl setuser' command".to_string());
            }
            let username = match &frames[2] {
                Frame::Bulk(b) => String::from_utf8_lossy(b).to_string(),
                _ => return Frame::Error("ERR syntax error".to_string()),
            };
            
            // Expected format: ACL SETUSER username >password
            let password_rule = match &frames[3] {
                Frame::Bulk(b) => String::from_utf8_lossy(b).to_string(),
                _ => return Frame::Error("ERR syntax error".to_string()),
            };

            if password_rule.starts_with('>') {
                let actual_pass = &password_rule[1..];
                users.insert(username, actual_pass.to_string());
                Frame::Simple("OK".to_string())
            } else {
                Frame::Error("ERR Error in ACL SETUSER modifier".to_string())
            }
        }
        "DELUSER" => {
            if frames.len() < 3 {
                return Frame::Error("ERR wrong number of arguments for 'acl deluser' command".to_string());
            }
            let username = match &frames[2] {
                Frame::Bulk(b) => String::from_utf8_lossy(b).to_string(),
                _ => return Frame::Error("ERR syntax error".to_string()),
            };
            if users.remove(&username).is_some() {
                Frame::Integer(1)
            } else {
                Frame::Integer(0)
            }
        }
        _ => Frame::Error(format!("ERR unknown subcommand or wrong number of arguments for '{}'", subcmd)),
    }
}
