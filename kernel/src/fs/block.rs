//! Block device abstraction (stub).
//!
//! On-disk filesystem backends will read and write fixed-size blocks through
//! this trait. M7 provides a null implementation for compile-time wiring only.

use aether_types::{from_error_code, AetherResult, ErrorCode};

/// Sector-aligned block storage accessed by on-disk filesystems.
pub trait BlockDevice {
    /// Size of one addressable block in bytes.
    fn block_size(&self) -> u32;

    /// Total number of blocks on the device.
    fn block_count(&self) -> u64;

    /// Reads block `index` into `buf` (must be at least [`Self::block_size`] bytes).
    fn read_block(&self, index: u64, buf: &mut [u8]) -> AetherResult<()>;

    /// Writes block `index` from `buf`.
    fn write_block(&mut self, index: u64, buf: &[u8]) -> AetherResult<()>;
}

/// Placeholder block device that reports zero blocks and rejects I/O.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NullBlockDevice;

impl BlockDevice for NullBlockDevice {
    fn block_size(&self) -> u32 {
        512
    }

    fn block_count(&self) -> u64 {
        0
    }

    fn read_block(&self, _index: u64, _buf: &mut [u8]) -> AetherResult<()> {
        from_error_code(ErrorCode::NotSupported)
    }

    fn write_block(&mut self, _index: u64, _buf: &[u8]) -> AetherResult<()> {
        from_error_code(ErrorCode::NotSupported)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_block_device_rejects_io() {
        let mut dev = NullBlockDevice;
        assert_eq!(dev.block_count(), 0);
        assert_eq!(dev.block_size(), 512);
        let mut buf = [0u8; 512];
        assert!(dev.read_block(0, &mut buf).is_err());
        assert!(dev.write_block(0, &buf).is_err());
    }
}
