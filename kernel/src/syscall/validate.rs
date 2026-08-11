//! Userspace pointer validation — delegates to shared `aether-types` helpers.

use aether_types::{
    validate_user_address, validate_user_buffer, validate_user_path_ptr, ErrorCode,
    SecurityDefaults, UserBuffer,
};

/// Validates `[ptr, ptr+len)` for syscall buffer arguments.
pub fn validate_user_slice(ptr: u64, len: u64, _write: bool) -> Result<(), ErrorCode> {
    let config = SecurityDefaults::active();
    if !config.validate_user_pointers {
        return Ok(());
    }

    validate_user_buffer(UserBuffer::new(ptr, len), &config).map(|_| ()).map_err(|err| err.code)
}

/// Validates a NUL-terminated userspace path pointer.
pub fn validate_user_cstr(ptr: u64) -> Result<(), ErrorCode> {
    let config = SecurityDefaults::active();
    if !config.validate_user_pointers {
        return Ok(());
    }

    validate_user_path_ptr(ptr, config.max_user_path_bytes).map(|_| ()).map_err(|err| err.code)
}

/// Validates a single userspace pointer argument.
#[allow(dead_code)]
pub fn validate_user_ptr(ptr: u64) -> Result<(), ErrorCode> {
    validate_user_address(ptr).map(|_| ()).map_err(|err| err.code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_kernel_pointer() {
        assert!(validate_user_slice(0xFFFF_8000_0000_0000, 1, false).is_err());
    }

    #[test]
    fn accepts_user_range() {
        assert!(validate_user_slice(0x1000, 64, true).is_ok());
    }
}
