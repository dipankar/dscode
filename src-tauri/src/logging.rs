//! Conditional logging utilities for DSCode
//!
//! In debug builds, all logging is enabled.
//! In release builds, only error and warning logs are shown.

/// Log levels for filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Get the minimum log level for the current build
pub fn min_log_level() -> LogLevel {
    #[cfg(debug_assertions)]
    {
        LogLevel::Trace
    }
    #[cfg(not(debug_assertions))]
    {
        LogLevel::Warn
    }
}

/// Check if a log level should be displayed
pub fn should_log(level: LogLevel) -> bool {
    level >= min_log_level()
}

/// Conditional debug logging macro
/// Only logs in debug builds
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            println!($($arg)*);
        }
    };
}

/// Conditional info logging macro
/// Only logs in debug builds
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            println!($($arg)*);
        }
    };
}

/// Warning logging macro
/// Always logs
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        eprintln!("[WARN] {}", format!($($arg)*));
    };
}

/// Error logging macro
/// Always logs
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        eprintln!("[ERROR] {}", format!($($arg)*));
    };
}

/// Structured log entry
pub struct LogEntry {
    pub level: LogLevel,
    pub module: String,
    pub message: String,
}

impl LogEntry {
    pub fn new(level: LogLevel, module: &str, message: &str) -> Self {
        Self {
            level,
            module: module.to_string(),
            message: message.to_string(),
        }
    }

    pub fn log(&self) {
        if should_log(self.level) {
            let prefix = match self.level {
                LogLevel::Trace => "[TRACE]",
                LogLevel::Debug => "[DEBUG]",
                LogLevel::Info => "[INFO]",
                LogLevel::Warn => "[WARN]",
                LogLevel::Error => "[ERROR]",
            };

            if self.level >= LogLevel::Warn {
                eprintln!("{} [{}] {}", prefix, self.module, self.message);
            } else {
                println!("{} [{}] {}", prefix, self.module, self.message);
            }
        }
    }
}

/// Helper function for logging with module context
pub fn log(level: LogLevel, module: &str, message: &str) {
    LogEntry::new(level, module, message).log();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_levels_ordering() {
        assert!(LogLevel::Error > LogLevel::Warn);
        assert!(LogLevel::Warn > LogLevel::Info);
        assert!(LogLevel::Info > LogLevel::Debug);
        assert!(LogLevel::Debug > LogLevel::Trace);
    }

    #[test]
    fn test_should_log() {
        // In debug builds, everything should log
        #[cfg(debug_assertions)]
        {
            assert!(should_log(LogLevel::Trace));
            assert!(should_log(LogLevel::Debug));
        }

        // Warn and Error should always log
        assert!(should_log(LogLevel::Warn));
        assert!(should_log(LogLevel::Error));
    }
}
