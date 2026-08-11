//! Userspace pointer validation helpers.

use crate::security_config::SecurityDefaults;
use crate::user::{is_kernel_address, is_user_address, USER_SPACE_MAX, USER_SPACE_MIN};
use crate::{AetherError, AetherResult, ErrorCode};

/// Minimum canonical user address used for pointer checks.
pub const USER_ADDRESS_MIN: u64 = USER_SPACE_MIN;
/// Maximum canonical user address used for pointer checks.
pub const USER_ADDRESS_MAX: u64 = USER_SPACE_MAX;

/// Userspace buffer described by base pointer and length.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UserBuffer {
    ptr: u64,
    len: u64,
}

impl UserBuffer {
    /// Creates a new userspace buffer description.
    #[must_use]
    pub const fn new(ptr: u64, len: u64) -> Self {
        Self { ptr, len }
    }

    /// Base pointer.
    #[must_use]
    pub const fn ptr(self) -> u64 {
        self.ptr
    }

    /// Length in bytes.
    #[must_use]
    pub const fn len(self) -> u64 {
        self.len
    }

    /// Returns `true` when the buffer spans zero bytes.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.len == 0
    }
}

/// Returns `true` when `addr` is a canonical user address.
#[must_use]
pub const fn is_canonical_user_address(addr: u64) -> bool {
    is_user_address(addr)
}

/// Returns `true` when `addr` is not in the user range.
#[must_use]
pub const fn is_non_user_address(addr: u64) -> bool {
    !is_user_address(addr) || is_kernel_address(addr)
}

/// Validates a single userspace pointer.
pub fn validate_user_address(addr: u64) -> AetherResult<u64> {
    if is_kernel_address(addr) || !is_user_address(addr) {
        return Err(AetherError::new(ErrorCode::BadAddress));
    }
    Ok(addr)
}

/// Validates `[buf.ptr(), buf.ptr() + buf.len())` fits in user space without overflow.
pub fn validate_user_buffer(
    buf: UserBuffer,
    _config: &SecurityDefaults,
) -> AetherResult<UserBuffer> {
    if buf.is_empty() {
        return Ok(buf);
    }
    validate_user_address(buf.ptr())?;
    let end = buf.ptr().checked_add(buf.len()).ok_or(AetherError::new(ErrorCode::BadAddress))?;
    if end > USER_ADDRESS_MAX || is_kernel_address(end) {
        return Err(AetherError::new(ErrorCode::BadAddress));
    }
    Ok(buf)
}

/// Validates a NUL-terminated userspace path pointer (stub — length bound only).
pub fn validate_user_path_ptr(ptr: u64, max_len: u64) -> AetherResult<u64> {
    validate_user_address(ptr)?;
    if max_len == 0 {
        return Err(AetherError::new(ErrorCode::BadAddress));
    }
    Ok(ptr)
}
