//! Package manager error types.

use core::fmt;

/// Result alias for package manager operations.
pub type PkgResult<T> = Result<T, PkgError>;

/// Errors returned by manifest parsing, verification, and install/uninstall.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PkgError {
    /// Manifest version or field is invalid.
    InvalidManifest(&'static str),
    /// Package archive layout is invalid or incomplete.
    InvalidPackage(&'static str),
    /// Signature block failed structural or cryptographic checks.
    InvalidSignature(&'static str),
    /// Requested package is not installed.
    NotInstalled,
    /// Package is already installed.
    AlreadyInstalled,
    /// A dependency requirement is unsatisfied.
    UnsatisfiedDependency {
        /// Package that was being installed.
        package: alloc::string::String,
        /// Missing or incompatible dependency name.
        dependency: alloc::string::String,
    },
    /// Underlying I/O failure.
    Io(alloc::string::String),
    /// TOML or other parse failure.
    Parse(alloc::string::String),
    /// Cryptographic verification is not enabled in this build.
    VerificationNotEnabled,
}

impl fmt::Display for PkgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidManifest(msg) => write!(f, "invalid manifest: {msg}"),
            Self::InvalidPackage(msg) => write!(f, "invalid package: {msg}"),
            Self::InvalidSignature(msg) => write!(f, "invalid signature: {msg}"),
            Self::NotInstalled => write!(f, "package not installed"),
            Self::AlreadyInstalled => write!(f, "package already installed"),
            Self::UnsatisfiedDependency { package, dependency } => {
                write!(f, "package `{package}` requires `{dependency}`")
            }
            Self::Io(msg) => write!(f, "I/O error: {msg}"),
            Self::Parse(msg) => write!(f, "parse error: {msg}"),
            Self::VerificationNotEnabled => {
                write!(f, "signature verification not enabled (rebuild with `verify` feature)")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for PkgError {}

extern crate alloc;
