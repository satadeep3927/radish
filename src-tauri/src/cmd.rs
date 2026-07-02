use std::env;
use std::process::Command;
use std::path::PathBuf;

/// Parses and executes command line arguments.
/// Returns true if the Tauri GUI should be launched, false otherwise.
pub fn execute(args: &[String]) -> bool {
    if args.len() <= 1 {
        return true; // Default to Studio GUI
    }

    let command = args[1].as_str();

    match command {
        "--help" | "-h" | "help" => {
            print_help();
            false
        }
        "start" => {
            handle_start(args);
            false
        }
        "cli" => {
            handle_cli(args);
            false
        }
        "status" | "stop" | "flush" | "inspect" | "service" => {
            handle_admin_command(command, args);
            false
        }
        "studio" => true,
        _ => {
            println!("Unknown command: {}", command);
            print_help();
            false
        }
    }
}

fn print_help() {
    println!("Radish: Lightweight, Local-First Valkey-Compatible Database & Studio\n");
    println!("Usage: radish [COMMAND] [OPTIONS]\n");
    println!("Commands:");
    println!("  start      Start the database server natively in the terminal");
    println!("             Options: --port <port_number>, --save <interval_seconds>");
    println!("  cli        Launch the interactive REPL client");
    println!("             Options: --host <address>, --port <port_number>");
    println!("  status     Check if the local server is running");
    println!("  stop       Send a graceful SHUTDOWN command to the server");
    println!("  flush      Send a FLUSHALL command to empty the database");
    println!("  inspect    Query diagnostics and memory telemetry (INFO command)");
    println!("  service    'radish service install | uninstall' manages background Windows Services");
    println!("  studio     Launch the visual Radish Studio GUI (default behavior)\n");
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
        let exe_path = env::current_exe().expect("Failed to get current executable path");
        let mut cmd = Command::new(exe_path);
        
        cmd.arg("start").arg("--daemon");
        
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
    if command == "service" {
        handle_service_command(args);
        return;
    }

    let port = parse_port(args);
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    
    rt.block_on(async {
        let mut client = match radish_cli::client::Client::connect("127.0.0.1", port).await {
            Ok(c) => c,
            Err(_) => {
                println!("Could not connect to Radish server on port {}. Is it running?", port);
                return;
            }
        };

        // Authenticate automatically if required by the configuration
        let config = radish_server::config::RadishConfig::load();
        if config.requires_auth {
            let pwd = if config.password.is_empty() { "radish" } else { &config.password };
            let _ = client.send_command(&format!("AUTH {}", pwd)).await;
            if let Ok(radish_proto::Frame::Error(e)) = client.receive_response().await {
                println!("Authentication failed: {}", e);
                return;
            }
        }

        match command {
            "status" => {
                let _ = client.send_command("PING").await;
                match client.receive_response().await {
                    Ok(radish_proto::Frame::Simple(s)) => {
                        if s.to_uppercase() == "PONG" {
                            println!("Server is RUNNING on port {} (received PONG).", port);
                        } else {
                            println!("Server is RUNNING on port {}.", port);
                        }
                    }
                    Ok(radish_proto::Frame::Bulk(b)) => {
                        let s = String::from_utf8_lossy(&b).to_string();
                        if s.to_uppercase() == "PONG" {
                            println!("Server is RUNNING on port {} (received PONG).", port);
                        } else {
                            println!("Server is RUNNING on port {}.", port);
                        }
                    }
                    Ok(radish_proto::Frame::Error(e)) => println!("Server returned error: {}", e),
                    _ => println!("Connected, but failed to receive a valid response."),
                }
            }
            "stop" => {
                let _ = client.send_command("SHUTDOWN").await;
                match client.receive_response().await {
                    Ok(radish_proto::Frame::Error(e)) => println!("Failed to stop server: {}", e),
                    _ => println!("Stop signal sent to server on port {}.", port),
                }
            }
            "flush" => {
                let _ = client.send_command("FLUSHALL").await;
                match client.receive_response().await {
                    Ok(radish_proto::Frame::Error(e)) => println!("Failed to flush database: {}", e),
                    _ => println!("Database flushed on port {}.", port),
                }
            }
            "inspect" => {
                if client.send_command("INFO").await.is_ok() {
                    match client.receive_response().await {
                        Ok(radish_proto::Frame::Bulk(data)) => {
                            if let Ok(s) = std::str::from_utf8(&data) {
                                println!("{}", s);
                            }
                        }
                        Ok(radish_proto::Frame::Error(e)) => println!("Failed to inspect server: {}", e),
                        _ => println!("Invalid response from server."),
                    }
                }
            }
            _ => {}
        }
    });
}

fn handle_service_command(args: &[String]) {
    let subcmd = args.get(2).map(|s| s.as_str()).unwrap_or("");
    
    let exe_path = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => {
            println!("Failed to determine executable path.");
            return;
        }
    };

    match subcmd {
        "install" => install_service(&exe_path),
        "uninstall" => uninstall_service(),
        _ => println!("Unknown service subcommand. Use: radish service install | uninstall"),
    }
}

fn execute_elevated_powershell(script: &str, success_msg: &str, err_msg: &str) {
    let mut temp_path = std::env::temp_dir();
    temp_path.push("radish_service_temp.ps1");
    
    if let Ok(mut file) = std::fs::File::create(&temp_path) {
        use std::io::Write;
        let _ = file.write_all(script.as_bytes());
        
        let status = std::process::Command::new("powershell")
            .args(&["-ExecutionPolicy", "Bypass", "-WindowStyle", "Hidden", "-File", temp_path.to_str().unwrap()])
            .status();
            
        let _ = std::fs::remove_file(temp_path);
        
        match status {
            Ok(s) if s.success() => println!("{}", success_msg),
            _ => println!("{}", err_msg),
        }
    } else {
        println!("Failed to create temporary script for service operation.");
    }
}

fn install_service(exe_path: &PathBuf) {
    let script = format!(
        "Start-Process sc.exe -ArgumentList 'create Radish binPath= \"\\\"{}\\\" start\" start= auto' -Verb RunAs -Wait",
        exe_path.display()
    );
    
    println!("Requesting Administrator privileges to install Radish as a Windows Service...");
    execute_elevated_powershell(
        &script,
        "Successfully requested service installation.\nTo start it, open an Administrator prompt and run: sc start Radish",
        "Failed to install service or user cancelled the UAC prompt."
    );
}

fn uninstall_service() {
    let script = "Start-Process cmd.exe -ArgumentList '/c sc stop Radish & sc delete Radish' -Verb RunAs -Wait".to_string();
    
    println!("Requesting Administrator privileges to uninstall Radish as a Windows Service...");
    execute_elevated_powershell(
        &script,
        "Successfully requested service uninstallation.",
        "Failed to uninstall service or user cancelled the UAC prompt."
    );
}
