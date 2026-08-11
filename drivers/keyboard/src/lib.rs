//! PS/2 keyboard controller driver (scancode set 1).

#![no_std]
#![deny(missing_docs)]

use aether_drv_hal::{inb, outb};
use core::sync::atomic::{AtomicU32, Ordering};

const PS2_DATA: u16 = 0x60;
const PS2_STATUS: u16 = 0x64;
const PS2_CMD: u16 = 0x64;

/// Hardware IRQ line for the keyboard (8259 IRQ 1).
pub const KEYBOARD_IRQ: u8 = 1;

/// Status register: output buffer full (data available).
const STATUS_OUTPUT_FULL: u8 = 0x01;

static LAST_SCANCODE: AtomicU32 = AtomicU32::new(0);
static KEY_COUNT: AtomicU32 = AtomicU32::new(0);

/// Initializes the PS/2 keyboard: flush buffer and enable scanning.
pub fn init() {
    flush_output();
    // Enable scanning (command 0xF4 to keyboard device).
    write_command(0xD2); // Write to keyboard output buffer
    wait_input_empty();
    write_data(0xF4);
    wait_input_empty();
}

/// Returns `true` when the PS/2 controller has a byte in its output buffer.
pub fn data_available() -> bool {
    // SAFETY: Status port read is always safe.
    unsafe { (inb(PS2_STATUS) & STATUS_OUTPUT_FULL) != 0 }
}

/// Reads one scancode from the PS/2 data port, or `None` if the buffer is empty.
pub fn read_scancode() -> Option<u8> {
    if !data_available() {
        return None;
    }
    // SAFETY: Output buffer was verified full.
    Some(unsafe { inb(PS2_DATA) })
}

/// IRQ handler entry: drains one scancode and records it.
pub fn handle_irq() {
    if let Some(code) = read_scancode() {
        LAST_SCANCODE.store(u32::from(code), Ordering::Relaxed);
        KEY_COUNT.fetch_add(1, Ordering::Relaxed);
    }
}

/// Returns the most recently received scancode.
pub fn last_scancode() -> u8 {
    LAST_SCANCODE.load(Ordering::Relaxed) as u8
}

/// Returns the number of scancodes received since boot.
pub fn key_count() -> u32 {
    KEY_COUNT.load(Ordering::Relaxed)
}

fn flush_output() {
    for _ in 0..32 {
        if !data_available() {
            break;
        }
        // SAFETY: Draining output buffer during init.
        unsafe {
            let _ = inb(PS2_DATA);
        }
    }
}

fn wait_input_empty() {
    for _ in 0..100_000 {
        // SAFETY: Polling status during init.
        if unsafe { (inb(PS2_STATUS) & 0x02) == 0 } {
            return;
        }
        core::hint::spin_loop();
    }
}

fn write_command(value: u8) {
    wait_input_empty();
    // SAFETY: Command port write after input buffer empty.
    unsafe {
        outb(PS2_CMD, value);
    }
}

fn write_data(value: u8) {
    wait_input_empty();
    // SAFETY: Data port write after input buffer empty.
    unsafe {
        outb(PS2_DATA, value);
    }
}
