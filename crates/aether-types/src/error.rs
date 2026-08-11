//! Error codes and the primary error type.

use core::fmt;

/// Stable error codes returned by syscalls and kernel subsystems.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum ErrorCode {
    /// Operation completed successfully.
    Success = 0,
    /// Invalid argument supplied.
    InvalidArgument = -1,
    /// Permission denied.
    PermissionDenied = -2,
    /// Resource not found.
    NotFound = -3,
    /// Resource already exists.
    AlreadyExists = -4,
    /// Out of memory.
    OutOfMemory = -5,
    /// I/O error.
    IoError = -6,
    /// Operation not supported.
    NotSupported = -7,
    /// Resource busy.
    Busy = -8,
    /// Operation timed out.
    TimedOut = -9,
    /// Internal kernel error.
    Internal = -10,
    /// Invalid user-space address or buffer bounds.
    BadAddress = -11,
}

impl ErrorCode {
    /// Returns the numeric value of this error code.
    #[must_use]
    pub const fn as_i32(self) -> i32 {
        self as i32
    }

    /// Converts a raw i32 into an `ErrorCode`, defaulting to `Internal` for unknown values.
    #[must_use]
    pub const fn from_i32(value: i32) -> Self {
        match value {
            0 => Self::Success,
            -1 => Self::InvalidArgument,
            -2 => Self::PermissionDenied,
            -3 => Self::NotFound,
            -4 => Self::AlreadyExists,
            -5 => Self::OutOfMemory,
            -6 => Self::IoError,
            -7 => Self::NotSupported,
            -8 => Self::Busy,
            -9 => Self::TimedOut,
            -10 => Self::Internal,
            -11 => Self::BadAddress,
            _ => Self::Internal,
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Success => "Success",
            Self::InvalidArgument => "InvalidArgument",
            Self::PermissionDenied => "PermissionDenied",
            Self::NotFound => "NotFound",
            Self::AlreadyExists => "AlreadyExists",
            Self::OutOfMemory => "OutOfMemory",
            Self::IoError => "IoError",
            Self::NotSupported => "NotSupported",
            Self::Busy => "Busy",
            Self::TimedOut => "TimedOut",
            Self::Internal => "Internal",
            Self::BadAddress => "BadAddress",
        };
        write!(f, "{name} ({})", self.as_i32())
    }
}

/// A typed error with an optional context message.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AetherError {
    /// The error code.
    pub code: ErrorCode,
}

impl AetherError {
    /// Creates a new error with the given code.
    #[must_use]
    pub const fn new(code: ErrorCode) -> Self {
        Self { code }
    }

    /// Returns `true` if this represents success.
    #[must_use]
    pub fn is_ok(self) -> bool {
        self.code == ErrorCode::Success
    }

    /// Returns `true` if this represents a failure.
    #[must_use]
    pub fn is_err(self) -> bool {
        !self.is_ok()
    }
}

impl fmt::Display for AetherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_code_roundtrip() {
        assert_eq!(ErrorCode::from_i32(-5), ErrorCode::OutOfMemory);
        assert_eq!(ErrorCode::OutOfMemory.as_i32(), -5);
    }
}
