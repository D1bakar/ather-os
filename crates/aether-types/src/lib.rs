//! Shared types used across Aether OS components.
//!
//! This crate provides fundamental address types, error types, and other
//! cross-cutting definitions that the kernel, boot loader, and user-space
//! tooling all depend on.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod address;
mod boot_info;
mod error;
mod page;
mod result;

pub use address::{PhysicalAddress, VirtualAddress};
pub use boot_info::{
    BootInfo, FramebufferInfo, MemoryMapEntry, SerialPortInfo, BOOT_INFO_MAGIC, BOOT_INFO_VERSION,
    MEMORY_TYPE_CONVENTIONAL,
};
pub use error::{AetherError, ErrorCode};
pub use page::{PageFlags, PageSize};
pub use result::{from_error_code, to_error_code, AetherResult};
