//! Host integration tests for x86_64 GDT descriptor encoding.
//!
//! Exercises the kernel's host-testable [`aether_kernel::arch::x86_64::gdt::layout`]
//! module so descriptor math is verified outside the kernel crate.

use aether_kernel::arch::x86_64::gdt::layout::{
    kernel_code_descriptor, kernel_data_descriptor, table_limit_bytes, tss_descriptor,
    DescriptorTablePointer, GDT_ENTRY_COUNT, GDT_ENTRY_SIZE, KERNEL_CODE_INDEX,
    KERNEL_CODE_SELECTOR, KERNEL_DATA_INDEX, KERNEL_DATA_SELECTOR, TSS_INDEX, TSS_SELECTOR,
};
use aether_kernel::arch::x86_64::gdt::{tss_size, TaskStateSegment};

#[test]
fn gdt_selector_rpl_is_zero() {
    assert_eq!(KERNEL_CODE_SELECTOR & 0b111, 0);
    assert_eq!(KERNEL_DATA_SELECTOR & 0b111, 0);
    assert_eq!(TSS_SELECTOR & 0b111, 0);
}

#[test]
fn gdt_table_limit_matches_entry_count() {
    assert_eq!(
        table_limit_bytes(GDT_ENTRY_COUNT, GDT_ENTRY_SIZE),
        (GDT_ENTRY_COUNT * GDT_ENTRY_SIZE - 1) as u16
    );
}

#[test]
fn kernel_code_descriptor_is_long_mode_code() {
    let desc = kernel_code_descriptor();
    // Present (bit 47), L=1 (bit 53), type executable/readable (0xA in nibble).
    assert_eq!(desc & (1 << 47), 1 << 47);
    assert_eq!(desc & (1 << 53), 1 << 53);
    assert_eq!((desc >> 40) & 0xFF, 0x9A);
}

#[test]
fn kernel_data_descriptor_is_long_mode_data() {
    let desc = kernel_data_descriptor();
    assert_eq!(desc & (1 << 47), 1 << 47);
    assert_eq!((desc >> 40) & 0xFF, 0x92);
}

#[test]
fn tss_descriptor_splits_base_across_two_qwords() {
    let base = 0xFFFF_8000_0001_0000_u64;
    let limit = tss_size();
    let (low, high) = tss_descriptor(base, limit);

    let base_low =
        ((low >> 16) & 0xFFFF) | (((low >> 32) & 0xFF) << 16) | (((low >> 56) & 0xFF) << 24);
    assert_eq!(base_low, base & 0xFFFF_FFFF);
    assert_eq!(high, base >> 32);
}

#[test]
fn gdt_pointer_struct_is_packed_for_lgdt() {
    assert_eq!(core::mem::size_of::<DescriptorTablePointer>(), 10);
}

#[test]
fn segment_indices_match_public_selectors() {
    assert_eq!(KERNEL_CODE_SELECTOR, (KERNEL_CODE_INDEX as u16) << 3);
    assert_eq!(KERNEL_DATA_SELECTOR, (KERNEL_DATA_INDEX as u16) << 3);
    assert_eq!(TSS_SELECTOR, (TSS_INDEX as u16) << 3);
}

#[test]
fn default_tss_iomap_base_equals_structure_size() {
    let tss = TaskStateSegment::new();
    let iomap_base = tss.iomap_base;
    assert_eq!(iomap_base, core::mem::size_of::<TaskStateSegment>() as u16);
}
