//! System call ABI definitions for Aether OS.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod dispatch;
mod numbers;
mod regs;

pub use dispatch::{descriptor_for, lookup_syscall, SyscallDescriptor, SYSCALL_TABLE};
pub use numbers::{syscall_count, SyscallNumber};
pub use regs::SyscallArgs;
