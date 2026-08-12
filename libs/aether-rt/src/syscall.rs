//! Syscall wrappers using [`aether_abi::SyscallNumber`].

#[cfg(not(feature = "host"))]
use aether_abi::SyscallNumber;

/// Standard file descriptor numbers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum StdFd {
    /// Standard input.
    Stdin = 0,
    /// Standard output.
    Stdout = 1,
    /// Standard error.
    Stderr = 2,
}

impl StdFd {
    /// Returns the numeric fd value.
    #[must_use]
    pub const fn as_i32(self) -> i32 {
        self as i32
    }
}

/// Memory protection flags for [`mmap`] (subset of POSIX).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MmapProt {
    /// Page can be read.
    pub read: bool,
    /// Page can be written.
    pub write: bool,
    /// Page can be executed.
    pub exec: bool,
}

impl MmapProt {
    /// Read/write, no execute.
    #[must_use]
    pub const fn rw() -> Self {
        Self { read: true, write: true, exec: false }
    }

    #[cfg(not(feature = "host"))]
    const fn bits(self) -> u64 {
        let mut bits = 0u64;
        if self.read {
            bits |= 1;
        }
        if self.write {
            bits |= 2;
        }
        if self.exec {
            bits |= 4;
        }
        bits
    }
}

/// Sentinel returned by [`mmap`] when mapping fails.
pub const MAP_FAILED: *mut u8 = usize::MAX as *mut u8;

/// Yields the CPU to another runnable task.
#[must_use]
pub fn yield_cpu() -> i32 {
    #[cfg(feature = "host")]
    {
        0
    }

    #[cfg(not(feature = "host"))]
    {
        raw_syscall0(SyscallNumber::Yield.as_u64()) as i32
    }
}

/// Opens `path` with `flags` (`OpenFlags` bits from the kernel VFS).
#[must_use]
pub fn open(path: &str, flags: u32) -> i32 {
    #[cfg(feature = "host")]
    {
        let _ = (path, flags);
        -7
    }

    #[cfg(not(feature = "host"))]
    {
        let mut buf = [0u8; 256];
        let bytes = path.as_bytes();
        let len = bytes.len().min(buf.len() - 1);
        buf[..len].copy_from_slice(&bytes[..len]);
        buf[len] = 0;
        raw_syscall3(SyscallNumber::Open.as_u64(), buf.as_ptr() as u64, flags as u64, 0) as i32
    }
}

/// Reads up to `buf.len()` bytes from `fd`.
#[must_use]
pub fn read(fd: i32, buf: &mut [u8]) -> isize {
    #[cfg(feature = "host")]
    {
        let _ = (fd, buf);
        0
    }

    #[cfg(not(feature = "host"))]
    {
        if buf.is_empty() {
            return 0;
        }
        raw_syscall3(
            SyscallNumber::Read.as_u64(),
            fd as u64,
            buf.as_mut_ptr() as u64,
            buf.len() as u64,
        ) as isize
    }
}

/// Closes an open file descriptor.
#[must_use]
pub fn close(fd: i32) -> i32 {
    #[cfg(feature = "host")]
    {
        let _ = fd;
        0
    }

    #[cfg(not(feature = "host"))]
    {
        raw_syscall1(SyscallNumber::Close.as_u64(), fd as u64) as i32
    }
}

/// Terminates the calling process.
pub fn exit(code: i32) -> ! {
    #[cfg(feature = "host")]
    {
        std::process::exit(code);
    }

    #[cfg(not(feature = "host"))]
    {
        let _ = raw_syscall1(SyscallNumber::Exit.as_u64(), code as u64);
        loop {
            core::hint::spin_loop();
        }
    }
}

/// Writes up to `buf.len()` bytes to `fd`. Returns bytes written or negative error code.
pub fn write(fd: i32, buf: &[u8]) -> isize {
    #[cfg(feature = "host")]
    {
        host_write(fd, buf)
    }

    #[cfg(not(feature = "host"))]
    {
        if buf.is_empty() {
            return 0;
        }
        let ret = raw_syscall3(
            SyscallNumber::Write.as_u64(),
            fd as u64,
            buf.as_ptr() as u64,
            buf.len() as u64,
        );
        ret as isize
    }
}

/// Maps memory (stub — returns [`MAP_FAILED`] until the kernel implements M5 mmap).
#[must_use]
pub fn mmap(addr: *mut u8, length: usize, prot: MmapProt) -> *mut u8 {
    #[cfg(feature = "host")]
    {
        let _ = (addr, length, prot);
        MAP_FAILED
    }

    #[cfg(not(feature = "host"))]
    {
        let ret = raw_syscall6(
            SyscallNumber::Mmap.as_u64(),
            addr as u64,
            length as u64,
            prot.bits(),
            0,
            u64::MAX,
            0,
        );
        if ret < 0 {
            MAP_FAILED
        } else {
            ret as *mut u8
        }
    }
}

/// Unmaps memory (stub).
#[must_use]
pub fn munmap(addr: *mut u8, length: usize) -> i32 {
    #[cfg(feature = "host")]
    {
        let _ = (addr, length);
        -7
    }

    #[cfg(not(feature = "host"))]
    {
        raw_syscall2(SyscallNumber::Munmap.as_u64(), addr as u64, length as u64) as i32
    }
}

/// Returns the calling process id (stub returns 1 on host).
#[must_use]
pub fn getpid() -> i32 {
    #[cfg(feature = "host")]
    {
        1
    }

    #[cfg(not(feature = "host"))]
    {
        raw_syscall0(SyscallNumber::GetPid.as_u64()) as i32
    }
}

#[cfg(feature = "host")]
fn host_write(fd: i32, buf: &[u8]) -> isize {
    use std::io::{self, Write};
    let result = match fd {
        0 => Ok(0),
        1 => io::stdout().write_all(buf).map(|()| buf.len()),
        2 => io::stderr().write_all(buf).map(|()| buf.len()),
        _ => Err(io::Error::from(io::ErrorKind::InvalidInput)),
    };
    match result {
        Ok(n) => n as isize,
        Err(_) => -6,
    }
}

#[cfg(not(feature = "host"))]
#[inline(always)]
fn raw_syscall0(num: u64) -> i64 {
    let ret: i64;
    // SAFETY: kernel syscall entry validates the request; inline asm matches ABI.
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") num,
            lateout("rax") ret,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack),
        );
    }
    ret
}

#[cfg(not(feature = "host"))]
#[inline(always)]
fn raw_syscall1(num: u64, a0: u64) -> i64 {
    let ret: i64;
    // SAFETY: see raw_syscall0.
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") num,
            in("rdi") a0,
            lateout("rax") ret,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack),
        );
    }
    ret
}

#[cfg(not(feature = "host"))]
#[inline(always)]
fn raw_syscall2(num: u64, a0: u64, a1: u64) -> i64 {
    let ret: i64;
    // SAFETY: see raw_syscall0.
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") num,
            in("rdi") a0,
            in("rsi") a1,
            lateout("rax") ret,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack),
        );
    }
    ret
}

#[cfg(not(feature = "host"))]
#[inline(always)]
fn raw_syscall3(num: u64, a0: u64, a1: u64, a2: u64) -> i64 {
    let ret: i64;
    // SAFETY: see raw_syscall0.
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") num,
            in("rdi") a0,
            in("rsi") a1,
            in("rdx") a2,
            lateout("rax") ret,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack),
        );
    }
    ret
}

#[cfg(not(feature = "host"))]
#[inline(always)]
fn raw_syscall6(num: u64, a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64) -> i64 {
    let ret: i64;
    // SAFETY: see raw_syscall0.
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") num,
            in("rdi") a0,
            in("rsi") a1,
            in("rdx") a2,
            in("r10") a3,
            in("r8") a4,
            in("r9") a5,
            lateout("rax") ret,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack),
        );
    }
    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_write_stdout() {
        let n = write(StdFd::Stdout.as_i32(), b"test");
        assert_eq!(n, 4);
    }

    #[test]
    fn host_getpid_stub() {
        assert_eq!(getpid(), 1);
    }

    #[test]
    fn host_mmap_stub_fails() {
        assert_eq!(mmap(core::ptr::null_mut(), 4096, MmapProt::rw()), MAP_FAILED);
    }
}
