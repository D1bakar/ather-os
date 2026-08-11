//! Aether OS kernel library.
//!
//! M0 provides a host-buildable stub so the workspace passes CI. M1 adds the
//! `#![no_std]` entry point, panic handler, and serial-backed logging for
//! `x86_64-unknown-none`.
//!
//! Build for bare metal (M1+):
//!
//! ```text
//! rustup target add x86_64-unknown-none
//! cargo build -p aether-kernel --no-default-features --target x86_64-unknown-none
//! ```

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

/// Placeholder for M1 `kmain` — initializes subsystems and enters the scheduler loop.
///
/// Not called in M0; exists to document the intended entry API.
pub fn kmain_stub() -> ! {
    if is_host_stub() {
        panic!("kmain_stub must not run in production; M1 implements bare-metal kmain");
    }
    // Bare-metal idle loop until M1 implements proper halt/wfi.
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
