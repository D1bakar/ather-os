//! Filesystem backends and block-device stubs.

pub mod block;
pub mod ramfs;

pub use block::{BlockDevice, NullBlockDevice};
pub use ramfs::RamFs;
