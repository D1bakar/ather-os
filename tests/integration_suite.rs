//! Cross-subsystem integration smoke suite.
//!
//! Aggregates lightweight checks that span kernel modules and shared crates.
//! Heavier property and fuzz tests live in dedicated `property_*` and `fuzz_*`
//! targets; QEMU boot remains in `qemu_boot` (`#[ignore]`).

use aether_abi::{lookup_syscall, syscall_count, SyscallArgs, SyscallNumber};
use aether_kernel::fs::RamFs;
use aether_kernel::process::{Process, ProcessId};
use aether_kernel::sched::TaskId;
use aether_kernel::syscall::dispatch;
use aether_kernel::vfs::path::validate_path;
use aether_kernel::vfs::{OpenFlags, Vfs};
use aether_kernel::{is_host_stub, KERNEL_VERSION};
use aether_types::{ErrorCode, PhysicalAddress};

#[test]
fn host_stub_and_version_invariants() {
    assert!(is_host_stub());
    assert!(!KERNEL_VERSION.is_empty());
}

#[test]
fn syscall_table_covers_full_number_range() {
    for index in 0..syscall_count() {
        let desc = lookup_syscall(index).expect("descriptor");
        assert_eq!(desc.number.as_u64(), index);
    }
}

#[test]
fn process_ramfs_open_read_close_pipeline() {
    let mut fs = RamFs::new();
    fs.seed_file("/init", b"boot").expect("seed");
    let mut proc = Process::new(ProcessId::new(1), PhysicalAddress::new(0), TaskId::new(1));
    let fd = proc.open(&mut fs, 0, "/init", OpenFlags::READ).expect("process open");

    let mut buf = [0u8; 4];
    let entry = proc.fd_table.get(fd).expect("fd entry");
    let count = fs.read(entry.vfs_handle, &mut buf, 0).expect("read");
    assert_eq!(count, 4);
    assert_eq!(&buf, b"boot");
    proc.close(&mut fs, fd).expect("close");
}

#[test]
fn dispatch_unknown_and_yield_endpoints() {
    let unknown = dispatch(999, SyscallArgs::default());
    assert_eq!(unknown, ErrorCode::NotSupported.as_i32() as i64);

    let yielded = dispatch(SyscallNumber::Yield.as_u64(), SyscallArgs::default());
    assert_eq!(yielded, ErrorCode::Success.as_i32() as i64);
}

#[test]
fn vfs_rejects_invalid_paths_used_by_process_layer() {
    assert!(validate_path("/../etc/passwd").is_err());
    assert!(validate_path("relative").is_err());
}
