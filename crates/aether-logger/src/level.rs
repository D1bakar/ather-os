//! Log level definitions.

use core::fmt;

/// Log severity levels, ordered from most to least verbose.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum LogLevel {
    /// Fine-grained diagnostic information.
    Trace = 0,
    /// General diagnostic information.
    Debug = 1,
    /// Informational messages.
    Info = 2,
    /// Warning conditions.
    Warn = 3,
    /// Error conditions.
    Error = 4,
    /// Unrecoverable failure.
    Panic = 5,
}

impl LogLevel {
    /// Returns the single-character level indicator.
    #[must_use]
    pub const fn as_char(self) -> char {
        match self {
            Self::Trace => 'T',
            Self::Debug => 'D',
            Self::Info => 'I',
            Self::Warn => 'W',
            Self::Error => 'E',
            Self::Panic => 'P',
        }
    }

    /// Returns the level name as a static string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
            Self::Panic => "PANIC",
        }
    }

    /// Returns `true` if messages at `other` should be emitted given `self` as the filter level.
    #[must_use]
    pub const fn enables(self, other: LogLevel) -> bool {
        other as u8 >= self as u8
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_filtering() {
        assert!(LogLevel::Info.enables(LogLevel::Warn));
        assert!(!LogLevel::Warn.enables(LogLevel::Debug));
    }
}
