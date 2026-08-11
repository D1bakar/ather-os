//! Syscall dispatch metadata table shared by kernel and tooling.
//!
//! The kernel uses this table to validate syscall numbers, determine which
//! arguments require userspace pointer checks, and document the stable ABI.

use crate::SyscallNumber;
use aether_types::{CapabilityRights, ObjectKind};

/// Metadata describing one syscall entry in the dispatch table.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SyscallDescriptor {
    /// Stable syscall number.
    pub number: SyscallNumber,
    /// Human-readable name (`"write"`, `"yield"`, …).
    pub name: &'static str,
    /// Number of register arguments (0–6 per x86_64 ABI).
    pub arg_count: u8,
    /// Bitmask: bit `i` set if argument `i` is a userspace pointer.
    pub ptr_arg_mask: u8,
    /// Capability object kind required to invoke this syscall, if any.
    pub required_object: Option<ObjectKind>,
    /// Minimum rights required on the capability, when `required_object` is set.
    pub required_rights: CapabilityRights,
}

impl SyscallDescriptor {
    /// Returns `true` if argument `index` (0–5) is documented as a user pointer.
    #[must_use]
    pub const fn arg_is_pointer(self, index: u8) -> bool {
        index < 6 && (self.ptr_arg_mask & (1 << index)) != 0
    }
}

/// Stable dispatch table — one entry per defined syscall number.
pub const SYSCALL_TABLE: &[SyscallDescriptor] = &[
    SyscallDescriptor {
        number: SyscallNumber::Exit,
        name: "exit",
        arg_count: 1,
        ptr_arg_mask: 0,
        required_object: None,
        required_rights: CapabilityRights::NONE,
    },
    SyscallDescriptor {
        number: SyscallNumber::Write,
        name: "write",
        arg_count: 3,
        ptr_arg_mask: 0b010,
        required_object: Some(ObjectKind::File),
        required_rights: CapabilityRights::WRITE,
    },
    SyscallDescriptor {
        number: SyscallNumber::Read,
        name: "read",
        arg_count: 3,
        ptr_arg_mask: 0b010,
        required_object: Some(ObjectKind::File),
        required_rights: CapabilityRights::READ,
    },
    SyscallDescriptor {
        number: SyscallNumber::Open,
        name: "open",
        arg_count: 3,
        ptr_arg_mask: 0b001,
        required_object: None,
        required_rights: CapabilityRights::NONE,
    },
    SyscallDescriptor {
        number: SyscallNumber::Close,
        name: "close",
        arg_count: 1,
        ptr_arg_mask: 0,
        required_object: Some(ObjectKind::File),
        required_rights: CapabilityRights::NONE,
    },
    SyscallDescriptor {
        number: SyscallNumber::Mmap,
        name: "mmap",
        arg_count: 6,
        ptr_arg_mask: 0,
        required_object: Some(ObjectKind::Memory),
        required_rights: CapabilityRights::READ,
    },
    SyscallDescriptor {
        number: SyscallNumber::Munmap,
        name: "munmap",
        arg_count: 2,
        ptr_arg_mask: 0,
        required_object: Some(ObjectKind::Memory),
        required_rights: CapabilityRights::WRITE,
    },
    SyscallDescriptor {
        number: SyscallNumber::Yield,
        name: "yield",
        arg_count: 0,
        ptr_arg_mask: 0,
        required_object: None,
        required_rights: CapabilityRights::NONE,
    },
    SyscallDescriptor {
        number: SyscallNumber::GetPid,
        name: "getpid",
        arg_count: 0,
        ptr_arg_mask: 0,
        required_object: None,
        required_rights: CapabilityRights::NONE,
    },
    SyscallDescriptor {
        number: SyscallNumber::Kill,
        name: "kill",
        arg_count: 2,
        ptr_arg_mask: 0,
        required_object: Some(ObjectKind::Process),
        required_rights: CapabilityRights::DESTROY,
    },
];

/// Looks up dispatch metadata for a raw syscall number.
#[must_use]
pub fn lookup_syscall(number: u64) -> Option<&'static SyscallDescriptor> {
    let num = SyscallNumber::from_u64(number)?;
    SYSCALL_TABLE.iter().find(|entry| entry.number == num)
}

/// Returns the dispatch table entry for a known syscall number.
#[must_use]
pub fn descriptor_for(number: SyscallNumber) -> Option<&'static SyscallDescriptor> {
    SYSCALL_TABLE.iter().find(|entry| entry.number == number)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_covers_all_syscall_numbers() {
        assert_eq!(SYSCALL_TABLE.len(), crate::syscall_count() as usize);
    }

    #[test]
    fn write_marks_buffer_as_pointer() {
        let desc = descriptor_for(SyscallNumber::Write).unwrap();
        assert!(desc.arg_is_pointer(1));
        assert!(!desc.arg_is_pointer(0));
    }

    #[test]
    fn lookup_unknown_returns_none() {
        assert!(lookup_syscall(999).is_none());
    }
}
