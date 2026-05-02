//! Watchdog: optional termination of EA Desktop / Origin when enabled (same role as `ea_blocker`).
use std::time::Duration;
use tokio::time::interval;

const EA_PROCESSES: &[&str] = &["eadesktop.exe", "origin.exe", "EADesktop.exe", "Origin.exe"];

/// Check if a process name matches EA processes
fn is_ea_process(process_name: &str) -> bool {
    let name_lower = process_name.to_lowercase();
    EA_PROCESSES.iter().any(|&ea_proc| name_lower == ea_proc.to_lowercase())
}

/// Get all running processes
fn get_running_processes() -> Vec<(u32, String)> {
    let mut processes = Vec::new();
    
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        use std::process::{Command, Stdio};
        use winapi::um::winbase::CREATE_NO_WINDOW;
        
        // Use tasklist to get process list (suppress console window)
        if let Ok(output) = Command::new("tasklist")
            .args(&["/FO", "CSV", "/NH"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .output()
        {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                for line in output_str.lines() {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() >= 2 {
                        // Remove quotes from process name
                        let process_name = parts[0].trim_matches('"').to_string();
                        if let Ok(pid_str) = parts[1].trim_matches('"').parse::<u32>() {
                            processes.push((pid_str, process_name));
                        }
                    }
                }
            }
        }
    }
    
    processes
}

/// Terminate a process by PID
fn terminate_process(pid: u32) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        use std::process::{Command, Stdio};
        use winapi::um::winbase::CREATE_NO_WINDOW;
        
        // Use taskkill to terminate the process (suppress console window)
        let output = Command::new("taskkill")
            .args(&["/F", "/PID", &pid.to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        
        if let Ok(result) = output {
            result.status.success()
        } else {
            false
        }
    }
    
    #[cfg(not(windows))]
    {
        false
    }
}

/// Periodically scan for configured launcher processes and terminate them when blocking is enabled.
pub async fn monitor_and_block_ea_processes(enabled: bool) {
    if !enabled {
        return;
    }
    
    // Medium purple color for [Watchdog] tag: RGB(150, 100, 200)
    crate::console_println!("\x1b[38;2;150;100;200m[Watchdog]\x1b[0m Starting EA process monitor...");
    
    let mut interval = interval(Duration::from_secs(2)); // Check every 2 seconds
    
    loop {
        interval.tick().await;
        
        let processes = get_running_processes();
        let mut killed_count = 0;
        
        for (pid, name) in processes {
            if is_ea_process(&name) {
                crate::console_println!("\x1b[38;2;150;100;200m[Watchdog]\x1b[0m Detected EA process '{}' (PID: {}), terminating...", name, pid);
                
                if terminate_process(pid) {
                    killed_count += 1;
                    crate::console_println!("\x1b[38;2;150;100;200m[Watchdog]\x1b[0m Successfully terminated '{}' (PID: {})", name, pid);
                } else {
                    crate::console_println!("\x1b[38;2;150;100;200m[Watchdog]\x1b[0m Failed to terminate '{}' (PID: {})", name, pid);
                }
            }
        }
        
        if killed_count > 0 {
            crate::console_println!("\x1b[38;2;150;100;200m[Watchdog]\x1b[0m Terminated {} EA process(es)", killed_count);
        }
    }
}

/// Check if EA processes are currently running
pub fn check_ea_processes_running() -> Vec<(u32, String)> {
    let processes = get_running_processes();
    processes
        .into_iter()
        .filter(|(_, name)| is_ea_process(name))
        .collect()
}

/// Kill all currently running EA processes
pub fn kill_all_ea_processes() -> usize {
    
    let ea_processes = check_ea_processes_running();
    let mut killed = 0;
    
    // Medium purple color for [Watchdog] tag: RGB(150, 100, 200)
    for (pid, name) in &ea_processes {
        crate::console_println!("\x1b[38;2;150;100;200m[Watchdog]\x1b[0m Terminating EA process '{}' (PID: {})", name, pid);
        if terminate_process(*pid) {
            killed += 1;
        }
    }
    
    if killed > 0 {
        crate::console_println!("\x1b[38;2;150;100;200m[Watchdog]\x1b[0m Terminated {} EA process(es) on startup", killed);
    }
    
    killed
}

