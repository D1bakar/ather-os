//! Host integration tests for ramfs and VFS path validation.

use aether_kernel::fs::RamFs;
use aether_kernel::process::{Process, ProcessId};
use aether_kernel::sched::TaskId;
use aether_kernel::vfs::path::{validate_path, AccessMode, Credentials};
use aether_kernel::vfs::{open_with_validation, stat_with_validation, OpenFlags, Vfs};
use aether_types::PhysicalAddress;

#[test]
fn ramfs_seed_and_stat_via_validation_helpers() {
    let mut fs = RamFs::new();
    fs.seed_file("/etc/hostname", b"aether-qemu").expect("seed file");
    let creds = Credentials::kernel();
    let stat = stat_with_validation(&fs, "/etc/hostname", &creds, AccessMode::Read).expect("stat");
    assert_eq!(stat.size, 11);
}

#[test]
fn ramfs_integration_open_read_close() {
    let mut fs = RamFs::new();
    let mut proc = Process::new(ProcessId::new(42), PhysicalAddress::new(0), TaskId::new(1));
    let flags = OpenFlags::READ | OpenFlags::WRITE | OpenFlags::CREATE;
    let fd = proc.open(&mut fs, 0, "/dev/null", flags).expect("process open");
    fs.write(proc.fd_table.get(fd).expect("entry").vfs_handle, b"\0", 0).expect("write byte");
    let mut buf = [0u8; 1];
    let count =
        fs.read(proc.fd_table.get(fd).expect("entry").vfs_handle, &mut buf, 0).expect("read byte");
    assert_eq!(count, 1);
    proc.close(&mut fs, fd).expect("process close");
}

#[test]
fn path_validation_rejects_traversal_in_integration_crate() {
    assert!(validate_path("/tmp/../etc/passwd").is_err());
}

#[test]
fn open_with_validation_requires_access_flags() {
    let mut fs = RamFs::new();
    let creds = Credentials::kernel();
    let err = open_with_validation(&mut fs, "/x", OpenFlags::default(), &creds, AccessMode::Read)
        .expect_err("flags required");
    assert_eq!(err.code, aether_types::ErrorCode::InvalidArgument);
}
