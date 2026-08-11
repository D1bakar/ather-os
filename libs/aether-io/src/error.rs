//! I/O error type.

/// Error returned by [`crate::Read`] and [`crate::Write`] operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoError {
    /// The underlying device is not ready (e.g. transmit buffer full).
    WouldBlock,
    /// The operation failed due to invalid parameters.
    InvalidInput,
    /// An unspecified device or transport failure.
    DeviceError,
    /// End of stream reached (read returned no data).
    EndOfStream,
}

impl IoError {
    /// Returns a short static description suitable for early-boot logging.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WouldBlock => "would block",
            Self::InvalidInput => "invalid input",
            Self::DeviceError => "device error",
            Self::EndOfStream => "end of stream",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_strings_are_non_empty() {
        assert!(!IoError::WouldBlock.as_str().is_empty());
        assert!(!IoError::EndOfStream.as_str().is_empty());
    }
}
