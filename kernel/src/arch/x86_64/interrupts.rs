//! IRQ dispatch and timer tick accounting.

use core::sync::atomic::{AtomicU64, Ordering};

use super::idt;
use super::pic;
use super::timer::{self, TIMER_IRQ};

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
    pub fn timer_interrupt_stub();
}

/// Installs the timer IRQ handler in the IDT (call after [`super::idt::init`]).
pub fn register_handlers() {
    idt::set_handler(timer::TIMER_VECTOR as usize, timer_interrupt_stub);
}

/// Initializes the PIC, registers the timer IRQ handler, and starts the PIT.
pub fn init() {
    pic::init();
    register_handlers();
    timer::init();
}

pub fn ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}

pub fn enable_interrupts() {
    unsafe {
        core::arch::asm!("sti", options(nomem, nostack));
    }
}

#[no_mangle]
extern "C" fn timer_handler() {
    let tick = TICKS.fetch_add(1, Ordering::Relaxed) + 1;

    pic::send_eoi(TIMER_IRQ);

    if tick % LOG_INTERVAL_TICKS == 0 {
        crate::serial::write_str("[timer] tick ");
        write_u64(tick);
        crate::serial::write_str("\r\n");
    }

    crate::sched::check_init_watchdog(tick);
    crate::sched::tick_preempt();
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
