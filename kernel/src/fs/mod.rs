//! Filesystem backends and block-device stubs.

pub mod block;
pub mod mount;
pub mod ramfs;

pub use block::{BlockDevice, NullBlockDevice};
pub use mount::{init as mount_root, is_mounted as root_mounted, with_root};
pub use ramfs::RamFs;
