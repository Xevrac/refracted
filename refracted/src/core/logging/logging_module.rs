/// Custom println! macro that writes to both stdout and GUI buffer
/// This macro uses tracing::info! which goes through LogWriter to GUI buffer
/// The CustomFormatter detects ANSI codes and Blaze markers to skip [Console] prefix
#[macro_export]
macro_rules! console_println {
    ($($arg:tt)*) => {
        {
            // Format the message first - this preserves ANSI codes
            let text = format!($($arg)*);
            
            // Use tracing::info! with the formatted text
            // CustomFormatter will detect ANSI codes ([Client→], [Blaze→], \x1b) and skip [Console] prefix
            // LogWriter will then parse ANSI codes and display with proper colors
            tracing::info!("{}", text);
        }
    };
}



