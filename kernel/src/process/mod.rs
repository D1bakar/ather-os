//! Process control block types and registry (M4/M6).

use aether_types::{AetherError, AetherResult, ErrorCode, PhysicalAddress};

use crate::cap::CapabilityTable;
use crate::sched::TaskId;
use crate::vfs::{FileDescriptor, OpenFlags, Vfs};
use aether_types::{CapabilityRights, ObjectKind};

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

/// Maximum live processes tracked by the bring-up registry.
pub const MAX_PROCESSES: usize = 8;

/// Maximum open file descriptors per process.
pub const MAX_FDS_PER_PROCESS: usize = 32;

/// Root mount id for the ramfs instance mounted at `/`.
pub const ROOT_MOUNT_ID: u32 = 0;

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

    /// Reads from an open process fd via the root ramfs mount.
    pub fn read(
        &self,
        vfs: &mut impl Vfs,
        fd: FileDescriptor,
        buf: &mut [u8],
    ) -> AetherResult<usize> {
        let entry = self.fd_table.get(fd).ok_or(AetherError::new(ErrorCode::InvalidArgument))?;
        if !entry.flags.contains(OpenFlags::READ) {
            return Err(AetherError::new(ErrorCode::PermissionDenied));
        }
        vfs.read(entry.vfs_handle, buf, 0)
    }

    /// Grants default I/O capabilities for the first user process.
    pub fn grant_default_io_caps(&mut self) {
        let _ = self.capabilities.grant(ObjectKind::File, CapabilityRights::READ);
        let _ = self.capabilities.grant(ObjectKind::File, CapabilityRights::WRITE);
    }

    /// Grants every object kind with all rights (init pid 1 bring-up).
    pub fn grant_all_caps(&mut self) {
        let all = CapabilityRights::READ
            .union(CapabilityRights::WRITE)
            .union(CapabilityRights::MAP)
            .union(CapabilityRights::EXECUTE)
            .union(CapabilityRights::DELEGATE)
            .union(CapabilityRights::DESTROY);
        for kind in [ObjectKind::File, ObjectKind::Device, ObjectKind::Memory, ObjectKind::Process]
        {
            let _ = self.capabilities.grant(kind, all);
        }
    }

    /// Closes a process fd and the underlying VFS handle.
    pub fn close<V: Vfs>(&mut self, vfs: &mut V, fd: FileDescriptor) -> AetherResult<()> {
        let entry = self.fd_table.close(fd)?;
        vfs.close(entry.vfs_handle)
    }
}

static mut PROCESS_TABLE: [Option<Process>; MAX_PROCESSES] = [const { None }; MAX_PROCESSES];

/// Registers `process` and returns its pid slot index.
pub fn register(process: Process) -> ProcessId {
    let pid = process.pid;
    // SAFETY: BSP-only until a real process allocator exists.
    unsafe {
        let table = &mut *core::ptr::addr_of_mut!(PROCESS_TABLE);
        for slot in table.iter_mut() {
            if slot.is_none() {
                *slot = Some(process);
                return pid;
            }
        }
    }
    pid
}

/// Returns a shared reference to the process with `pid`.
#[must_use]
pub fn get(pid: ProcessId) -> Option<&'static Process> {
    // SAFETY: Read-only lookup; callers must not mutate concurrently.
    unsafe {
        let table = &*core::ptr::addr_of!(PROCESS_TABLE);
        table.iter().find_map(|slot| {
            slot.as_ref().and_then(|proc| if proc.pid == pid { Some(proc) } else { None })
        })
    }
}

/// Runs `f` with mutable access to the process with `pid`.
pub fn with_process<R>(pid: ProcessId, f: impl FnOnce(&mut Process) -> R) -> Option<R> {
    // SAFETY: Exclusive access coordinated by syscall/scheduler context.
    unsafe {
        let table = &mut *core::ptr::addr_of_mut!(PROCESS_TABLE);
        for slot in table.iter_mut() {
            if let Some(proc) = slot.as_mut() {
                if proc.pid == pid {
                    return Some(f(proc));
                }
            }
        }
    }
    None
}

/// Runs `f` with mutable access to the current process, if any.
pub fn with_current<R>(f: impl FnOnce(&mut Process) -> R) -> Option<R> {
    let pid = crate::sched::current_process_id()?;
    with_process(pid, f)
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
