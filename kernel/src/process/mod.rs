//! Process control block types (integration stubs).
//!
//! M4 adds scheduler integration fields; M7 wires the per-process fd table so
//! syscall dispatch can map process fds to VFS handles on mounted filesystems.

use aether_types::{AetherError, AetherResult, ErrorCode, PhysicalAddress};

use crate::cap::CapabilityTable;
use crate::sched::TaskId;
use crate::vfs::{FileDescriptor, OpenFlags, Vfs};

/// Process identifier (stable across threads in one process).
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct ProcessId(pub u32);

impl ProcessId {
    /// Creates a process id from a raw value.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }
}

/// Maximum open file descriptors per process.
pub const MAX_FDS_PER_PROCESS: usize = 32;

/// Maps a process-local fd slot to a VFS backend handle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FdEntry {
    /// Index of the mounted [`Vfs`] instance (future mount table).
    pub mount_id: u32,
    /// Backend-local handle returned by [`Vfs::open`].
    pub vfs_handle: FileDescriptor,
    /// Flags recorded at open time.
    pub flags: OpenFlags,
}

/// Per-process file descriptor table.
#[derive(Debug)]
pub struct FileDescriptorTable {
    slots: [Option<FdEntry>; MAX_FDS_PER_PROCESS],
}

impl Default for FileDescriptorTable {
    fn default() -> Self {
        Self::new()
    }
}

impl FileDescriptorTable {
    /// Returns an empty table (all slots free).
    #[must_use]
    pub fn new() -> Self {
        Self { slots: [const { None }; MAX_FDS_PER_PROCESS] }
    }

    /// Allocates the lowest free slot and stores `entry`.
    pub fn allocate(&mut self, entry: FdEntry) -> AetherResult<FileDescriptor> {
        let slot = self
            .slots
            .iter()
            .position(|fd| fd.is_none())
            .ok_or(AetherError::new(ErrorCode::Busy))?;
        self.slots[slot] = Some(entry);
        Ok(FileDescriptor(slot as u32))
    }

    /// Returns the entry for `fd`, if allocated.
    #[must_use]
    pub fn get(&self, fd: FileDescriptor) -> Option<&FdEntry> {
        self.slots.get(fd.index() as usize).and_then(|slot| slot.as_ref())
    }

    /// Removes and returns the entry for `fd`.
    pub fn close(&mut self, fd: FileDescriptor) -> AetherResult<FdEntry> {
        let slot = fd.index() as usize;
        self.slots
            .get_mut(slot)
            .and_then(|entry| entry.take())
            .ok_or(AetherError::new(ErrorCode::InvalidArgument))
    }

    /// Number of currently open descriptors.
    #[must_use]
    pub fn open_count(&self) -> usize {
        self.slots.iter().filter(|slot| slot.is_some()).count()
    }
}

/// Minimal process control block (M4 scheduler + M5 capability fields).
#[derive(Debug)]
pub struct Process {
    /// Process identifier.
    pub pid: ProcessId,
    /// Physical address of the page-table root (CR3).
    pub page_table_root: PhysicalAddress,
    /// Capability table for this process.
    pub capabilities: CapabilityTable,
    /// Primary thread for this process.
    pub main_thread: TaskId,
    /// Open file descriptors for this process.
    pub fd_table: FileDescriptorTable,
}

impl Process {
    /// Creates a process with an empty fd table and capability set.
    #[must_use]
    pub fn new(pid: ProcessId, page_table_root: PhysicalAddress, main_thread: TaskId) -> Self {
        Self {
            pid,
            page_table_root,
            capabilities: CapabilityTable::new(),
            main_thread,
            fd_table: FileDescriptorTable::new(),
        }
    }

    /// Opens `path` on `vfs` and records the handle in this process's fd table.
    pub fn open<V: Vfs>(
        &mut self,
        vfs: &mut V,
        mount_id: u32,
        path: &str,
        flags: OpenFlags,
    ) -> AetherResult<FileDescriptor> {
        let vfs_handle = vfs.open(path, flags)?;
        self.fd_table.allocate(FdEntry { mount_id, vfs_handle, flags })
    }

    /// Closes a process fd and the underlying VFS handle.
    pub fn close<V: Vfs>(&mut self, vfs: &mut V, fd: FileDescriptor) -> AetherResult<()> {
        let entry = self.fd_table.close(fd)?;
        vfs.close(entry.vfs_handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::RamFs;

    #[test]
    fn fd_table_tracks_allocations() {
        let mut table = FileDescriptorTable::new();
        let fd = table
            .allocate(FdEntry {
                mount_id: 0,
                vfs_handle: FileDescriptor(0),
                flags: OpenFlags::READ,
            })
            .expect("allocate");
        assert_eq!(fd.index(), 0);
        assert_eq!(table.open_count(), 1);
        table.close(fd).expect("close");
        assert_eq!(table.open_count(), 0);
    }

    #[test]
    fn process_open_close_wires_vfs() {
        let mut proc = Process::new(ProcessId::new(1), PhysicalAddress::new(0), TaskId::new(1));
        let mut fs = RamFs::new();
        let flags = OpenFlags::READ | OpenFlags::WRITE | OpenFlags::CREATE;
        let fd = proc.open(&mut fs, 0, "/bin/init", flags).expect("open");
        assert_eq!(proc.fd_table.open_count(), 1);
        proc.close(&mut fs, fd).expect("close");
        assert_eq!(proc.fd_table.open_count(), 0);
    }
}
