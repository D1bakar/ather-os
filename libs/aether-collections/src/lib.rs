//! Collection stubs for Aether OS.
//!
//! Provides a minimal [`Vec`] wrapper around `alloc::vec::Vec` for kernel and
//! driver code that will eventually grow into a full collection library.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]

extern crate alloc;

mod vec;

pub use vec::Vec;

#[cfg(feature = "std")]
extern crate std;
