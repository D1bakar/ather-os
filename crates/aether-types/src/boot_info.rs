//! Boot-time handoff structures shared between the UEFI boot loader and kernel.
//!
//! # ABI stability
//!
//! Layout is `#[repr(C)]` and versioned via [`BOOT_INFO_VERSION`]. Fields are
//! only appended in future versions; existing offsets and semantics remain stable.
//! Kernels must reject unknown versions rather than interpret them.

/// Magic value identifying a valid [`BootInfo`] block (`b"AETHERBI"`).
pub const BOOT_INFO_MAGIC: u64 = 0x4145_5448_4552_4249;

/// Current [`BootInfo`] layout version.
pub const BOOT_INFO_VERSION: u32 = 1;

/// `BootInfo.flags`: linear framebuffer fields in [`FramebufferInfo`] are valid.
#[allow(dead_code)] // ABI flag — consumed by kernel once framebuffer handoff is wired
pub const BOOT_INFO_FLAG_FRAMEBUFFER: u32 = 1 << 0;

/// UEFI memory type for conventional RAM (UEFI spec value).
pub const MEMORY_TYPE_CONVENTIONAL: u32 = 7;

/// Fixed-layout handoff structure passed from boot loader to kernel entry.
///
/// The boot loader passes a pointer to this structure in the System V AMD64
/// first-argument register (`RDI`) when jumping to the kernel entry point.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct BootInfo {
    /// Must equal [`BOOT_INFO_MAGIC`].
    pub magic: u64,
    /// Must equal [`BOOT_INFO_VERSION`] for this layout.
    pub version: u32,
    /// Reserved for future flags (e.g. framebuffer present).
    pub flags: u32,
    /// Pointer to an array of [`MemoryMapEntry`] (physical address, boot-loader allocated).
    pub memory_map: *const MemoryMapEntry,
    /// Number of entries in the memory map array.
    pub memory_map_len: usize,
    /// Optional linear framebuffer description (may be zeroed when unavailable).
    pub framebuffer: FramebufferInfo,
    /// Physical address of the ACPI RSDP, or `0` when not located.
    pub rsdp: u64,
    /// Primary serial port used for early console output.
    pub serial_port: SerialPortInfo,
}

impl BootInfo {
    /// Returns `true` when magic and version match the current ABI.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.magic == BOOT_INFO_MAGIC && self.version == BOOT_INFO_VERSION
    }
}

/// One UEFI memory map descriptor, normalized for kernel consumption.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MemoryMapEntry {
    /// Start physical address of the region.
    pub phys_start: u64,
    /// Number of 4 KiB pages in the region.
    pub page_count: u64,
    /// UEFI memory type (see UEFI spec).
    pub memory_type: u32,
    /// UEFI attribute flags for the region.
    pub attributes: u64,
}

/// Linear framebuffer metadata (optional in M1).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct FramebufferInfo {
    /// Framebuffer base physical address, or `0`.
    pub base: u64,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Bytes per scan line.
    pub stride: u32,
    /// Pixel format identifier (boot-loader defined; `0` = unknown).
    pub pixel_format: u32,
}

/// Describes the UART used for early serial output.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SerialPortInfo {
    /// I/O port base (e.g. `0x3F8` for COM1).
    pub port: u16,
    /// Configured baud rate (`0` = boot loader did not configure).
    pub baud_rate: u32,
}

impl Default for SerialPortInfo {
    fn default() -> Self {
        Self { port: 0x3F8, baud_rate: 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::{align_of, size_of};

    #[test]
    fn boot_info_is_repr_c_and_stable_size() {
        assert!(size_of::<BootInfo>() >= 64);
        assert_eq!(align_of::<BootInfo>(), 8);
        assert_eq!(align_of::<MemoryMapEntry>(), 8);
    }

    #[test]
    fn framebuffer_flag_is_bit_zero() {
        assert_eq!(BOOT_INFO_FLAG_FRAMEBUFFER, 1);
    }

    #[test]
    fn validation_rejects_bad_magic() {
        let info = BootInfo {
            magic: 0,
            version: BOOT_INFO_VERSION,
            flags: 0,
            memory_map: core::ptr::null(),
            memory_map_len: 0,
            framebuffer: FramebufferInfo::default(),
            rsdp: 0,
            serial_port: SerialPortInfo::default(),
        };
        assert!(!info.is_valid());
    }
}
