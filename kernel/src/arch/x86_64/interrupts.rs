//! IRQ dispatch and timer tick accounting.
//!
//! # Initialization order
//!
//! Interrupt delivery requires a valid GDT and IDT before any of the steps
//! below. An integration layer must:
//!
//! 1. Initialize the **GDT** (kernel code/data segments).
//! 2. Initialize the **IDT** and register [`timer_interrupt_stub`] at
//!    [`super::timer::TIMER_VECTOR`] (32 after PIC remap) via
//!    `idt::set_handler`.
//! 3. Call [`super::init_pic`] to remap the 8259 PIC.
//! 4. Call [`super::init_timer`] to start the PIT at ~100 Hz.
//! 5. Call [`enable_interrupts`] to execute `STI` and allow IRQ delivery.

use core::sync::atomic::{AtomicU64, Ordering};

use aether_drv_keyboard as keyboard;

use super::idt;
use super::pic;
use super::timer::{self, TIMER_IRQ};

/// CPU vector for the keyboard interrupt after PIC remapping.
const KEYBOARD_VECTOR: u8 = pic::PIC_VECTOR_OFFSET + keyboard::KEYBOARD_IRQ;

/// Serial log interval in ticks (~1 s at 100 Hz).
const LOG_INTERVAL_TICKS: u64 = 100;

static TICKS: AtomicU64 = AtomicU64::new(0);

core::arch::global_asm!(
    ".global timer_interrupt_stub",
    "timer_interrupt_stub:",
    "push rax",
    "push rcx",
    "push rdx",
    "push rsi",
    "push rdi",
    "push r8",
    "push r9",
    "push r10",
    "push r11",
    "call timer_handler",
    "pop r11",
    "pop r10",
    "pop r9",
    "pop r8",
    "pop rdi",
    "pop rsi",
    "pop rdx",
    "pop rcx",
    "pop rax",
    "iretq",
);

extern "sysv64" {
    /// Assembly trampoline for the timer IRQ; register at [`timer::TIMER_VECTOR`].
    pub fn timer_interrupt_stub();
}

core::arch::global_asm!(
    ".global keyboard_interrupt_stub",
    "keyboard_interrupt_stub:",
    "push rax",
    "push rcx",
    "push rdx",
    "push rsi",
    "push rdi",
    "push r8",
    "push r9",
    "push r10",
    "push r11",
    "call keyboard_handler",
    "pop r11",
    "pop r10",
    "pop r9",
    "pop r8",
    "pop rdi",
    "pop rsi",
    "pop rdx",
    "pop rcx",
    "pop rax",
    "iretq",
);

extern "sysv64" {
    /// Assembly trampoline for the keyboard IRQ.
    fn keyboard_interrupt_stub();
}

/// Registers the timer vector handler in the IDT.
///
/// Requires [`super::init_pic`] and [`super::idt::init`] to have run first.
pub fn register_handlers() {
    idt::set_handler(timer::TIMER_VECTOR as usize, timer_interrupt_stub);
}

/// Registers the keyboard IRQ handler and unmasks IRQ 1.
pub fn register_keyboard_handler() {
    idt::set_handler(KEYBOARD_VECTOR as usize, keyboard_interrupt_stub);
    pic::unmask(keyboard::KEYBOARD_IRQ);
}

/// Initializes the PIC, registers the timer IRQ handler, and starts the PIT.
///
/// Requires [`super::gdt::init`] and [`super::idt::init`] to have run first.
#[allow(dead_code)]
pub fn init() {
    super::init_pic();
    register_handlers();
    super::init_timer();
}

/// Returns the number of timer ticks since boot.
pub fn ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}

/// Enables hardware interrupts (`STI`).
///
/// # Safety prerequisites
///
/// The GDT and IDT must already be loaded and [`timer_interrupt_stub`] must be
/// registered at [`timer::TIMER_VECTOR`] before calling this function.
pub fn enable_interrupts() {
    unsafe {
        core::arch::asm!("sti", options(nomem, nostack));
    }
}

#[no_mangle]
extern "C" fn timer_handler() {
    let tick = TICKS.fetch_add(1, Ordering::Relaxed) + 1;

    if tick % LOG_INTERVAL_TICKS == 0 {
        crate::serial::write_str("[timer] tick ");
        write_u64(tick);
        crate::serial::write_str("\r\n");
    }

    pic::send_eoi(TIMER_IRQ);

    crate::sched::tick_preempt();
}

#[no_mangle]
extern "C" fn keyboard_handler() {
    keyboard::handle_irq();
    pic::send_eoi(keyboard::KEYBOARD_IRQ);
}

fn write_u64(mut value: u64) {
    let mut buf = [0u8; 20];
    let mut index = buf.len();

    if value == 0 {
        crate::serial::write_str("0");
        return;
    }

    while value > 0 {
        index -= 1;
        buf[index] = b'0' + (value % 10) as u8;
        value /= 10;
    }

    crate::serial::write_str(core::str::from_utf8(&buf[index..]).unwrap_or("?"));
}
