//! AArch64 early boot scaffold (M13).
//!
//! **Status:** not linked — no `kernel` binary target for `aarch64-unknown-none` yet.

use super::serial;

/// Architecture-specific setup invoked before the portable kernel main loop.
///
/// Not called by any shipped boot path.
pub fn early_init() {
    serial::init();
}

/// Enters the idle loop using `WFI` (wait for interrupt).
pub fn halt_loop() -> ! {
    loop {
        unsafe {
            core::arch::asm!("wfi", options(nomem, nostack, preserves_flags));
        }
    }
}
