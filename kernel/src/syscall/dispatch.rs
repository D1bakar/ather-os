//! Syscall demux, pointer validation, capability enforcement, and handlers.

use super::validate::{validate_user_cstr, validate_user_slice};
use aether_abi::{lookup_syscall, SyscallArgs, SyscallDescriptor, SyscallNumber};
use aether_types::{CapabilityRights, ErrorCode, SecurityDefaults};

use crate::cap::with_current_table;
use crate::fs::mount;
use crate::process::{self, FdEntry, Process, ROOT_MOUNT_ID};
use crate::sched;
use crate::security::audit::record_event;
use crate::vfs::path::{AccessMode, Credentials};
use crate::vfs::{open_with_validation, FileDescriptor, OpenFlags};
use aether_types::AuditEventKind;

#[cfg(not(feature = "host-stub"))]
use crate::serial;

/// Dispatches a syscall by number, validating pointers and capabilities per the ABI table.
#[must_use]
pub fn dispatch(number: u64, args: SyscallArgs) -> i64 {
    let config = SecurityDefaults::active();

    let Some(desc) = lookup_syscall(number) else {
        if config.deny_unknown_syscalls {
            record_event(
                AuditEventKind::SyscallDenied,
                0,
                number,
                0,
                ErrorCode::NotSupported.as_i32(),
            );
            return ErrorCode::NotSupported.as_i64();
        }
        return ErrorCode::Internal.as_i64();
    };

    if config.validate_user_pointers {
        if let Err(code) = validate_syscall_pointers(desc, &args) {
            record_event(AuditEventKind::BadUserPointer, 0, number, args.arg0, code.as_i32());
            return code.as_i64();
        }
    }

    if let Err(code) = enforce_syscall_capabilities(desc) {
        record_event(AuditEventKind::CapabilityDenied, 0, number, args.arg0, code.as_i32());
        return code.as_i64();
    }

    match desc.number {
        SyscallNumber::Exit => sys_exit(args),
        SyscallNumber::Write => sys_write(args),
        SyscallNumber::Read => sys_read(args),
        SyscallNumber::Open => sys_open(args),
        SyscallNumber::Close => sys_close(args),
        SyscallNumber::Mmap => sys_not_implemented(),
        SyscallNumber::Munmap => sys_not_implemented(),
        SyscallNumber::Yield => sys_yield(),
        SyscallNumber::GetPid => sys_getpid(),
        SyscallNumber::Kill => sys_not_implemented(),
    }
}

fn validate_syscall_pointers(
    desc: &SyscallDescriptor,
    args: &SyscallArgs,
) -> Result<(), ErrorCode> {
    let raw = [args.arg0, args.arg1, args.arg2, args.arg3, args.arg4, args.arg5];
    for (index, value) in raw.iter().enumerate() {
        if !desc.arg_is_pointer(index as u8) {
            continue;
        }
        match desc.number {
            SyscallNumber::Open => validate_user_cstr(*value)?,
            SyscallNumber::Write | SyscallNumber::Read => {
                validate_user_slice(*value, args.arg2, true)?;
            }
            _ => validate_user_slice(*value, 1, true)?,
        }
    }
    Ok(())
}

fn enforce_syscall_capabilities(desc: &SyscallDescriptor) -> Result<(), ErrorCode> {
    let Some(object) = desc.required_object else {
        return Ok(());
    };
    if desc.required_rights == CapabilityRights::NONE {
        return Ok(());
    }

    if let Some(pid) = sched::current_process_id() {
        if let Some(result) = process::with_process(pid, |proc| {
            proc.capabilities.enforce_syscall(object, desc.required_rights)
        }) {
            return result;
        }
    }

    with_current_table(|table| table.enforce_syscall(object, desc.required_rights))
}

fn sys_exit(args: SyscallArgs) -> i64 {
    let _code = args.arg0 as i32;
    sched::terminate_current();
    ErrorCode::Success.as_i64()
}

/// Standard output file descriptor (serial console in M5).
const FD_STDOUT: u64 = 1;
/// Standard error file descriptor (serial console in M5).
const FD_STDERR: u64 = 2;

fn sys_write(args: SyscallArgs) -> i64 {
    let fd = args.arg0;
    let count = args.arg2;

    if fd != FD_STDOUT && fd != FD_STDERR {
        return ErrorCode::InvalidArgument.as_i64();
    }
    if count == 0 {
        return 0;
    }

    #[cfg(feature = "host-stub")]
    {
        // Pointer validation already ran; host CI cannot dereference fake user addresses.
        count as i64
    }

    #[cfg(not(feature = "host-stub"))]
    {
        let buf = args.arg1;
        const MAX_WRITE: usize = 4096;
        let len = count.min(MAX_WRITE as u64) as usize;
        let mut scratch = [0u8; MAX_WRITE];
        if copy_from_user(&mut scratch[..len], buf).is_err() {
            return ErrorCode::BadAddress.as_i64();
        }
        serial::write_str(core::str::from_utf8(&scratch[..len]).unwrap_or(""));
        len as i64
    }
}

fn sys_read(args: SyscallArgs) -> i64 {
    let fd = FileDescriptor(args.arg0 as u32);
    let count = args.arg2;
    if count == 0 {
        return 0;
    }

    #[cfg(feature = "host-stub")]
    {
        let _ = fd;
        0
    }

    #[cfg(not(feature = "host-stub"))]
    {
        const MAX_READ: usize = 4096;
        let len = count.min(MAX_READ as u64) as usize;
        let mut scratch = [0u8; MAX_READ];

        let read = match process::with_current(|proc| {
            mount::with_root(|fs| proc.read(fs, fd, &mut scratch[..len]))
        }) {
            Some(Ok(n)) => n,
            Some(Err(err)) => return err.code.as_i32() as i64,
            None => return ErrorCode::Internal.as_i64(),
        };

        if copy_to_user(args.arg1, &scratch[..read]).is_err() {
            return ErrorCode::BadAddress.as_i64();
        }
        read as i64
    }
}

fn sys_open(args: SyscallArgs) -> i64 {
    let flags = OpenFlags::from_bits(args.arg1 as u32);

    #[cfg(feature = "host-stub")]
    {
        let _ = flags;
        ErrorCode::NotSupported.as_i64()
    }

    #[cfg(not(feature = "host-stub"))]
    {
        let mut path_buf = [0u8; 256];
        let path_len = match copy_user_cstr(args.arg0, &mut path_buf) {
            Ok(len) => len,
            Err(code) => return code.as_i64(),
        };
        let path = match core::str::from_utf8(&path_buf[..path_len]) {
            Ok(text) => text,
            Err(_) => return ErrorCode::InvalidArgument.as_i64(),
        };

        match process::with_current(|proc| open_path(proc, path, flags)) {
            Some(Ok(fd)) => fd.index() as i64,
            Some(Err(err)) => err.code.as_i32() as i64,
            None => ErrorCode::Internal.as_i64(),
        }
    }
}

fn sys_close(args: SyscallArgs) -> i64 {
    let fd = FileDescriptor(args.arg0 as u32);

    #[cfg(feature = "host-stub")]
    {
        let _ = fd;
        ErrorCode::NotSupported.as_i64()
    }

    #[cfg(not(feature = "host-stub"))]
    {
        match process::with_current(|proc| mount::with_root(|fs| proc.close(fs, fd))) {
            Some(Ok(())) => ErrorCode::Success.as_i64(),
            Some(Err(err)) => err.code.as_i32() as i64,
            None => ErrorCode::Internal.as_i64(),
        }
    }
}

/// Opens `path` on the root ramfs and records the handle in `proc`'s fd table.
#[allow(dead_code)] // Host integration tests call this via `aether_kernel::syscall::dispatch::open_path`.
pub fn open_path(
    proc: &mut Process,
    path: &str,
    flags: OpenFlags,
) -> Result<FileDescriptor, aether_types::AetherError> {
    mount::with_root(|fs| {
        let vfs_fd =
            open_with_validation(fs, path, flags, &Credentials::kernel(), AccessMode::Read)?;
        proc.fd_table.allocate(FdEntry { mount_id: ROOT_MOUNT_ID, vfs_handle: vfs_fd, flags })
    })
}

fn sys_yield() -> i64 {
    sched::yield_now();
    ErrorCode::Success.as_i64()
}

fn sys_getpid() -> i64 {
    sched::current_process_id().map_or(1, |pid| pid.0 as i64)
}

/// Copies `len` bytes from a validated userspace address into `dest`.
#[cfg(not(feature = "host-stub"))]
fn copy_from_user(dest: &mut [u8], src: u64) -> Result<(), ErrorCode> {
    for (index, slot) in dest.iter_mut().enumerate() {
        // SAFETY: `src` was validated as a canonical user buffer by dispatch.
        let byte = unsafe { core::ptr::read_volatile((src + index as u64) as *const u8) };
        *slot = byte;
    }
    Ok(())
}

/// Copies `data` into a validated userspace buffer.
#[cfg(not(feature = "host-stub"))]
fn copy_to_user(dest: u64, data: &[u8]) -> Result<(), ErrorCode> {
    for (index, &byte) in data.iter().enumerate() {
        // SAFETY: `dest` was validated as a canonical user buffer by dispatch.
        unsafe { core::ptr::write_volatile((dest + index as u64) as *mut u8, byte) };
    }
    Ok(())
}

/// Copies a NUL-terminated string from userspace into `buf`.
#[cfg(not(feature = "host-stub"))]
fn copy_user_cstr(ptr: u64, buf: &mut [u8]) -> Result<usize, ErrorCode> {
    for (index, slot) in buf.iter_mut().enumerate() {
        // SAFETY: `ptr` was validated as a user C string by dispatch.
        let byte = unsafe { core::ptr::read_volatile((ptr + index as u64) as *const u8) };
        *slot = byte;
        if byte == 0 {
            return Ok(index);
        }
    }
    Err(ErrorCode::InvalidArgument)
}

fn sys_not_implemented() -> i64 {
    ErrorCode::NotSupported.as_i64()
}

#[allow(clippy::wrong_self_convention)]
trait ErrorCodeExt {
    fn as_i64(self) -> i64;
}

impl ErrorCodeExt for ErrorCode {
    fn as_i64(self) -> i64 {
        self.as_i32() as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cap::CapabilityTable;
    use crate::fs::mount;
    use crate::process::{Process, ProcessId};
    use crate::sched::TaskId;
    use crate::security::audit::{clear, record_count};
    use aether_abi::SyscallNumber;
    use aether_types::{ObjectKind, PhysicalAddress};

    #[test]
    fn unknown_syscall_returns_not_supported() {
        clear();
        let result = dispatch(999, SyscallArgs::default());
        assert_eq!(result, ErrorCode::NotSupported.as_i64());
        assert!(record_count() >= 1);
    }

    #[test]
    fn yield_syscall_succeeds_on_host() {
        let result = dispatch(SyscallNumber::Yield.as_u64(), SyscallArgs::default());
        assert_eq!(result, ErrorCode::Success.as_i64());
    }

    #[test]
    fn write_rejects_kernel_buffer() {
        clear();
        let args = SyscallArgs::new(1, 0xFFFF_8000_0000_0000, 8, 0, 0, 0);
        let result = dispatch(SyscallNumber::Write.as_u64(), args);
        assert_eq!(result, ErrorCode::BadAddress.as_i64());
    }

    #[test]
    fn write_requires_file_write_capability_when_enforced() {
        let _guard = crate::cap::lock_table_for_test();
        clear();
        with_current_table(|table| {
            *table = CapabilityTable::new();
        });
        let args = SyscallArgs::new(1, 0x1000, 4, 0, 0, 0);
        let denied = dispatch(SyscallNumber::Write.as_u64(), args);
        assert_eq!(denied, ErrorCode::PermissionDenied.as_i64());

        with_current_table(|table| {
            table.grant(ObjectKind::File, CapabilityRights::WRITE).unwrap();
        });
        let allowed = dispatch(SyscallNumber::Write.as_u64(), args);
        assert_eq!(allowed, 4);
    }

    #[test]
    fn write_to_stdout_returns_byte_count() {
        let _guard = crate::cap::lock_table_for_test();
        with_current_table(|table| {
            *table = CapabilityTable::new();
            table.grant(ObjectKind::File, CapabilityRights::WRITE).unwrap();
        });
        let args = SyscallArgs::new(1, 0x1000, 8, 0, 0, 0);
        assert_eq!(dispatch(SyscallNumber::Write.as_u64(), args), 8);
    }

    #[test]
    fn open_path_allocates_process_fd() {
        mount::init();
        mount::with_root(|fs| fs.seed_file("/init", b"boot").expect("seed"));
        let mut proc = Process::new(ProcessId::new(7), PhysicalAddress::new(0), TaskId::new(1));
        proc.grant_default_io_caps();
        let fd = open_path(&mut proc, "/init", OpenFlags::READ).expect("open");
        assert_eq!(fd.index(), 0);
        assert_eq!(proc.fd_table.open_count(), 1);
    }
}
