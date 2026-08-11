//! Page size and flag definitions.

/// Supported page sizes in Aether OS.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PageSize {
    /// Standard 4 KiB page.
    Size4KiB = 4096,
    /// Large 2 MiB page.
    Size2MiB = 2_097_152,
    /// Huge 1 GiB page.
    Size1GiB = 1_073_741_824,
}

impl PageSize {
    /// Returns the page size in bytes.
    #[must_use]
    pub const fn bytes(self) -> u64 {
        self as u64
    }

    /// Returns the page shift (log2 of page size).
    #[must_use]
    pub const fn shift(self) -> u8 {
        match self {
            Self::Size4KiB => 12,
            Self::Size2MiB => 21,
            Self::Size1GiB => 30,
        }
    }
}

/// x86_64 page table entry flags.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PageFlags(u64);

impl PageFlags {
    /// No flags set.
    pub const EMPTY: Self = Self(0);
    /// Page is present in memory.
    pub const PRESENT: Self = Self(1 << 0);
    /// Page is writable.
    pub const WRITABLE: Self = Self(1 << 1);
    /// Page is accessible from user mode.
    pub const USER: Self = Self(1 << 2);
    /// Write-through caching.
    pub const WRITE_THROUGH: Self = Self(1 << 3);
    /// Disable caching.
    pub const NO_CACHE: Self = Self(1 << 4);
    /// Page has been accessed.
    pub const ACCESSED: Self = Self(1 << 5);
    /// Page has been written to.
    pub const DIRTY: Self = Self(1 << 6);
    /// Huge page (2 MiB or 1 GiB).
    pub const HUGE: Self = Self(1 << 7);
    /// Global page (not flushed on CR3 switch).
    pub const GLOBAL: Self = Self(1 << 8);
    /// No-execute bit.
    pub const NO_EXECUTE: Self = Self(1 << 63);

    /// Creates flags from a raw value.
    #[must_use]
    pub const fn from_bits(bits: u64) -> Self {
        Self(bits)
    }

    /// Returns the raw flag bits.
    #[must_use]
    pub const fn bits(self) -> u64 {
        self.0
    }

    /// Returns `true` if all bits in `other` are set in `self`.
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Combines two flag sets.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_flags_union() {
        let flags = PageFlags::PRESENT.union(PageFlags::WRITABLE);
        assert!(flags.contains(PageFlags::PRESENT));
        assert!(flags.contains(PageFlags::WRITABLE));
        assert!(!flags.contains(PageFlags::USER));
    }
}
