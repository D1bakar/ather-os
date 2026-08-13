//! x86_64 CPU bring-up: GDT (M2), IDT, and exception handling.

pub mod gdt;

#[cfg(not(feature = "host-stub"))]
mod exceptions;

#[cfg(not(feature = "host-stub"))]
pub mod idt;

#[cfg(not(feature = "host-stub"))]
mod interrupts;

#[cfg(not(feature = "host-stub"))]
mod pic;

#[cfg(not(feature = "host-stub"))]
mod ports;

#[cfg(not(feature = "host-stub"))]
mod timer;

pub mod switch;

#[cfg(not(feature = "host-stub"))]
mod syscall;

#[cfg(not(feature = "host-stub"))]
pub mod user_entry;

#[cfg(not(feature = "host-stub"))]
pub use user_entry::enter_user_mode;

/// Initializes PIC remapping, timer IRQ handler, and PIT (~100 Hz).
///
/// Requires [`gdt::init`] and [`idt::init`] to have run first.
#[cfg(not(feature = "host-stub"))]
pub fn init_interrupts() {
    interrupts::init();
}

/// Registers device IRQ handlers in the IDT (requires [`idt::init`] first).
#[cfg(not(feature = "host-stub"))]
pub fn register_irq_handlers() {
    interrupts::register_handlers();
}

/// Remaps the 8259 PIC so hardware IRQ 0–15 map to CPU vectors 32–47.
#[cfg(not(feature = "host-stub"))]
pub fn init_pic() {
    pic::init();
}

/// Starts the PIT at ~100 Hz and unmasks timer IRQ 0.
#[cfg(not(feature = "host-stub"))]
pub fn init_timer() {
    timer::init();
}

/// Enables hardware interrupts (`STI`).
#[cfg(not(feature = "host-stub"))]
pub fn enable_interrupts() {
    interrupts::enable_interrupts();
}

/// Returns the number of timer ticks since boot.
#[cfg(not(feature = "host-stub"))]
pub fn ticks() -> u64 {
    interrupts::ticks()
}

#[cfg(not(feature = "host-stub"))]
pub use pic::PIC_VECTOR_OFFSET;

#[cfg(not(feature = "host-stub"))]
pub use timer::{EFFECTIVE_TIMER_HZ, PIT_BASE_HZ, TIMER_HZ, TIMER_IRQ, TIMER_VECTOR};

/// Installs SYSCALL MSRs and the kernel entry stub (M5).
#[cfg(not(feature = "host-stub"))]
pub fn init_syscall(kernel_stack_top: u64) {
    syscall::init(kernel_stack_top);
}

/// Points the SYSCALL entry stub at `stack_top` for the current user task.
#[cfg(not(feature = "host-stub"))]
pub fn set_syscall_handler_stack(stack_top: u64) {
    syscall::set_handler_stack(stack_top);
}
