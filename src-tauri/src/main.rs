use std::env;

#[cfg(windows)]
fn handle_console() {
    use windows_sys::Win32::System::Console::{GetConsoleProcessList, GetConsoleWindow};
    use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE};
    
    unsafe {
        // If we are the only process attached to this console, it means Windows
        // created a new console window for us (e.g., user double-clicked the .exe).
        // In that case, we want to hide it immediately since we're launching the GUI.
        let mut processes = [0u32; 2];
        let count = GetConsoleProcessList(processes.as_mut_ptr(), 2);
        if count == 1 {
            let window = GetConsoleWindow();
            if window != 0 {
                ShowWindow(window, SW_HIDE);
            }
        }
    }
}

#[cfg(not(windows))]
fn handle_console() {}

fn main() {
    // Hide the console window if launched via double-click
    handle_console();

    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        let command = &args[1];
        
        if command == "--help" || command == "-h" || command == "help" {
            print_help();
            return;
        }

        match command.as_str() {
            "start" => {
                handle_start(&args);
                return;
            }
            "cli" => {
                handle_cli(&args);
                return;
            }
            "status" | "stop" | "flush" | "inspect" | "service" => {
                handle_admin_command(command, &args);
                return;
            }
            "studio" => {
                // Fallthrough to Tauri GUI
            }
            _ => {
                println!("Unknown command: {}", command);
                print_help();
                return;
            }
        }
    }
    
    // Default to Tauri GUI
    radish_lib::run()
}

fn print_help() {
    println!("Radish: Lightweight, Local-First Valkey-Compatible Database & Studio");
    println!("");
    println!("Usage: radish [COMMAND] [OPTIONS]");
    println!("");
    println!("Commands:");
    println!("  start      Start the database server natively in the terminal");
    println!("             Options: --port <port_number>, --save <interval_seconds>");
    println!("  cli        Launch the interactive REPL client");
    println!("             Options: --host <address>, --port <port_number>");
    println!("  status     Check if the local server is running");
    println!("  stop       Send a graceful SHUTDOWN command to the server");
    println!("  flush      Send a FLUSHALL command to empty the database");
    println!("  inspect    Query diagnostics and memory telemetry (INFO command)");
    println!("  service    'radish service install' displays Windows Service installation instructions");
    println!("  studio     Launch the visual Radish Studio GUI (default behavior)");
    println!("");
    println!("Examples:");
    println!("  radish start --port 6380");
    println!("  radish cli --host 127.0.0.1 --port 6380");
    println!("  radish status");
}

fn parse_port(args: &[String]) -> u16 {
    for i in 2..args.len() {
        if args[i] == "--port" && i + 1 < args.len() {
            if let Ok(p) = args[i + 1].parse::<u16>() {
                return p;
            }
        }
    }
    6379
}

fn handle_start(args: &[String]) {
    let is_daemon = args.iter().any(|a| a == "--daemon");
    
    if !is_daemon {
        use std::process::Command;
        
        let exe_path = env::current_exe().expect("Failed to get current executable path");
        let mut cmd = Command::new(exe_path);
        
        cmd.arg("start");
        cmd.arg("--daemon");
        
        // Pass all other arguments like --port
        for arg in &args[2..] {
            cmd.arg(arg);
        }

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        match cmd.spawn() {
            Ok(_) => println!("Radish server started in the background."),
            Err(e) => println!("Failed to start server in background: {}", e),
        }
        return;
    }

    let mut config = radish_server::config::RadishConfig::load();
    for i in 2..args.len() {
        if args[i] == "--port" && i + 1 < args.len() {
            if let Ok(p) = args[i + 1].parse::<u16>() {
                config.port = p;
            }
        }
        if args[i] == "--save" && i + 1 < args.len() {
            if let Ok(s) = args[i + 1].parse::<u64>() {
                config.save_interval = Some(s);
            }
        }
    }
    let (_tx, rx) = tokio::sync::oneshot::channel();
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    rt.block_on(async {
        radish_server::start(config, rx).await;
    });
}

fn handle_cli(args: &[String]) {
    let mut host = "127.0.0.1".to_string();
    let mut port = 6379;
    for i in 2..args.len() {
        if args[i] == "--host" && i + 1 < args.len() {
            host = args[i + 1].clone();
        }
        if args[i] == "--port" && i + 1 < args.len() {
            if let Ok(p) = args[i + 1].parse::<u16>() {
                port = p;
            }
        }
    }
    radish_cli::start(&host, port);
}

fn handle_admin_command(command: &str, args: &[String]) {
    let port = parse_port(args);
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(async {
        if command == "service" {
            if args.len() > 2 && args[2] == "install" {
                if let Ok(exe_path) = env::current_exe() {
                    println!("To install Radish as a Windows Service, open an Administrator Command Prompt and run:\n");
                    println!("sc create \"Radish\" binPath= \"{} start\" start= auto\n", exe_path.display());
                    println!("To start the service:\nsc start \"Radish\"");
                } else {
                    println!("Failed to determine executable path.");
                }
            } else {
                println!("Usage: radish service install");
            }
            return;
        }

        match radish_cli::client::Client::connect("127.0.0.1", port).await {
            Ok(mut client) => {
                match command {
                    "status" => {
                        if let Err(_) = client.send_command("PING").await {
                            println!("Server is NOT running (connection refused).");
                        } else {
                            if let Ok(_) = client.receive_response().await {
                                println!("Server is RUNNING on port {} (received PONG).", port);
                            } else {
                                println!("Connected, but failed to receive PONG.");
                            }
                        }
                    }
                    "stop" => {
                        if let Err(_) = client.send_command("SHUTDOWN").await {}
                        println!("Stop signal sent to server on port {}.", port);
                    }
                    "flush" => {
                        if let Err(_) = client.send_command("FLUSHALL").await {}
                        if let Ok(_) = client.receive_response().await {
                            println!("Database flushed on port {}.", port);
                        } else {
                            println!("Failed to flush database.");
                        }
                    }
                    "inspect" => {
                        if let Err(_) = client.send_command("INFO").await {}
                        if let Ok(radish_proto::Frame::Bulk(b)) = client.receive_response().await {
                            println!("{}", String::from_utf8_lossy(&b));
                        } else {
                            println!("Failed to fetch server info.");
                        }
                    }
                    _ => {}
                }
            }
            Err(_) => {
                println!("Could not connect to Radish server on port {}. Is it running?", port);
            }
        }
    });
}
