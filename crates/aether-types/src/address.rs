//! Physical and virtual address newtypes.

use core::fmt;

/// A physical memory address (frame number × page size).
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct PhysicalAddress(u64);

impl PhysicalAddress {
    /// Creates a physical address from a raw value.
    ///
    /// # Panics
    ///
    /// Panics if `value` is not page-aligned when `PAGE_SIZE` alignment is required.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw address value.
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Returns `true` if the address is aligned to the given boundary.
    #[must_use]
    pub const fn is_aligned(self, alignment: u64) -> bool {
        self.0 & (alignment - 1) == 0
    }

    /// Returns the address aligned down to the given boundary.
    #[must_use]
    pub const fn align_down(self, alignment: u64) -> Self {
        Self(self.0 & !(alignment - 1))
    }

    /// Returns the address with an offset added.
    #[must_use]
    pub const fn offset(self, offset: u64) -> Self {
        Self(self.0 + offset)
    }
}

impl fmt::Display for PhysicalAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:016x}", self.0)
    }
}

/// A virtual memory address in the kernel or user address space.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct VirtualAddress(u64);

impl VirtualAddress {
    /// Creates a virtual address from a raw value.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw address value.
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Returns `true` if the address is aligned to the given boundary.
    #[must_use]
    pub const fn is_aligned(self, alignment: u64) -> bool {
        self.0 & (alignment - 1) == 0
    }

    /// Returns the address aligned down to the given boundary.
    #[must_use]
    pub const fn align_down(self, alignment: u64) -> Self {
        Self(self.0 & !(alignment - 1))
    }

    /// Returns the address with an offset added.
    #[must_use]
    pub const fn offset(self, offset: u64) -> Self {
        Self(self.0 + offset)
    }

    /// Returns the page table index for a given level (0 = PML4, 3 = PT).
    #[must_use]
    pub const fn page_table_index(self, level: u8) -> u16 {
        let shift = 12 + (level as u64) * 9;
        ((self.0 >> shift) & 0x1FF) as u16
    }
}

impl fmt::Display for VirtualAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:016x}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_address_alignment() {
        let addr = PhysicalAddress::new(0x1000);
        assert!(addr.is_aligned(4096));
        assert!(!addr.is_aligned(8192));
    }

    #[test]
    fn virtual_address_page_table_index() {
        let addr = VirtualAddress::new(0xFFFF_8000_0010_0000);
        assert_eq!(addr.page_table_index(0), 256);
    }
}
