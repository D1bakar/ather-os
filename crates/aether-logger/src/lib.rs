//! Structured logging for Aether OS.
//!
//! Provides log levels and a simple logger that works in both host (test) and
//! `no_std` (kernel) environments. In M0 the backend writes to a pluggable
//! sink; M1 will wire this to serial output.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]

extern crate alloc;

mod level;
mod logger;

pub use level::LogLevel;
pub use logger::{LogRecord, Logger, LoggerBuilder, StaticLogger, VecSink};

/// Initializes the global logger with the given minimum level.
pub fn init(level: LogLevel) {
    logger::init_global(level);
}

/// Logs a message at the given level.
pub fn log(level: LogLevel, target: &str, message: &str) {
    logger::log_global(level, target, message);
}

/// Logs at TRACE level.
#[macro_export]
macro_rules! trace {
    ($target:expr, $($arg:tt)*) => {{
        $crate::log(
            $crate::LogLevel::Trace,
            $target,
            &alloc::format!($($arg)*),
        );
    }};
}

/// Logs at DEBUG level.
#[macro_export]
macro_rules! debug {
    ($target:expr, $($arg:tt)*) => {{
        $crate::log(
            $crate::LogLevel::Debug,
            $target,
            &alloc::format!($($arg)*),
        );
    }};
}

/// Logs at INFO level.
#[macro_export]
macro_rules! info {
    ($target:expr, $($arg:tt)*) => {{
        $crate::log(
            $crate::LogLevel::Info,
            $target,
            &alloc::format!($($arg)*),
        );
    }};
}

/// Logs at WARN level.
#[macro_export]
macro_rules! warn {
    ($target:expr, $($arg:tt)*) => {{
        $crate::log(
            $crate::LogLevel::Warn,
            $target,
            &alloc::format!($($arg)*),
        );
    }};
}

/// Logs at ERROR level.
#[macro_export]
macro_rules! error {
    ($target:expr, $($arg:tt)*) => {{
        $crate::log(
            $crate::LogLevel::Error,
            $target,
            &alloc::format!($($arg)*),
        );
    }};
}

/// Logs at PANIC level.
#[macro_export]
macro_rules! panic_log {
    ($target:expr, $($arg:tt)*) => {{
        $crate::log(
            $crate::LogLevel::Panic,
            $target,
            &alloc::format!($($arg)*),
        );
    }};
}

#[cfg(feature = "std")]
extern crate std;
