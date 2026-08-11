//! Logger implementation with pluggable sink.

use crate::LogLevel;

#[cfg(feature = "std")]
use std::string::String;
#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

/// A single log record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LogRecord {
    /// Severity level.
    pub level: LogLevel,
    /// Log target (module or subsystem name).
    pub target: String,
    /// Log message body.
    pub message: String,
}

/// Trait for log output sinks.
pub trait LogSink {
    /// Writes a log record to the sink.
    fn write(&mut self, record: &LogRecord);
}

/// A structured logger with configurable minimum level and sink.
pub struct Logger<S: LogSink> {
    min_level: LogLevel,
    sink: S,
}

impl<S: LogSink> Logger<S> {
    /// Creates a new logger with the given minimum level and sink.
    #[must_use]
    pub fn new(min_level: LogLevel, sink: S) -> Self {
        Self { min_level, sink }
    }

    /// Logs a message if it meets the minimum level threshold.
    pub fn log(&mut self, level: LogLevel, target: &str, message: &str) {
        if !self.min_level.enables(level) {
            return;
        }

        let record = LogRecord { level, target: target.to_owned(), message: message.to_owned() };
        self.sink.write(&record);
    }

    /// Returns the current minimum log level.
    #[must_use]
    pub const fn min_level(&self) -> LogLevel {
        self.min_level
    }

    /// Sets the minimum log level.
    pub fn set_min_level(&mut self, level: LogLevel) {
        self.min_level = level;
    }
}

impl Logger<VecSink> {
    /// Returns a slice of all records captured by the underlying sink.
    #[must_use]
    pub fn records(&self) -> &[LogRecord] {
        &self.sink.records
    }
}

/// Builder for constructing a `Logger`.
pub struct LoggerBuilder<S: LogSink> {
    min_level: LogLevel,
    sink: S,
}

impl<S: LogSink> LoggerBuilder<S> {
    /// Creates a new builder with the given sink.
    #[must_use]
    pub fn new(sink: S) -> Self {
        Self { min_level: LogLevel::Info, sink }
    }

    /// Sets the minimum log level.
    #[must_use]
    pub fn min_level(mut self, level: LogLevel) -> Self {
        self.min_level = level;
        self
    }

    /// Builds the logger.
    #[must_use]
    pub fn build(self) -> Logger<S> {
        Logger::new(self.min_level, self.sink)
    }
}

/// Collects log records into a `Vec` for testing.
#[derive(Default)]
pub struct VecSink {
    /// Collected records.
    pub records: Vec<LogRecord>,
}

impl LogSink for VecSink {
    fn write(&mut self, record: &LogRecord) {
        self.records.push(record.clone());
    }
}

/// Formats log records to stderr (host builds only).
pub struct StdoutSink;

impl LogSink for StdoutSink {
    fn write(&mut self, record: &LogRecord) {
        #[cfg(feature = "std")]
        {
            eprintln!("[{}] {}: {}", record.level, record.target, record.message);
        }
        #[cfg(not(feature = "std"))]
        let _ = record;
    }
}

#[cfg(feature = "std")]
mod global {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    static GLOBAL: OnceLock<Mutex<Logger<VecSink>>> = OnceLock::new();

    /// Initializes the process-global logger.
    pub fn init_global(level: LogLevel) {
        let _ = GLOBAL.set(Mutex::new(Logger::new(level, VecSink::default())));
    }

    /// Logs through the process-global logger.
    pub fn log_global(level: LogLevel, target: &str, message: &str) {
        if let Some(logger) = GLOBAL.get() {
            if let Ok(mut guard) = logger.lock() {
                guard.log(level, target, message);
            }
        }
    }

    /// Returns all records captured by the global logger (intended for tests).
    #[cfg(test)]
    pub fn records() -> Vec<LogRecord> {
        GLOBAL.get().and_then(|l| l.lock().ok()).map(|g| g.sink.records.clone()).unwrap_or_default()
    }
}

#[cfg(feature = "std")]
pub use global::{init_global, log_global};

#[cfg(not(feature = "std"))]
mod nostd_global {
    use super::*;

    /// Initializes the global logger (no-op in `no_std` until M1).
    pub fn init_global(_level: LogLevel) {}

    /// Logs through the global logger (no-op in `no_std` until M1).
    pub fn log_global(_level: LogLevel, _target: &str, _message: &str) {}
}

#[cfg(not(feature = "std"))]
pub use nostd_global::{init_global, log_global};

/// A static logger for `no_std` environments (no global state).
pub struct StaticLogger;

impl StaticLogger {
    /// Logs directly — in `no_std` this is a no-op until a serial sink is wired in M1.
    pub fn log(_level: LogLevel, _target: &str, _message: &str) {}
}

#[cfg(test)]
mod tests {
    use super::global::records;
    use super::*;

    #[test]
    fn logger_respects_min_level() {
        let mut logger = Logger::new(LogLevel::Warn, VecSink::default());
        logger.log(LogLevel::Debug, "test", "hidden");
        logger.log(LogLevel::Error, "test", "visible");
        assert_eq!(logger.records().len(), 1);
        assert_eq!(logger.records()[0].level, LogLevel::Error);
    }

    #[test]
    fn builder_sets_level() {
        let sink = VecSink::default();
        let logger = LoggerBuilder::new(sink).min_level(LogLevel::Trace).build();
        assert_eq!(logger.min_level(), LogLevel::Trace);
    }

    #[test]
    fn global_logger() {
        init_global(LogLevel::Info);
        log_global(LogLevel::Info, "test", "hello");
        let recs = records();
        assert!(!recs.is_empty());
        assert_eq!(recs.last().unwrap().message, "hello");
    }
}
