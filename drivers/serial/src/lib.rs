//! COM1 (16550 UART) serial output for early console and diagnostics.

#![no_std]
#![deny(missing_docs)]

use aether_drv_hal::{inb, outb};
use aether_types::SerialPortInfo;

/// Default COM1 I/O base used by QEMU `-serial stdio`.
pub const COM1_PORT: u16 = 0x3F8;

static mut ACTIVE_PORT: u16 = COM1_PORT;

/// Initializes the UART at `info.port` for 8N1 at 115200 baud (QEMU default).
pub fn init(info: &SerialPortInfo) {
    let port = if info.port != 0 { info.port } else { COM1_PORT };
    // SAFETY: Single-threaded early boot before interrupts.
    unsafe {
        ACTIVE_PORT = port;
        // Disable interrupts.
        outb(port + 1, 0x00);
        // Enable DLAB.
        outb(port + 3, 0x80);
        // Divisor 1 => 115200 baud on QEMU.
        outb(port, 0x01);
        outb(port + 1, 0x00);
        // 8 bits, no parity, one stop bit; clear DLAB.
        outb(port + 3, 0x03);
        // Enable FIFO, clear, 14-byte threshold.
        outb(port + 2, 0xC7);
        // IRQs enabled, RTS/DSR set.
        outb(port + 4, 0x0B);
    }
}

/// Initializes COM1 with default settings.
pub fn init_default() {
    init(&SerialPortInfo::default());
}

/// Writes a UTF-8 string to the active serial port.
pub fn write_str(s: &str) {
    for byte in s.bytes() {
        write_byte(byte);
    }
}

/// Writes a single byte to the active serial port.
pub fn write_byte(byte: u8) {
    // SAFETY: UART is initialized and accessed from the boot CPU only.
    unsafe {
        let port = ACTIVE_PORT;
        while (inb(port + 5) & 0x20) == 0 {}
        outb(port, byte);
    }
}

/// Returns `true` when the UART data register is ready to accept a byte.
pub fn tx_ready() -> bool {
    // SAFETY: Read-only status check on initialized UART.
    unsafe { (inb(ACTIVE_PORT + 5) & 0x20) != 0 }
}
