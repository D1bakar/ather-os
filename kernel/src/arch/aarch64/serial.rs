//! PL011 UART early console scaffold for QEMU `virt`.
//!
//! **Status:** scaffold — MMIO helpers exist but are not exercised by any shipped
//! boot path or CI job.

/// PL011 UART0 base on QEMU `virt`.
const PL011_BASE: usize = 0x0900_0000;
const UARTDR: *mut u32 = PL011_BASE as *mut u32;
const UARTFR: *const u32 = (PL011_BASE + 0x18) as *const u32;

const FR_TXFF: u32 = 1 << 5;

/// Initializes PL011 (placeholder — baud and line control programmed in a future M13 step).
pub fn init() {
    // Intentionally empty until AArch64 boot validates this path in QEMU.
}

/// Writes a UTF-8 string to the PL011 UART.
pub fn write_str(s: &str) {
    for byte in s.bytes() {
        write_byte(byte);
    }
}

fn write_byte(byte: u8) {
    unsafe {
        while core::ptr::read_volatile(UARTFR) & FR_TXFF != 0 {}
        core::ptr::write_volatile(UARTDR, byte as u32);
    }
}
