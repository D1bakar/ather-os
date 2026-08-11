//! I/O traits for Aether OS.
//!
//! Defines [`Read`] and [`Write`] abstractions shared by kernel drivers and
//! future user-space runtime code.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]

mod error;
mod traits;

pub use error::IoError;
pub use traits::{Read, StrWriter, Write, WriteStr};

#[cfg(test)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;
