//! Host integration tests for M5 scheduler and syscall foundations.

mod support;

use aether_abi::{descriptor_for, lookup_syscall, syscall_count, SyscallArgs, SyscallNumber};
use aether_kernel::cap::{with_current_table, CapabilityTable};
use aether_kernel::syscall::dispatch;
use aether_types::{CapabilityRights, ErrorCode, ObjectKind};
use support::with_global_cap_lock;

#[test]
fn syscall_table_has_entry_per_number() {
    assert_eq!(syscall_count(), 10);
    for index in 0..syscall_count() {
        assert!(lookup_syscall(index).is_some());
    }
}

#[test]
fn write_descriptor_requires_file_capability() {
    let desc = descriptor_for(SyscallNumber::Write).unwrap();
    assert_eq!(desc.required_object, Some(ObjectKind::File));
    assert!(desc.required_rights.contains(CapabilityRights::WRITE));
}

#[test]
fn dispatch_rejects_kernel_pointer_on_write() {
    let args = SyscallArgs::new(0, 0xFFFF_8000_0000_0000, 4, 0, 0, 0);
    let result = dispatch(SyscallNumber::Write.as_u64(), args);
    assert_eq!(result, ErrorCode::BadAddress.as_i32() as i64);
}

#[test]
fn capability_table_enforces_rights() {
    let mut table = CapabilityTable::new();
    let id = table.grant(ObjectKind::Device, CapabilityRights::READ).unwrap();
    assert!(table.check(id, ObjectKind::Device, CapabilityRights::READ).is_ok());
    assert!(table.check(id, ObjectKind::Device, CapabilityRights::WRITE).is_err());
}

#[test]
fn write_requires_capability_in_global_table() {
    with_global_cap_lock(|| {
        with_current_table(|table| *table = CapabilityTable::new());
        let args = SyscallArgs::new(1, 0x1000, 4, 0, 0, 0);
        assert_eq!(
            dispatch(SyscallNumber::Write.as_u64(), args),
            ErrorCode::PermissionDenied.as_i32() as i64
        );
        with_current_table(|table| {
            table.grant(ObjectKind::File, CapabilityRights::WRITE).unwrap();
        });
        assert_eq!(dispatch(SyscallNumber::Write.as_u64(), args), 4);
    });
}
