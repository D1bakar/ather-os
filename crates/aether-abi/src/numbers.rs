//! Syscall number definitions.
//!
//! Syscall numbers are assigned sequentially starting at 0.
//! Once assigned, numbers are never reused to preserve ABI stability.

/// System call numbers for Aether OS.
///
/// # Calling Convention (x86_64)
///
/// - Syscall number in `RAX`
/// - Arguments in `RDI`, `RSI`, `RDX`, `R10`, `R8`, `R9`
/// - Return value in `RAX` (negative values indicate errors per `ErrorCode`)
/// - `SYSCALL` instruction enters the kernel
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u64)]
pub enum SyscallNumber {
    /// Terminate the calling process. Args: exit_code (i32).
    Exit = 0,
    /// Write bytes to a file descriptor. Args: fd, buf, count.
    Write = 1,
    /// Read bytes from a file descriptor. Args: fd, buf, count.
    Read = 2,
    /// Open a file. Args: path, flags, mode.
    Open = 3,
    /// Close a file descriptor. Args: fd.
    Close = 4,
    /// Map memory. Args: addr, length, prot, flags, fd, offset.
    Mmap = 5,
    /// Unmap memory. Args: addr, length.
    Munmap = 6,
    /// Yield the CPU to the scheduler.
    Yield = 7,
    /// Get process ID.
    GetPid = 8,
    /// Send a signal to a process. Args: pid, signal.
    Kill = 9,
}

impl SyscallNumber {
    /// Converts a raw u64 into a `SyscallNumber`.
    ///
    /// Returns `None` for unassigned numbers.
    #[must_use]
    pub const fn from_u64(value: u64) -> Option<Self> {
        match value {
            0 => Some(Self::Exit),
            1 => Some(Self::Write),
            2 => Some(Self::Read),
            3 => Some(Self::Open),
            4 => Some(Self::Close),
            5 => Some(Self::Mmap),
            6 => Some(Self::Munmap),
            7 => Some(Self::Yield),
            8 => Some(Self::GetPid),
            9 => Some(Self::Kill),
            _ => None,
        }
    }

    /// Returns the numeric value of this syscall.
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self as u64
    }

    /// Returns the human-readable name of this syscall.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Exit => "exit",
            Self::Write => "write",
            Self::Read => "read",
            Self::Open => "open",
            Self::Close => "close",
            Self::Mmap => "mmap",
            Self::Munmap => "munmap",
            Self::Yield => "yield",
            Self::GetPid => "getpid",
            Self::Kill => "kill",
        }
    }
}

/// Total number of defined syscalls (exclusive upper bound for validation).
const SYSCALL_COUNT: u64 = 10;

/// Returns the number of defined syscalls.
#[must_use]
pub const fn syscall_count() -> u64 {
    SYSCALL_COUNT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn syscall_roundtrip() {
        for i in 0..SYSCALL_COUNT {
            let num = SyscallNumber::from_u64(i).expect("valid syscall");
            assert_eq!(num.as_u64(), i);
        }
    }

    #[test]
    fn unknown_syscall_returns_none() {
        assert!(SyscallNumber::from_u64(999).is_none());
    }
}
