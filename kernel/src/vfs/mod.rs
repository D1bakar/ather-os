//! Virtual filesystem layer.
//!
//! Provides a uniform [`Vfs`] trait for pluggable filesystem backends (ramfs,
//! future tmpfs/devfs) and shared types for file metadata and open flags.

pub mod path;

use aether_types::{AetherError, AetherResult, ErrorCode};

pub use path::{check_permission, validate_path, AccessMode, Credentials};

/// Maximum path length accepted by the VFS layer.
pub const MAX_PATH_LEN: usize = 256;

/// Process-visible file descriptor (index into a per-process table).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FileDescriptor(pub u32);

impl FileDescriptor {
    /// Returns the raw index value.
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }
}

/// Flags supplied to [`Vfs::open`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OpenFlags(u32);

impl OpenFlags {
    /// Open for read.
    pub const READ: Self = Self(1 << 0);
    /// Open for write.
    pub const WRITE: Self = Self(1 << 1);
    /// Create the file if it does not exist.
    pub const CREATE: Self = Self(1 << 2);
    /// Truncate existing file to zero length on open.
    pub const TRUNCATE: Self = Self(1 << 3);

    /// Returns whether `flag` is set.
    #[must_use]
    pub const fn contains(self, flag: Self) -> bool {
        (self.0 & flag.0) == flag.0
    }

    /// Returns the raw bitfield.
    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Combines `self` with `other` flag bits.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl core::ops::BitOr for OpenFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitOrAssign for OpenFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// POSIX-style file mode bits (subset used by early boot).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileMode(pub u32);

impl FileMode {
    /// Regular file (`0644`).
    pub const FILE: Self = Self(0o100_644);
    /// Directory (`0755`).
    pub const DIR: Self = Self(0o040_755);

    /// Returns the raw mode value.
    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }
}

/// Metadata returned by [`Vfs::stat`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileStat {
    /// Size in bytes (zero for empty files and directories).
    pub size: u64,
    /// File type and permission bits.
    pub mode: FileMode,
    /// `true` when the path refers to a directory.
    pub is_dir: bool,
}

/// Backend-neutral virtual filesystem interface.
///
/// Implementations manage internal handle tables; the process layer maps
/// per-process file descriptors to handles via [`crate::process::FileDescriptorTable`].
pub trait Vfs {
    /// Opens `path` with `flags`, returning a VFS-local handle.
    fn open(&mut self, path: &str, flags: OpenFlags) -> AetherResult<FileDescriptor>;

    /// Reads up to `buf.len()` bytes from `fd` starting at `offset`.
    fn read(&mut self, fd: FileDescriptor, buf: &mut [u8], offset: u64) -> AetherResult<usize>;

    /// Writes `buf` to `fd` starting at `offset`.
    fn write(&mut self, fd: FileDescriptor, buf: &[u8], offset: u64) -> AetherResult<usize>;

    /// Releases resources associated with `fd`.
    fn close(&mut self, fd: FileDescriptor) -> AetherResult<()>;

    /// Returns metadata for `path` without opening it.
    fn stat(&self, path: &str) -> AetherResult<FileStat>;
}

/// Validates `path` and `flags`, then delegates to `vfs.open`.
pub fn open_with_validation<V: Vfs>(
    vfs: &mut V,
    path: &str,
    flags: OpenFlags,
    creds: &Credentials,
    access: AccessMode,
) -> AetherResult<FileDescriptor> {
    validate_path(path)?;
    check_permission(path, creds, access)?;
    if !flags.contains(OpenFlags::READ) && !flags.contains(OpenFlags::WRITE) {
        return Err(AetherError::new(ErrorCode::InvalidArgument));
    }
    vfs.open(path, flags)
}

/// Validates `path`, then delegates to `vfs.stat`.
pub fn stat_with_validation<V: Vfs>(
    vfs: &V,
    path: &str,
    creds: &Credentials,
    access: AccessMode,
) -> AetherResult<FileStat> {
    validate_path(path)?;
    check_permission(path, creds, access)?;
    vfs.stat(path)
}
