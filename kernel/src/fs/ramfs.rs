//! In-memory RAM filesystem for early boot.
//!
//! Uses fixed-capacity tables so the backend works without a kernel heap. Paths
//! are stored as absolute strings (e.g. `/init`, `/etc/config`).

use aether_types::{AetherError, AetherResult, ErrorCode};

use crate::vfs::{FileDescriptor, FileMode, FileStat, OpenFlags, Vfs};

/// Maximum number of distinct paths (files and directories).
pub const MAX_ENTRIES: usize = 32;

/// Maximum bytes per regular file.
pub const MAX_FILE_BYTES: usize = 4096;

/// Maximum number of simultaneously open handles.
pub const MAX_OPEN_HANDLES: usize = 16;

/// Maximum path bytes stored per entry (including leading `/`).
pub const MAX_ENTRY_PATH_LEN: usize = 128;

/// Fixed-size in-memory filesystem for early boot.
#[derive(Debug)]
pub struct RamFs {
    entries: [Option<RamEntry>; MAX_ENTRIES],
    entry_count: usize,
    handles: [Option<OpenHandle>; MAX_OPEN_HANDLES],
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RamEntry {
    path: [u8; MAX_ENTRY_PATH_LEN],
    path_len: usize,
    is_dir: bool,
    data: [u8; MAX_FILE_BYTES],
    data_len: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct OpenHandle {
    entry_index: usize,
    flags: OpenFlags,
}

impl Default for RamFs {
    fn default() -> Self {
        Self::new()
    }
}

impl RamFs {
    /// Returns an empty ramfs with only the root directory `/`.
    #[must_use]
    pub fn new() -> Self {
        let mut fs = Self {
            entries: [const { None }; MAX_ENTRIES],
            entry_count: 0,
            handles: [const { None }; MAX_OPEN_HANDLES],
        };
        let _ = fs.insert_entry("/", true, &[]);
        fs
    }

    /// Creates or overwrites `path` with `data` (helper for bootstrapping).
    pub fn seed_file(&mut self, path: &str, data: &[u8]) -> AetherResult<()> {
        if data.len() > MAX_FILE_BYTES {
            return Err(AetherError::new(ErrorCode::InvalidArgument));
        }
        let index = self.find_or_create_entry(path, false)?;
        let entry = self.entries[index].as_mut().expect("entry exists");
        entry.data[..data.len()].copy_from_slice(data);
        entry.data_len = data.len();
        Ok(())
    }

    fn find_or_create_entry(&mut self, path: &str, is_dir: bool) -> AetherResult<usize> {
        if let Some(index) = self.find_entry(path) {
            return Ok(index);
        }
        self.insert_entry(path, is_dir, &[])
    }

    fn insert_entry(&mut self, path: &str, is_dir: bool, data: &[u8]) -> AetherResult<usize> {
        if self.entry_count >= MAX_ENTRIES {
            return Err(AetherError::new(ErrorCode::OutOfMemory));
        }
        if path.len() > MAX_ENTRY_PATH_LEN {
            return Err(AetherError::new(ErrorCode::InvalidArgument));
        }
        if !is_dir && data.len() > MAX_FILE_BYTES {
            return Err(AetherError::new(ErrorCode::InvalidArgument));
        }

        let slot = self
            .entries
            .iter()
            .position(|entry| entry.is_none())
            .ok_or(AetherError::new(ErrorCode::OutOfMemory))?;

        let mut path_buf = [0u8; MAX_ENTRY_PATH_LEN];
        path_buf[..path.len()].copy_from_slice(path.as_bytes());

        let mut file_data = [0u8; MAX_FILE_BYTES];
        if !is_dir {
            file_data[..data.len()].copy_from_slice(data);
        }

        self.entries[slot] = Some(RamEntry {
            path: path_buf,
            path_len: path.len(),
            is_dir,
            data: file_data,
            data_len: if is_dir { 0 } else { data.len() },
        });
        self.entry_count += 1;
        Ok(slot)
    }

    fn find_entry(&self, path: &str) -> Option<usize> {
        self.entries.iter().enumerate().find_map(|(index, entry)| {
            entry.as_ref().and_then(
                |entry| {
                    if entry_path(entry) == path {
                        Some(index)
                    } else {
                        None
                    }
                },
            )
        })
    }

    fn alloc_handle(
        &mut self,
        entry_index: usize,
        flags: OpenFlags,
    ) -> AetherResult<FileDescriptor> {
        let slot = self
            .handles
            .iter()
            .position(|handle| handle.is_none())
            .ok_or(AetherError::new(ErrorCode::Busy))?;
        self.handles[slot] = Some(OpenHandle { entry_index, flags });
        Ok(FileDescriptor(slot as u32))
    }

    fn handle(&self, fd: FileDescriptor) -> AetherResult<&OpenHandle> {
        self.handles
            .get(fd.index() as usize)
            .and_then(|slot| slot.as_ref())
            .ok_or(AetherError::new(ErrorCode::InvalidArgument))
    }

    fn entry(&self, index: usize) -> AetherResult<&RamEntry> {
        self.entries
            .get(index)
            .and_then(|slot| slot.as_ref())
            .ok_or(AetherError::new(ErrorCode::Internal))
    }

    fn entry_mut(&mut self, index: usize) -> AetherResult<&mut RamEntry> {
        self.entries
            .get_mut(index)
            .and_then(|slot| slot.as_mut())
            .ok_or(AetherError::new(ErrorCode::Internal))
    }
}

impl Vfs for RamFs {
    fn open(&mut self, path: &str, flags: OpenFlags) -> AetherResult<FileDescriptor> {
        let entry_index = match self.find_entry(path) {
            Some(index) => {
                let entry = self.entry(index)?;
                if entry.is_dir {
                    return Err(AetherError::new(ErrorCode::InvalidArgument));
                }
                if flags.contains(OpenFlags::TRUNCATE) && flags.contains(OpenFlags::WRITE) {
                    let entry = self.entry_mut(index)?;
                    entry.data_len = 0;
                }
                index
            }
            None => {
                if !flags.contains(OpenFlags::CREATE) {
                    return Err(AetherError::new(ErrorCode::NotFound));
                }
                self.insert_entry(path, false, &[])?
            }
        };

        self.alloc_handle(entry_index, flags)
    }

    fn read(&mut self, fd: FileDescriptor, buf: &mut [u8], offset: u64) -> AetherResult<usize> {
        let handle = self.handle(fd)?;
        if !handle.flags.contains(OpenFlags::READ) {
            return Err(AetherError::new(ErrorCode::PermissionDenied));
        }
        let entry = self.entry(handle.entry_index)?;
        if entry.is_dir {
            return Err(AetherError::new(ErrorCode::InvalidArgument));
        }

        let offset =
            usize::try_from(offset).map_err(|_| AetherError::new(ErrorCode::InvalidArgument))?;
        if offset >= entry.data_len {
            return Ok(0);
        }
        let available = entry.data_len - offset;
        let count = available.min(buf.len());
        buf[..count].copy_from_slice(&entry.data[offset..offset + count]);
        Ok(count)
    }

    fn write(&mut self, fd: FileDescriptor, buf: &[u8], offset: u64) -> AetherResult<usize> {
        let handle = self.handle(fd)?;
        if !handle.flags.contains(OpenFlags::WRITE) {
            return Err(AetherError::new(ErrorCode::PermissionDenied));
        }
        let entry_index = handle.entry_index;
        let entry = self.entry_mut(entry_index)?;
        if entry.is_dir {
            return Err(AetherError::new(ErrorCode::InvalidArgument));
        }

        let offset =
            usize::try_from(offset).map_err(|_| AetherError::new(ErrorCode::InvalidArgument))?;
        let end = offset.saturating_add(buf.len());
        if end > MAX_FILE_BYTES {
            return Err(AetherError::new(ErrorCode::InvalidArgument));
        }
        entry.data[offset..end].copy_from_slice(buf);
        if end > entry.data_len {
            entry.data_len = end;
        }
        Ok(buf.len())
    }

    fn close(&mut self, fd: FileDescriptor) -> AetherResult<()> {
        let index = fd.index() as usize;
        if self.handles.get(index).and_then(|slot| slot.as_ref()).is_some() {
            self.handles[index] = None;
            Ok(())
        } else {
            Err(AetherError::new(ErrorCode::InvalidArgument))
        }
    }

    fn stat(&self, path: &str) -> AetherResult<FileStat> {
        let index = self.find_entry(path).ok_or(AetherError::new(ErrorCode::NotFound))?;
        let entry = self.entry(index)?;
        Ok(FileStat {
            size: entry.data_len as u64,
            mode: if entry.is_dir { FileMode::DIR } else { FileMode::FILE },
            is_dir: entry.is_dir,
        })
    }
}

fn entry_path(entry: &RamEntry) -> &str {
    // SAFETY: paths are inserted from validated UTF-8 str slices only.
    core::str::from_utf8(&entry.path[..entry.path_len]).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vfs::open_with_validation;
    use crate::vfs::path::{AccessMode, Credentials};

    #[test]
    fn new_ramfs_has_root_directory() {
        let fs = RamFs::new();
        let stat = fs.stat("/").expect("root stat");
        assert!(stat.is_dir);
        assert_eq!(stat.mode, FileMode::DIR);
    }

    #[test]
    fn create_read_write_close_roundtrip() {
        let mut fs = RamFs::new();
        let creds = Credentials::kernel();
        let flags = OpenFlags::READ | OpenFlags::WRITE | OpenFlags::CREATE;
        let fd = open_with_validation(&mut fs, "/hello", flags, &creds, AccessMode::Write)
            .expect("open");

        let payload = b"aether";
        assert_eq!(fs.write(fd, payload, 0).expect("write"), payload.len());

        let mut buf = [0u8; 8];
        assert_eq!(fs.read(fd, &mut buf, 0).expect("read"), payload.len());
        assert_eq!(&buf[..payload.len()], payload);

        fs.close(fd).expect("close");
        assert!(fs.close(fd).is_err());
    }

    #[test]
    fn stat_reports_file_size() {
        let mut fs = RamFs::new();
        fs.seed_file("/init", b"boot").expect("seed");
        let stat = fs.stat("/init").expect("stat");
        assert!(!stat.is_dir);
        assert_eq!(stat.size, 4);
    }

    #[test]
    fn open_missing_without_create_fails() {
        let mut fs = RamFs::new();
        let err = fs.open("/missing", OpenFlags::READ).expect_err("not found");
        assert_eq!(err.code, ErrorCode::NotFound);
    }

    #[test]
    fn truncate_on_open_clears_contents() {
        let mut fs = RamFs::new();
        fs.seed_file("/log", b"old-data").expect("seed");
        let fd = fs
            .open("/log", OpenFlags::READ | OpenFlags::WRITE | OpenFlags::TRUNCATE)
            .expect("open truncate");
        let stat = fs.stat("/log").expect("stat");
        assert_eq!(stat.size, 0);
        fs.close(fd).expect("close");
    }

    #[test]
    fn read_requires_read_flag() {
        let mut fs = RamFs::new();
        fs.seed_file("/ro", b"x").expect("seed");
        let fd = fs.open("/ro", OpenFlags::WRITE).expect("open write-only");
        let mut buf = [0u8; 1];
        assert_eq!(fs.read(fd, &mut buf, 0).expect_err("denied").code, ErrorCode::PermissionDenied);
    }

    #[test]
    fn seed_and_stat_size_matches_payload_property() {
        let paths = ["/file", "/init", "/etc/cfg"];
        let payloads: [&[u8]; 4] = [b"", b"x", b"boot", b"0123456789abcdef"];
        for path in paths {
            for payload in payloads {
                let mut fs = RamFs::new();
                if fs.seed_file(path, payload).is_ok() {
                    let stat = fs.stat(path).expect("stat");
                    assert_eq!(stat.size, payload.len() as u64);
                }
            }
        }
    }
}
