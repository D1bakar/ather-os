//! Physical memory, paging, and kernel heap (M3).
//!
//! Initialization order:
//! 1. [`frame`] — bitmap over conventional RAM from [`BootInfo`]
//! 2. [`paging`] — identity + higher-half direct map, W^X kernel sections
//! 3. [`heap`] — linked-list allocator over mapped heap pages

#[cfg(not(feature = "host-stub"))]
use aether_types::BootInfo;

pub mod frame;

#[cfg(not(feature = "host-stub"))]
pub mod heap;

#[cfg(not(feature = "host-stub"))]
pub mod paging;

#[cfg(not(feature = "host-stub"))]
pub mod user;

/// Direct-map offset: `virt = phys + KERNEL_DIRECT_MAP_BASE`.
pub const KERNEL_DIRECT_MAP_BASE: u64 = crate::KERNEL_VIRT_BASE;

/// Virtual base of the kernel heap region (16 MiB into the direct map).
pub const HEAP_VIRTUAL_START: u64 = KERNEL_DIRECT_MAP_BASE + 0x0100_0000;

/// Initial heap size (1 MiB).
pub const HEAP_SIZE: usize = 1024 * 1024;

/// Initializes frame allocator, page tables, and kernel heap.
#[cfg(not(feature = "host-stub"))]
pub fn init(boot_info: &BootInfo) {
    frame::init(boot_info);
    paging::init();
    heap::init();
    crate::serial::write_str("Aether OS M3: memory initialized\r\n");
}

/// Translates a physical address to the higher-half direct map.
#[must_use]
pub const fn phys_to_virt(phys: u64) -> u64 {
    phys + KERNEL_DIRECT_MAP_BASE
}

/// Converts a link-time kernel address to its higher-half direct-map alias.
///
/// The kernel is linked in the identity map (`linker.ld`); user CR3 shares only
/// the higher-half PML4 entry, so SYSCALL/IDT targets must use this alias.
#[must_use]
pub const fn link_to_direct_virt(link_addr: u64) -> u64 {
    phys_to_virt(link_addr)
}

/// Translates a higher-half direct-map address to physical.
#[must_use]
pub const fn virt_to_phys(virt: u64) -> u64 {
    virt - KERNEL_DIRECT_MAP_BASE
}
