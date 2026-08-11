//! Synchronization primitives for Aether OS.
//!
//! Provides a [`SpinMutex`] for short critical sections in `#![no_std]` contexts
//! where blocking sleep is unavailable (early boot, interrupt handlers).

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]

mod mutex;

pub use mutex::{SpinMutex, SpinMutexGuard};

#[cfg(feature = "std")]
extern crate std;
