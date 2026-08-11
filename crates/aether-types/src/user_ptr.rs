//! User-space pointer validation helpers.
//!
//! Pure functions for checking that syscall arguments reference canonical,
//! in-bounds user addresses before the kernel dereferences them.

use crate::{AetherError, AetherResult, ErrorCode, SecurityDefaults, VirtualAddress};

/// Inclusive upper bound of the x86_64 user canonical address range.
pub const USER_ADDRESS_MAX: u64 = 0x0000_7FFF_FFFF_FFFF;

/// Minimum non-null user address (page 0 is never mapped for user).
pub const USER_ADDRESS_MIN: u64 = 0x1;

/// A user-space buffer described by base address and byte length.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UserBuffer {
    /// Buffer base address.
    pub base: VirtualAddress,
    /// Buffer length in bytes (may be zero).
    pub len: u64,
}

impl UserBuffer {
    /// Creates a buffer descriptor from raw syscall arguments.
    #[must_use]
    pub const fn new(base: u64, len: u64) -> Self {
        Self { base: VirtualAddress::new(base), len }
    }

    /// Returns the inclusive end address, or `None` on overflow.
    #[must_use]
    pub const fn end_exclusive(&self) -> Option<u64> {
        self.base.as_u64().checked_add(self.len)
    }
}

/// Returns `true` if `addr` lies in the canonical user range.
#[must_use]
pub const fn is_canonical_user_address(addr: u64) -> bool {
    addr >= USER_ADDRESS_MIN && addr <= USER_ADDRESS_MAX
}

/// Returns `true` if `addr` is in the non-canonical x86_64 hole or kernel half.
#[must_use]
pub const fn is_non_user_address(addr: u64) -> bool {
    !is_canonical_user_address(addr)
}

/// Validates a single user pointer (non-null, canonical, in user range).
pub const fn validate_user_address(addr: u64) -> AetherResult<VirtualAddress> {
    if addr == 0 {
        return Err(AetherError::new(ErrorCode::BadAddress));
    }
    if is_non_user_address(addr) {
        return Err(AetherError::new(ErrorCode::BadAddress));
    }
    Ok(VirtualAddress::new(addr))
}

/// Validates a user buffer against policy limits.
pub fn validate_user_buffer(
    buffer: UserBuffer,
    config: &SecurityDefaults,
) -> AetherResult<UserBuffer> {
    if buffer.len > config.max_user_copy_bytes {
        return Err(AetherError::new(ErrorCode::InvalidArgument));
    }

    let base = buffer.base.as_u64();
    if base == 0 && buffer.len > 0 {
        return Err(AetherError::new(ErrorCode::BadAddress));
    }
    if base != 0 {
        validate_user_address(base)?;
    }

    if buffer.len > 0 {
        let end = buffer.end_exclusive().ok_or(AetherError::new(ErrorCode::BadAddress))?;
        if end > USER_ADDRESS_MAX + 1 {
            return Err(AetherError::new(ErrorCode::BadAddress));
        }
        // Wraparound: end <= base with non-zero len.
        if end <= base {
            return Err(AetherError::new(ErrorCode::BadAddress));
        }
    }

    Ok(buffer)
}

/// Validates a NUL-terminated path pointer and enforces maximum path length.
pub fn validate_user_path_ptr(addr: u64, max_len: u64) -> AetherResult<VirtualAddress> {
    let ptr = validate_user_address(addr)?;
    if max_len == 0 || max_len > SecurityDefaults::PRODUCTION.max_user_path_bytes {
        return Err(AetherError::new(ErrorCode::InvalidArgument));
    }
    let end = addr.checked_add(max_len).ok_or(AetherError::new(ErrorCode::BadAddress))?;
    if end > USER_ADDRESS_MAX {
        return Err(AetherError::new(ErrorCode::BadAddress));
    }
    Ok(ptr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_null_pointer() {
        assert!(validate_user_address(0).is_err());
    }

    #[test]
    fn rejects_kernel_pointer() {
        assert!(validate_user_address(0xFFFF_8000_0000_0000).is_err());
    }

    #[test]
    fn accepts_valid_user_pointer() {
        let addr = validate_user_address(0x1000).unwrap();
        assert_eq!(addr.as_u64(), 0x1000);
    }

    #[test]
    fn rejects_buffer_overflow() {
        let cfg = SecurityDefaults::active();
        let buf = UserBuffer::new(USER_ADDRESS_MAX, 2);
        assert!(validate_user_buffer(buf, &cfg).is_err());
    }

    #[test]
    fn rejects_excessive_copy_length() {
        let cfg = SecurityDefaults::active();
        let buf = UserBuffer::new(0x1000, cfg.max_user_copy_bytes + 1);
        assert!(validate_user_buffer(buf, &cfg).is_err());
    }

    #[test]
    fn zero_length_buffer_at_null_is_allowed() {
        let cfg = SecurityDefaults::active();
        let buf = UserBuffer::new(0, 0);
        assert!(validate_user_buffer(buf, &cfg).is_ok());
    }
}
