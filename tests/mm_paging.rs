//! Host integration tests for M3 paging layout helpers.

use aether_kernel::KERNEL_VIRT_BASE;
use x86_64::VirtAddr;

#[test]
fn higher_half_uses_pml4_index_256() {
    let addr = VirtAddr::new(KERNEL_VIRT_BASE);
    assert_eq!(u16::from(addr.p4_index()), 256);
}

#[test]
fn direct_map_offset_is_canonical() {
    assert_eq!(KERNEL_VIRT_BASE, 0xFFFF_8000_0000_0000);
}
