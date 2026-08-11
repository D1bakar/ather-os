//! Syscall dispatch and userspace pointer validation.

mod dispatch;
mod validate;

pub use dispatch::dispatch;
pub use validate::{validate_user_cstr, validate_user_slice};

/// Initializes the syscall layer (MSR entry is wired from `arch::x86_64::syscall`).
pub fn init(_kernel_stack_top: u64) {
    #[cfg(all(not(feature = "host-stub"), target_arch = "x86_64"))]
    crate::arch::x86_64::init_syscall(_kernel_stack_top);
}
