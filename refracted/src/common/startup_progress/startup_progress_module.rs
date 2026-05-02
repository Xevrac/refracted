// Startup progress logging - single line that updates in place
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

static STARTUP_IN_PROGRESS: AtomicBool = AtomicBool::new(false);
static CURRENT_STARTUP_MESSAGE: OnceLock<Mutex<Option<String>>> = OnceLock::new();
static LAST_MESSAGE_TIME: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();

fn get_message_mutex() -> &'static Mutex<Option<String>> {
    CURRENT_STARTUP_MESSAGE.get_or_init(|| Mutex::new(None))
}

/// Start startup progress mode (suppresses intermediate messages)
pub fn start_startup_progress() {
    STARTUP_IN_PROGRESS.store(true, Ordering::SeqCst);
    *get_message_mutex().lock().unwrap() = None;
    // Reset the last message time
    if let Some(last_time) = LAST_MESSAGE_TIME.get() {
        *last_time.lock().unwrap() = None;
    }
}

/// Log a startup progress message (replaces previous message on same line)
pub fn log_startup_progress(message: &str) {
    // Get or initialize the last message time tracker
    let last_time = LAST_MESSAGE_TIME.get_or_init(|| Mutex::new(None));
    
    // Check if enough time has passed since the last message (500ms delay)
    let mut last_time_guard = last_time.lock().unwrap();
    if let Some(last) = *last_time_guard {
        let elapsed = last.elapsed();
        if elapsed < Duration::from_millis(500) {
            // Wait for the remaining time
            std::thread::sleep(Duration::from_millis(500) - elapsed);
        }
    }
    *last_time_guard = Some(Instant::now());
    drop(last_time_guard);
    
    // Update the current message
    *get_message_mutex().lock().unwrap() = Some(message.to_string());
    
    // Also print to stdout with carriage return for console
    print!("\r\x1b[K{}", message);
    std::io::Write::flush(&mut std::io::stdout()).ok();
}

/// Finish startup progress (print newline and clear)
pub fn finish_startup_progress() {
    print!("\r\x1b[K\n");
    std::io::Write::flush(&mut std::io::stdout()).ok();
    STARTUP_IN_PROGRESS.store(false, Ordering::SeqCst);
    *get_message_mutex().lock().unwrap() = None;
}

/// Check if startup is in progress (for suppressing intermediate messages)
pub fn is_startup_in_progress() -> bool {
    STARTUP_IN_PROGRESS.load(Ordering::SeqCst)
}

/// Get the current startup message (for GUI display)
pub fn get_current_startup_message() -> Option<String> {
    get_message_mutex().lock().unwrap().clone()
}

