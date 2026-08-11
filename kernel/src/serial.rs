//! COM1 (0x3F8) early serial output for QEMU `-serial stdio`.

const COM1: u16 = 0x3F8;

/// Initializes COM1 for 8N1 at 115200 baud (QEMU default).
pub fn init() {
    unsafe {
        // Disable interrupts.
        outb(COM1 + 1, 0x00);
        // Enable DLAB.
        outb(COM1 + 3, 0x80);
        // Divisor 1 => 115200 baud on QEMU.
        outb(COM1 + 0, 0x01);
        outb(COM1 + 1, 0x00);
        // 8 bits, no parity, one stop bit; clear DLAB.
        outb(COM1 + 3, 0x03);
        // Enable FIFO, clear, 14-byte threshold.
        outb(COM1 + 2, 0xC7);
        // IRQs enabled, RTS/DSR set.
        outb(COM1 + 4, 0x0B);
    }
}

/// Writes a UTF-8 string to COM1.
pub fn write_str(s: &str) {
    for byte in s.bytes() {
        write_byte(byte);
    }
}

fn write_byte(byte: u8) {
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
