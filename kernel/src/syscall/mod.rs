//! Syscall dispatch and userspace pointer validation.
//!
//! # Entry mechanism (x86_64)
//!
//! **Primary:** `SYSCALL` / `SYSRET` via IA32_STAR, IA32_LSTAR, and IA32_FMASK
//! (see [`crate::arch::x86_64::syscall`]). User code loads the syscall number into
//! `RAX` and invokes `syscall`; the kernel trampoline saves state and calls
//! [`dispatch`].
//!
//! **Fallback (documented, not wired):** Legacy `int 0x80` with the same register
//! layout could be added for bring-up on hardware that lacks `SYSCALL` support.
//! Aether targets 64-bit long mode only; the MSR path is installed at boot.

mod dispatch;
mod validate;

pub use dispatch::dispatch;
pub use validate::{validate_user_cstr, validate_user_slice};

/// Initializes the syscall layer and seeds the bring-up capability table.
pub fn init(kernel_stack_top: u64) {
    crate::cap::with_current_table(|table| {
        use aether_types::{CapabilityRights, ObjectKind};
        let _ = table.grant(ObjectKind::File, CapabilityRights::WRITE);
    });

    init_syscall_msrs(kernel_stack_top);
}

#[cfg(all(not(feature = "host-stub"), target_arch = "x86_64"))]
fn init_syscall_msrs(kernel_stack_top: u64) {
    crate::arch::x86_64::init_syscall(kernel_stack_top);
}

#[cfg(not(all(not(feature = "host-stub"), target_arch = "x86_64")))]
fn init_syscall_msrs(_kernel_stack_top: u64) {}
