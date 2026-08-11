//! Path validation and permission checks for the VFS layer.
//!
//! Validation runs before any backend-specific lookup. Permission checks are
//! stubbed until the capability broker (post-M4) supplies real credentials.

use aether_types::{AetherError, AetherResult, ErrorCode};

use super::MAX_PATH_LEN;

/// Access mode requested for a path operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccessMode {
    /// Read metadata or file contents.
    Read,
    /// Write file contents or create entries.
    Write,
    /// Execute (reserved for future binary loading).
    Execute,
}

/// Caller identity used by permission checks (stub until M4 capabilities).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Credentials {
    /// Effective user id.
    pub uid: u32,
    /// Effective group id.
    pub gid: u32,
}

impl Credentials {
    /// Kernel superuser credentials (bypass stub checks).
    #[must_use]
    pub const fn kernel() -> Self {
        Self { uid: 0, gid: 0 }
    }
}

/// Validates a VFS path before lookup.
///
/// Rules (M7):
/// - Non-empty, starts with `/`, at most [`super::MAX_PATH_LEN`] bytes
/// - No NUL bytes, no `..` components, no `//` runs
/// - Each component is non-empty except the root `/`
pub fn validate_path(path: &str) -> AetherResult<()> {
    if path.is_empty() {
        return Err(AetherError::new(ErrorCode::InvalidArgument));
    }
    if !path.starts_with('/') {
        return Err(AetherError::new(ErrorCode::InvalidArgument));
    }
    if path.len() > MAX_PATH_LEN {
        return Err(AetherError::new(ErrorCode::InvalidArgument));
    }
    if path.as_bytes().contains(&0) {
        return Err(AetherError::new(ErrorCode::InvalidArgument));
    }
    if path.contains("//") {
        return Err(AetherError::new(ErrorCode::InvalidArgument));
    }
    if path == "/" {
        return Ok(());
    }

    for component in path.split('/').skip(1) {
        if component.is_empty() {
            return Err(AetherError::new(ErrorCode::InvalidArgument));
        }
        if component == ".." {
            return Err(AetherError::new(ErrorCode::InvalidArgument));
        }
    }

    Ok(())
}

/// Checks whether `creds` may access `path` with `mode`.
///
/// Stub implementation: kernel uid (0) is always allowed; other uids are
/// allowed for read/write until real inode permissions exist (M5+).
pub fn check_permission(_path: &str, creds: &Credentials, _mode: AccessMode) -> AetherResult<()> {
    if creds.uid == 0 {
        return Ok(());
    }
    // Non-root access permitted in stub until capability model lands.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_rejects_empty_path() {
        assert!(validate_path("").is_err());
    }

    #[test]
    fn validate_rejects_relative_path() {
        assert!(validate_path("etc/passwd").is_err());
    }

    #[test]
    fn validate_rejects_parent_component() {
        assert!(validate_path("/etc/../passwd").is_err());
    }

    #[test]
    fn validate_accepts_root_and_nested() {
        assert!(validate_path("/").is_ok());
        assert!(validate_path("/init").is_ok());
        assert!(validate_path("/etc/config").is_ok());
    }

    #[test]
    fn check_permission_allows_kernel_and_user_stub() {
        assert!(check_permission("/init", &Credentials::kernel(), AccessMode::Read).is_ok());
        assert!(check_permission(
            "/init",
            &Credentials { uid: 1000, gid: 1000 },
            AccessMode::Write
        )
        .is_ok());
    }
}
