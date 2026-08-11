//! Result type alias for Aether OS operations.

use crate::{AetherError, ErrorCode};

/// Standard result type for Aether OS operations.
pub type AetherResult<T> = core::result::Result<T, AetherError>;

/// Converts an `ErrorCode` into a `Result` error.
pub fn from_error_code<T>(code: ErrorCode) -> AetherResult<T> {
    Err(AetherError::new(code))
}

/// Maps an `AetherResult<()>` to an `ErrorCode` suitable for syscall return.
#[must_use]
pub fn to_error_code(result: AetherResult<()>) -> ErrorCode {
    match result {
        Ok(()) => ErrorCode::Success,
        Err(err) => err.code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_error_code_produces_err() {
        let result: AetherResult<()> = from_error_code(ErrorCode::InvalidArgument);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, ErrorCode::InvalidArgument);
    }

    #[test]
    fn to_error_code_success() {
        assert_eq!(to_error_code(Ok(())), ErrorCode::Success);
    }

    #[test]
    fn to_error_code_failure() {
        let err = AetherError::new(ErrorCode::NotFound);
        assert_eq!(to_error_code(Err(err)), ErrorCode::NotFound);
    }
}
