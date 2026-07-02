mod cmd;

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

    let args: Vec<String> = std::env::args().collect();
    
    // Defer CLI and backend execution logic to the `cmd` module.
    // `execute` returns true only if it expects the application to boot the Studio GUI.
    if cmd::execute(&args) {
        radish_lib::run();
    }
}
