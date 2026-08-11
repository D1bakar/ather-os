//! Host integration tests for ring-3 GDT segment descriptors.

use aether_kernel::arch::x86_64::gdt::layout::{
    user_code_descriptor, user_data_descriptor, USER_CODE_INDEX, USER_CODE_SELECTOR,
    USER_DATA_INDEX, USER_DATA_SELECTOR,
};

#[test]
fn user_segments_are_ring_three() {
    let data = user_data_descriptor();
    let code = user_code_descriptor();
    // DPL bits 45:44 == 11
    assert_eq!((data >> 45) & 0b11, 0b11);
    assert_eq!((code >> 45) & 0b11, 0b11);
}

#[test]
fn user_selectors_match_gdt_indices() {
    assert_eq!(USER_DATA_SELECTOR, ((USER_DATA_INDEX as u16) << 3) | 3);
    assert_eq!(USER_CODE_SELECTOR, ((USER_CODE_INDEX as u16) << 3) | 3);
}

#[test]
fn sysret_selector_spacing() {
    assert_eq!(USER_CODE_SELECTOR.wrapping_sub(USER_DATA_SELECTOR), 8);
}
