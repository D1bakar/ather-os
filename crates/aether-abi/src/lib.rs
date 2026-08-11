//! System call ABI definitions for Aether OS.
//!
//! This crate defines the stable interface between user-space and the kernel.
//! Syscall numbers, calling conventions, and argument layouts are specified
//! here so that libc, the kernel, and tooling share a single source of truth.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod numbers;
mod regs;

pub use numbers::{syscall_count, SyscallNumber};
pub use regs::SyscallArgs;
