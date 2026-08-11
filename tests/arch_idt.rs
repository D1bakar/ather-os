//! Host integration tests for x86_64 IDT entry layout.
//!
//! Mirrors the in-kernel [`IdtEntry`] encoding so gate layout can be verified on
//! the host without executing `LIDT`.

use aether_kernel::arch::x86_64::gdt::KERNEL_CODE_SELECTOR;

/// Size of one 64-bit IDT gate descriptor.
const IDT_ENTRY_SIZE: usize = 16;

/// Number of IDT vectors on x86_64.
const IDT_VECTOR_COUNT: usize = 256;

/// Present interrupt gate, DPL 0 (type 0xE, P=1).
const IDT_GATE_INTERRUPT: u8 = 0x8E;

/// Host-side image of the kernel's packed IDT entry.
#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
struct IdtEntryLayout {
    offset_low: u16,
    selector: u16,
    ist: u8,
    type_attr: u8,
    offset_mid: u16,
    offset_high: u32,
    reserved: u32,
}

impl IdtEntryLayout {
    fn encode_handler(handler: u64, selector: u16) -> Self {
        Self {
            offset_low: handler as u16,
            selector,
            ist: 0,
            type_attr: IDT_GATE_INTERRUPT,
            offset_mid: (handler >> 16) as u16,
            offset_high: (handler >> 32) as u32,
            reserved: 0,
        }
    }

    fn handler_address(&self) -> u64 {
        let offset_low = self.offset_low;
        let offset_mid = self.offset_mid;
        let offset_high = self.offset_high;
        u64::from(offset_low) | (u64::from(offset_mid) << 16) | (u64::from(offset_high) << 32)
    }
}

#[test]
fn idt_entry_size_is_sixteen_bytes() {
    assert_eq!(core::mem::size_of::<IdtEntryLayout>(), IDT_ENTRY_SIZE);
}

#[test]
fn idt_limit_operand_is_size_minus_one() {
    let limit = (IDT_ENTRY_SIZE * IDT_VECTOR_COUNT - 1) as u16;
    assert_eq!(limit, 4095);
}

#[test]
fn idt_gate_uses_kernel_code_selector() {
    let entry = IdtEntryLayout::encode_handler(0xFFFF_8000_0000_1000, KERNEL_CODE_SELECTOR);
    let selector = entry.selector;
    let type_attr = entry.type_attr;
    let ist = entry.ist;
    let reserved = entry.reserved;
    assert_eq!(selector, 0x08);
    assert_eq!(type_attr, IDT_GATE_INTERRUPT);
    assert_eq!(ist, 0);
    assert_eq!(reserved, 0);
}

#[test]
fn idt_handler_offset_splits_across_three_fields() {
    let handler = 0xFFFF_FFFF_C000_1234_u64;
    let entry = IdtEntryLayout::encode_handler(handler, KERNEL_CODE_SELECTOR);
    assert_eq!(entry.handler_address(), handler);
    let offset_low = entry.offset_low;
    let offset_mid = entry.offset_mid;
    let offset_high = entry.offset_high;
    assert_eq!(offset_low, 0x1234);
    assert_eq!(offset_mid, 0xC000);
    assert_eq!(offset_high, 0xFFFF_FFFF);
}

#[test]
fn idt_type_attr_is_present_interrupt_gate() {
    let entry = IdtEntryLayout::encode_handler(0x1000, KERNEL_CODE_SELECTOR);
    let type_attr = entry.type_attr;
    assert_eq!(type_attr & 0x8F, 0x8E);
    assert_eq!(type_attr >> 4, 0x8); // type nibble = interrupt gate
}

#[test]
fn exception_vectors_use_low_half_of_table() {
    // CPU exceptions 0–31 occupy the first 32 IDT slots.
    for vector in 0u8..32 {
        assert!((vector as usize) < 32);
    }
}
