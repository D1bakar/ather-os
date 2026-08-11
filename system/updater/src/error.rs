//! Update-specific error codes.

use core::fmt;

/// Stable error codes for the updater subsystem.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum UpdateErrorCode {
    /// Operation completed successfully.
    Ok = 0,
    /// Manifest or control block failed structural validation.
    InvalidManifest = -100,
    /// Cryptographic signature verification failed.
    SignatureInvalid = -101,
    /// Target slot is not in a state that accepts an update.
    SlotNotReady = -102,
    /// Rollback was requested but no valid previous slot exists.
    RollbackUnavailable = -103,
    /// Update payload hash does not match manifest.
    PayloadHashMismatch = -104,
    /// Trusted public key is unknown or revoked.
    UntrustedKey = -105,
    /// Update subsystem is not yet implemented at runtime.
    NotImplemented = -106,
}

impl UpdateErrorCode {
    /// Returns the numeric value of this error code.
    #[must_use]
    pub const fn as_i32(self) -> i32 {
        self as i32
    }

    /// Converts a raw i32 into an `UpdateErrorCode`, defaulting to `NotImplemented`.
    #[must_use]
    pub const fn from_i32(value: i32) -> Self {
        match value {
            0 => Self::Ok,
            -100 => Self::InvalidManifest,
            -101 => Self::SignatureInvalid,
            -102 => Self::SlotNotReady,
            -103 => Self::RollbackUnavailable,
            -104 => Self::PayloadHashMismatch,
            -105 => Self::UntrustedKey,
            _ => Self::NotImplemented,
        }
    }
}

impl fmt::Display for UpdateErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Ok => "Ok",
            Self::InvalidManifest => "InvalidManifest",
            Self::SignatureInvalid => "SignatureInvalid",
            Self::SlotNotReady => "SlotNotReady",
            Self::RollbackUnavailable => "RollbackUnavailable",
            Self::PayloadHashMismatch => "PayloadHashMismatch",
            Self::UntrustedKey => "UntrustedKey",
            Self::NotImplemented => "NotImplemented",
        };
        write!(f, "{name} ({})", self.as_i32())
    }
}

/// A typed updater error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UpdateError {
    /// The error code.
    pub code: UpdateErrorCode,
}

impl UpdateError {
    /// Creates a new error with the given code.
    #[must_use]
    pub const fn new(code: UpdateErrorCode) -> Self {
        Self { code }
    }

    /// Returns `true` if this represents success.
    #[must_use]
    pub fn is_ok(self) -> bool {
        self.code == UpdateErrorCode::Ok
    }

    /// Returns `true` if this represents a failure.
    #[must_use]
    pub fn is_err(self) -> bool {
        !self.is_ok()
    }
}

impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_code_roundtrip() {
        assert_eq!(UpdateErrorCode::from_i32(-101), UpdateErrorCode::SignatureInvalid);
        assert_eq!(UpdateErrorCode::SignatureInvalid.as_i32(), -101);
    }
}
