//! Aether OS kernel library.
//!
//! Host builds use the `host-stub` feature for CI tests; bare-metal builds
//! disable it and target `x86_64-unknown-none`.

#![cfg_attr(not(feature = "host-stub"), no_std)]
#![deny(missing_docs)]

/// Kernel semantic version string (not yet reported at boot).
pub const KERNEL_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Planned higher-half kernel virtual base (design intent — not mapped in M0).
pub const KERNEL_VIRT_BASE: u64 = 0xFFFF_8000_0000_0000;

/// Returns whether the crate was built with the M0 host stub feature.
pub const fn is_host_stub() -> bool {
    cfg!(feature = "host-stub")
}

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub mod arch;

/// Capability table and enforcement stubs (M5).
pub mod cap;

/// ELF64 parser for user binaries (host-testable).
pub mod elf;

/// Filesystem backends (ramfs, block device stubs).
pub mod fs;

/// Physical / virtual memory manager (M3 scaffold).
pub mod mm;

/// Network stack foundations — host-testable protocol stubs.
pub mod net;

/// Process control block and file descriptor table.
pub mod process;

/// Round-robin scheduler, tasks, and context switching (M4).
pub mod sched;

/// Security audit log and policy helpers (M5).
pub mod security;

/// Syscall dispatch and userspace validation (M5).
pub mod syscall;

/// Virtual filesystem trait and path validation.
pub mod vfs;

#[cfg(not(feature = "host-stub"))]
pub mod serial;

#[cfg(all(not(feature = "host-stub"), target_arch = "x86_64"))]
pub use arch::x86_64::{
    enable_interrupts, init_interrupts, init_pic, init_timer, register_irq_handlers, ticks,
};

#[cfg(target_arch = "x86_64")]
pub use arch::x86_64::switch::{CpuContext, CTX_SIZE};

/// Placeholder for M1 `kmain` — initializes subsystems and enters the scheduler loop.
pub fn kmain_stub() -> ! {
    if is_host_stub() {
        panic!("kmain_stub must not run in production; M1 implements bare-metal kmain");
    }
    #[allow(clippy::empty_loop)]
    loop {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_stub_enabled_in_m0() {
        assert!(is_host_stub());
    }

    #[test]
    fn kernel_version_is_set() {
        assert_eq!(KERNEL_VERSION, env!("CARGO_PKG_VERSION"));
    }
}
