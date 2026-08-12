//! Minimal user-space runtime for Aether OS.
//!
//! Provides thin wrappers around [`aether-abi`] syscall numbers. On the host
//! (`host` feature), syscalls are simulated so init/shell binaries can be
//! built and tested without a running kernel.

#![cfg_attr(not(feature = "host"), no_std)]
#![deny(missing_docs)]

mod syscall;

pub use syscall::{
    close, exit, getpid, mmap, munmap, open, read, write, yield_cpu, MmapProt, StdFd, MAP_FAILED,
};

/// Writes a string to standard output (host) or fd 1 (bare metal).
pub fn print(s: &str) {
    let _ = write(StdFd::Stdout.as_i32(), s.as_bytes());
}

/// Writes a string followed by CRLF to standard output.
pub fn println(s: &str) {
    print(s);
    print("\r\n");
}
