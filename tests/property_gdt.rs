//! Property tests for x86_64 GDT descriptor encoding.

mod support;

use aether_kernel::arch::x86_64::gdt::layout::{
    kernel_code_descriptor, kernel_data_descriptor, table_limit_bytes, tss_descriptor,
    GDT_ENTRY_COUNT, GDT_ENTRY_SIZE,
};
use aether_kernel::arch::x86_64::gdt::tss_size;
use support::for_each_case;

#[test]
fn table_limit_is_size_minus_one() {
    for count in 1..=GDT_ENTRY_COUNT {
        for entry_size in [8usize, GDT_ENTRY_SIZE] {
            let limit = table_limit_bytes(count, entry_size);
            assert_eq!(limit as usize + 1, count * entry_size);
        }
    }
}

#[test]
fn tss_descriptor_preserves_base_low_bits() {
    for_each_case(512, |rng, _| {
        let base_low = rng.next_u64() & 0xFFFF_FFFF;
        let limit = rng.next_bounded(u64::from(tss_size()) + 1) as u32;
        let (low, high) = tss_descriptor(base_low, limit);
        let reconstructed =
            ((low >> 16) & 0xFFFF) | (((low >> 32) & 0xFF) << 16) | (((low >> 56) & 0xFF) << 24);
        assert_eq!(reconstructed, base_low);
        assert_eq!(high, 0);
    });
}

#[test]
fn kernel_code_and_data_descriptors_are_present() {
    let code = kernel_code_descriptor();
    let data = kernel_data_descriptor();
    assert_eq!(code & (1 << 47), 1 << 47);
    assert_eq!(data & (1 << 47), 1 << 47);
    assert_ne!((code >> 40) & 0xFF, (data >> 40) & 0xFF);
}
