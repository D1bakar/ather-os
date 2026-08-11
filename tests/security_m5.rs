//! Host integration tests for M5 security — capabilities, syscall validation, audit log.

mod support;

use aether_abi::{descriptor_for, SyscallArgs, SyscallNumber};
use aether_kernel::cap::{with_current_table, CapabilityTable};
use aether_kernel::security::{clear_audit_log, latest_record, record_count, SecurityDefaults};
use aether_kernel::syscall::dispatch;
use aether_types::{
    validate_user_address, validate_user_buffer, AuditEventKind, CapabilityRights, ErrorCode,
    ObjectKind, UserBuffer,
};
use support::with_global_cap_lock;

#[test]
fn security_defaults_are_production_hardened() {
    let cfg = SecurityDefaults::active();
    assert!(cfg.deny_unknown_syscalls);
    assert!(cfg.validate_user_pointers);
    assert!(cfg.reject_forged_capabilities);
    assert!(cfg.require_capability_for_io);
    assert!(cfg.audit_denied_access);
}

#[test]
fn user_pointer_validation_rejects_kernel_addresses() {
    assert!(validate_user_address(0xFFFF_8000_0000_0000).is_err());
    assert!(validate_user_buffer(UserBuffer::new(0x1000, 8), &SecurityDefaults::active()).is_ok());
}

#[test]
fn write_syscall_audits_and_denies_without_capability() {
    with_global_cap_lock(|| {
        clear_audit_log();
        with_current_table(|table| *table = CapabilityTable::new());

        let args = SyscallArgs::new(1, 0x2000, 16, 0, 0, 0);
        let result = dispatch(SyscallNumber::Write.as_u64(), args);
        assert_eq!(result, ErrorCode::PermissionDenied.as_i32() as i64);
        assert!(record_count() >= 1);
        let record = latest_record().expect("audit entry");
        assert_eq!(record.kind, AuditEventKind::CapabilityDenied);
    });
}

#[test]
fn write_syscall_passes_validation_with_file_write_capability() {
    with_global_cap_lock(|| {
        clear_audit_log();
        with_current_table(|table| {
            *table = CapabilityTable::new();
            table.grant(ObjectKind::File, CapabilityRights::WRITE).unwrap();
        });

        let args = SyscallArgs::new(1, 0x2000, 16, 0, 0, 0);
        let result = dispatch(SyscallNumber::Write.as_u64(), args);
        assert_eq!(result, 16);
    });
}

#[test]
fn read_descriptor_marks_buffer_pointer() {
    let desc = descriptor_for(SyscallNumber::Read).unwrap();
    assert!(desc.arg_is_pointer(1));
    assert_eq!(desc.required_object, Some(ObjectKind::File));
    assert!(desc.required_rights.contains(CapabilityRights::READ));
}

#[test]
fn forged_capability_id_is_rejected_by_table() {
    let table = CapabilityTable::new();
    use aether_types::CapabilityId;
    let forged = CapabilityId::from_raw(0xBAD0_CAFE);
    assert!(table.get(forged).is_none());
}
