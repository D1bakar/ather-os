//! 256-entry Interrupt Descriptor Table (IDT) for x86_64.

use super::exceptions::{exception_stub_addr, interrupt_stub_addr};
use super::gdt::KERNEL_CODE_SELECTOR;
use core::mem::size_of;

/// Present interrupt gate, DPL 0 (type 0xE, P=1).
const IDT_GATE_INTERRUPT: u8 = 0x8E;

/// One IDT gate descriptor (64-bit interrupt/trap gate).
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    type_attr: u8,
    offset_mid: u16,
    offset_high: u32,
    reserved: u32,
}

impl IdtEntry {
    const fn missing() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            ist: 0,
            type_attr: 0,
            offset_mid: 0,
            offset_high: 0,
            reserved: 0,
        }
    }

    fn set_handler(&mut self, handler: u64) {
        let handler = crate::mm::link_to_direct_virt(handler);
        self.offset_low = handler as u16;
        self.selector = KERNEL_CODE_SELECTOR;
        self.ist = 0;
        self.type_attr = IDT_GATE_INTERRUPT;
        self.offset_mid = (handler >> 16) as u16;
        self.offset_high = (handler >> 32) as u32;
        self.reserved = 0;
    }
}

/// IDT register image loaded with `LIDT`.
#[repr(C, packed)]
struct Idtr {
    limit: u16,
    base: u64,
}

const IDT_LEN: usize = 256;

static mut IDT: [IdtEntry; IDT_LEN] = [IdtEntry::missing(); IDT_LEN];

/// Builds the IDT and loads it with `LIDT`.
///
/// Must be called after the GDT is loaded so `KERNEL_CODE_SELECTOR` (0x08) is valid.
pub fn init() {
    // SAFETY: Single-threaded early boot; no concurrent IDT access.
    unsafe {
        for vector in 0u8..32 {
            let handler = exception_stub_addr(vector);
            IDT[vector as usize].set_handler(handler);
        }

        for vector in 32u8..=255 {
            let handler = interrupt_stub_addr(vector);
            IDT[vector as usize].set_handler(handler);
        }

        let idt_base = crate::mm::link_to_direct_virt(core::ptr::addr_of!(IDT) as u64);
        let idtr = Idtr { limit: (size_of::<IdtEntry>() * IDT_LEN - 1) as u16, base: idt_base };

        core::arch::asm!(
            "lidt [{0}]",
            in(reg) &idtr,
            options(nomem, nostack)
        );
    }
}

/// Replaces the handler for a single IDT vector (used by IRQ subsystems).
pub fn set_handler(vector: usize, handler: unsafe extern "sysv64" fn()) {
    // SAFETY: Single-threaded early boot; caller must not override a vector concurrently.
    unsafe {
        IDT[vector].set_handler(handler as u64);
    }
}
