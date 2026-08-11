//! Intel 8259 Programmable Interrupt Controller (PIC) remapping.
//!
//! Remaps hardware IRQ lines 0–15 to CPU vectors 32–47 so they do not overlap
//! CPU exception vectors 0–31.

const MASTER_CMD: u16 = 0x20;
const MASTER_DATA: u16 = 0x21;
const SLAVE_CMD: u16 = 0xA0;
const SLAVE_DATA: u16 = 0xA1;

const ICW1_INIT: u8 = 0x11;
const ICW4_8086: u8 = 0x01;

/// First CPU vector assigned to master PIC IRQ 0.
pub const PIC_VECTOR_OFFSET: u8 = 32;

/// Initializes both PICs and remaps IRQ 0–15 to vectors 32–47.
///
/// All IRQ lines are masked on return; [`super::timer::init`] unmasks IRQ 0.
pub fn init() {
    unsafe {
        // Start initialization sequence.
        outb(MASTER_CMD, ICW1_INIT);
        io_wait();
        outb(SLAVE_CMD, ICW1_INIT);
        io_wait();

        // Vector offsets: IRQ 0–7 → 32–39, IRQ 8–15 → 40–47.
        outb(MASTER_DATA, PIC_VECTOR_OFFSET);
        io_wait();
        outb(SLAVE_DATA, PIC_VECTOR_OFFSET + 8);
        io_wait();

        // Tell master about slave at IRQ2.
        outb(MASTER_DATA, 0x04);
        io_wait();
        outb(SLAVE_DATA, 0x02);
        io_wait();

        // 8086 mode.
        outb(MASTER_DATA, ICW4_8086);
        io_wait();
        outb(SLAVE_DATA, ICW4_8086);
        io_wait();

        // Mask all IRQ lines until explicitly enabled.
        outb(MASTER_DATA, 0xFF);
        outb(SLAVE_DATA, 0xFF);
    }
}

/// Unmasks a single IRQ line (0–15).
pub fn unmask(irq: u8) {
    let (port, line) = if irq < 8 { (MASTER_DATA, irq) } else { (SLAVE_DATA, irq - 8) };

    unsafe {
        let mask = inb(port);
        outb(port, mask & !(1 << line));
    }
}

/// Sends End Of Interrupt to the PIC for `irq`.
pub fn send_eoi(irq: u8) {
    unsafe {
        if irq >= 8 {
            outb(SLAVE_CMD, 0x20);
        }
        outb(MASTER_CMD, 0x20);
    }
}

fn io_wait() {
    unsafe {
        outb(0x80, 0);
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
