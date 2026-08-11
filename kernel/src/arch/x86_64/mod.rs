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
mod syscall;

pub mod switch;

#[cfg(not(feature = "host-stub"))]
mod ports;

#[cfg(not(feature = "host-stub"))]
mod timer;

/// Registers the timer IRQ handler in the IDT (after [`init_pic`]).
#[cfg(not(feature = "host-stub"))]
pub fn register_irq_handlers() {
    interrupts::register_handlers();
}

/// Registers the PS/2 keyboard IRQ handler and unmasks IRQ 1.
#[cfg(not(feature = "host-stub"))]
pub fn register_keyboard_handler() {
    interrupts::register_keyboard_handler();
}

/// Initializes the `SYSCALL` MSR and STAR/GS base for ring-3 entry.
#[cfg(not(feature = "host-stub"))]
pub fn init_syscall(kernel_stack_top: u64) {
    syscall::init(kernel_stack_top);
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

/// Initializes PIC, IRQ handlers, and PIT in one call (library convenience).
#[cfg(not(feature = "host-stub"))]
#[allow(dead_code)]
pub fn init_interrupts() {
    init_pic();
    register_irq_handlers();
    init_timer();
}

/// Enables hardware interrupts (`STI`).
#[cfg(not(feature = "host-stub"))]
pub fn enable_interrupts() {
    interrupts::enable_interrupts();
}

/// Returns the number of timer ticks since boot.
#[cfg(not(feature = "host-stub"))]
#[allow(dead_code)]
pub fn ticks() -> u64 {
    interrupts::ticks()
}
