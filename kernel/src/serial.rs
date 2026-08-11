//! COM1 (0x3F8) early serial output for QEMU `-serial stdio`.

use aether_io::{IoError, WriteStr};
use aether_sync::SpinMutex;

const COM1: u16 = 0x3F8;

static SERIAL: SpinMutex<()> = SpinMutex::new(());

/// Initializes COM1 for 8N1 at 115200 baud (QEMU default).
pub fn init() {
    unsafe {
        outb(COM1 + 1, 0x00);
        outb(COM1 + 3, 0x80);
        outb(COM1 + 0, 0x01);
        outb(COM1 + 1, 0x00);
        outb(COM1 + 3, 0x03);
        outb(COM1 + 2, 0xC7);
        outb(COM1 + 4, 0x0B);
    }
}

/// Writes a UTF-8 string to COM1.
pub fn write_str(s: &str) {
    let _guard = SERIAL.lock();
    for byte in s.bytes() {
        write_byte_unlocked(byte);
    }
}

/// Serial port writer implementing [`WriteStr`].
pub struct SerialWriter;

impl WriteStr for SerialWriter {
    fn write_str(&mut self, s: &str) -> Result<(), IoError> {
        write_str(s);
        Ok(())
    }
}

/// Writes a single byte to COM1.
pub fn write_byte(byte: u8) {
    let _guard = SERIAL.lock();
    write_byte_unlocked(byte);
}

fn write_byte_unlocked(byte: u8) {
    unsafe {
        while (inb(COM1 + 5) & 0x20) == 0 {}
        outb(COM1, byte);
    }
}

unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!(
        "in al, dx",
        out("al") value,
        in("dx") port,
        options(nomem, nostack, preserves_flags)
    );
    value
}

unsafe fn outb(port: u16, value: u8) {
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nomem, nostack, preserves_flags)
    );
}
