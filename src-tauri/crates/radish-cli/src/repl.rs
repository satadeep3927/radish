use crate::client::Client;
use radish_proto::Frame;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, ExternalPrinter};
use std::sync::{Arc, Mutex};
use crate::ui::format_frame;

pub async fn start_repl(host: &str, port: u16) {
    let mut client = match Client::connect(host, port).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Could not connect to Radish at {}:{}: {}", host, port, e);
            return;
        }
    };

    println!("Connected to Radish at {}:{}", host, port);
    
    let mut rl = DefaultEditor::new().unwrap();
    let mut printer = rl.create_external_printer().unwrap();
    
    let normal_prompt = format!("{}:{}> ", host, port);
    let sub_prompt = format!("{}:{}(subscribed mode)> ", host, port);
    
    let current_prompt = Arc::new(Mutex::new(normal_prompt.clone()));
    let prompt_clone = current_prompt.clone();
    
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Result<String, ReadlineError>>(10);
    let (ack_tx, ack_rx) = std::sync::mpsc::channel::<()>();

    // Background thread for blocking readline
    std::thread::spawn(move || {
        loop {
            let p = { prompt_clone.lock().unwrap().clone() };
            let readline = rl.readline(&p);
            
            // Send the result to the main thread
            let is_eof = matches!(readline, Err(ReadlineError::Eof));
            
            if tx.blocking_send(readline).is_err() || is_eof {
                break;
            }
            // Wait for main loop to update prompt before looping
            if ack_rx.recv().is_err() {
                break;
            }
        }
    });

    let mut is_subscribed = false;

    loop {
        tokio::select! {
            // 1. User input
            line_res = rx.recv() => {
                match line_res {
                    Some(Ok(line)) => {
                        let input = line.trim();
                        if input.is_empty() {
                            let _ = ack_tx.send(());
                            continue;
                        }
                        
                        if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
                            let _ = ack_tx.send(());
                            break;
                        }

                        if is_subscribed {
                            if input.eq_ignore_ascii_case("clear") {
                                // Allow clear
                            } else if input.to_uppercase().starts_with("UNSUBSCRIBE") {
                                if let Err(e) = client.send_command(input).await {
                                    printer.print(format!("Error: {}", e)).unwrap();
                                }
                            } else {
                                printer.print("ERR only (P)SUBSCRIBE / (P)UNSUBSCRIBE / PING / QUIT allowed in this context\n".to_string()).unwrap();
                            }
                        } else {
                            if input.to_uppercase().starts_with("SUBSCRIBE") {
                                is_subscribed = true;
                                *current_prompt.lock().unwrap() = sub_prompt.clone();
                            }
                            
                            if let Err(e) = client.send_command(input).await {
                                printer.print(format!("Error: {}", e)).unwrap();
                                let _ = ack_tx.send(());
                                break;
                            }
                        }
                        let _ = ack_tx.send(());
                    }
                    Some(Err(ReadlineError::Interrupted)) => {
                        if is_subscribed {
                            if let Err(e) = client.send_command("UNSUBSCRIBE").await {
                                printer.print(format!("Error: {}", e)).unwrap();
                            }
                            is_subscribed = false;
                            *current_prompt.lock().unwrap() = normal_prompt.clone();
                            printer.print("Unsubscribed.\n".to_string()).unwrap();
                            let _ = ack_tx.send(());
                            continue;
                        } else {
                            let _ = ack_tx.send(());
                            break;
                        }
                    }
                    Some(Err(ReadlineError::Eof)) => {
                        let _ = ack_tx.send(());
                        break;
                    }
                    Some(Err(err)) => {
                        printer.print(format!("Error: {:?}\n", err)).unwrap();
                        let _ = ack_tx.send(());
                        break;
                    }
                    None => break,
                }
            }
            
            // 2. Server responses (async)
            resp_res = client.receive_response() => {
                match resp_res {
                    Ok(frame) => {
                        let s = format_frame(&frame, 0);
                        printer.print(format!("{}\n", s)).unwrap();
                        
                        // If we received unsubscribe confirmation, ensure we are in normal mode
                        if let Frame::Array(arr) = &frame {
                            if arr.len() >= 1 {
                                if let Frame::Bulk(b) = &arr[0] {
                                    if String::from_utf8_lossy(b).to_lowercase() == "unsubscribe" {
                                        is_subscribed = false;
                                        *current_prompt.lock().unwrap() = normal_prompt.clone();
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        printer.print(format!("Error reading response: {}\n", e)).unwrap();
                        break;
                    }
                }
            }
        }
    }
}


