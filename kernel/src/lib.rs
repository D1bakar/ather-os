//! Aether OS kernel library.

#![cfg_attr(not(feature = "host-stub"), no_std)]
#![deny(missing_docs)]

/// Kernel semantic version string.
pub const KERNEL_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Planned higher-half kernel virtual base.
pub const KERNEL_VIRT_BASE: u64 = 0xFFFF_8000_0000_0000;

/// Returns whether the crate was built with the host stub feature.
pub const fn is_host_stub() -> bool {
    cfg!(feature = "host-stub")
}

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub mod arch;

#[cfg(not(feature = "host-stub"))]
pub mod drivers;

pub mod cap;
pub mod elf;
pub mod fs;
pub mod mm;
pub mod net;
pub mod process;
pub mod sched;
pub mod security;
pub mod syscall;
pub mod vfs;

#[cfg(all(not(feature = "host-stub"), target_arch = "x86_64"))]
pub mod user;

#[cfg(not(feature = "host-stub"))]
pub mod serial;

#[cfg(all(not(feature = "host-stub"), target_arch = "x86_64"))]
pub use arch::x86_64::{
    enable_interrupts, init_interrupts, init_pic, init_timer, register_irq_handlers, ticks,
};

#[cfg(target_arch = "x86_64")]
pub use arch::x86_64::switch::{CpuContext, CTX_SIZE};

/// Re-exported collection types from [ether_collections].
pub use aether_collections::Vec as AetherVec;
/// Re-exported I/O traits from [ether_io].
pub use aether_io::{IoError, Read, StrWriter, Write, WriteStr};
/// Re-exported spin mutex from [ether_sync].
pub use aether_sync::{SpinMutex, SpinMutexGuard};

/// Placeholder kernel entry for host CI.
pub fn kmain_stub() -> ! {
    if is_host_stub() {
        panic!("kmain_stub must not run in production");
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
