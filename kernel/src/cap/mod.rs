//! Capability table and enforcement stubs (M5).

use aether_types::{
    AuditEventKind, CapabilityDescriptor, CapabilityId, CapabilityRights, ErrorCode, ObjectKind,
    SecurityDefaults,
};

use crate::security::audit::record_event;
use aether_sync::SpinMutex;

/// Maximum capabilities stored per process.
pub const MAX_CAPABILITIES: usize = 32;

/// Per-process capability table with enforcement stubs.
#[derive(Clone, Debug)]
pub struct CapabilityTable {
    slots: [Option<CapabilityDescriptor>; MAX_CAPABILITIES],
    next_slot: u32,
    policy: SecurityDefaults,
}

impl CapabilityTable {
    /// Creates an empty table with active security defaults.
    #[must_use]
    pub const fn new() -> Self {
        Self { slots: [None; MAX_CAPABILITIES], next_slot: 0, policy: SecurityDefaults::active() }
    }

    /// Inserts a capability and returns its kernel-issued id.
    pub fn grant(
        &mut self,
        object_kind: ObjectKind,
        rights: CapabilityRights,
    ) -> Result<CapabilityId, ErrorCode> {
        let slot =
            self.slots.iter_mut().position(|entry| entry.is_none()).ok_or(ErrorCode::OutOfMemory)?
                as u32;

        let descriptor = CapabilityDescriptor::new(slot, rights, object_kind);
        self.slots[slot as usize] = Some(descriptor);
        self.next_slot = self.next_slot.max(slot + 1);
        Ok(descriptor.id)
    }

    /// Looks up a descriptor by id, rejecting forged tokens when configured.
    #[must_use]
    pub fn get(&self, id: CapabilityId) -> Option<&CapabilityDescriptor> {
        if self.policy.reject_forged_capabilities && !id.is_valid() {
            record_event(
                AuditEventKind::ForgedCapability,
                0,
                u64::MAX,
                id.as_u64(),
                ErrorCode::PermissionDenied.as_i32(),
            );
            return None;
        }
        id.slot().and_then(|slot| self.slots.get(slot as usize)?.as_ref())
    }

    /// Returns `true` if any slot grants `required` on `object_kind`.
    #[must_use]
    pub fn has_rights(&self, object_kind: ObjectKind, required: CapabilityRights) -> bool {
        self.slots
            .iter()
            .flatten()
            .any(|desc| desc.object_kind == object_kind && desc.grants(required))
    }

    /// Enforcement stub — verifies capability id, object kind, and rights.
    pub fn check(
        &self,
        id: CapabilityId,
        object_kind: ObjectKind,
        required: CapabilityRights,
    ) -> Result<(), ErrorCode> {
        let descriptor = self.get(id).ok_or_else(|| {
            record_event(
                AuditEventKind::CapabilityDenied,
                0,
                u64::MAX,
                id.as_u64(),
                ErrorCode::PermissionDenied.as_i32(),
            );
            ErrorCode::PermissionDenied
        })?;

        if descriptor.object_kind != object_kind {
            record_event(
                AuditEventKind::CapabilityDenied,
                0,
                u64::MAX,
                id.as_u64(),
                ErrorCode::PermissionDenied.as_i32(),
            );
            return Err(ErrorCode::PermissionDenied);
        }

        if !descriptor.grants(required) {
            record_event(
                AuditEventKind::CapabilityDenied,
                0,
                u64::MAX,
                id.as_u64(),
                ErrorCode::PermissionDenied.as_i32(),
            );
            return Err(ErrorCode::PermissionDenied);
        }

        if self.policy.should_audit(true) {
            record_event(
                AuditEventKind::CapabilityGranted,
                0,
                u64::MAX,
                id.as_u64(),
                ErrorCode::Success.as_i32(),
            );
        }

        Ok(())
    }

    /// Stub enforcement for syscall dispatch — requires any matching capability in-table.
    pub fn enforce_syscall(
        &self,
        object_kind: ObjectKind,
        required: CapabilityRights,
    ) -> Result<(), ErrorCode> {
        if required == CapabilityRights::NONE {
            return Ok(());
        }
        if !self.policy.require_capability_for_io {
            return Ok(());
        }
        if self.has_rights(object_kind, required) {
            Ok(())
        } else {
            record_event(
                AuditEventKind::CapabilityDenied,
                0,
                u64::MAX,
                0,
                ErrorCode::PermissionDenied.as_i32(),
            );
            Err(ErrorCode::PermissionDenied)
        }
    }

    /// Returns the active security policy for this table.
    #[must_use]
    pub const fn policy(&self) -> SecurityDefaults {
        self.policy
    }
}

impl Default for CapabilityTable {
    fn default() -> Self {
        Self::new()
    }
}

// Global bring-up capability table until per-process tables are wired in the scheduler.
static CURRENT_TABLE: SpinMutex<CapabilityTable> = SpinMutex::new(CapabilityTable::new());

/// Runs `f` with mutable access to the global capability table (M5 bring-up stub).
pub fn with_current_table<R>(f: impl FnOnce(&mut CapabilityTable) -> R) -> R {
    f(&mut CURRENT_TABLE.lock())
}

/// Serializes host tests that mutate the global capability table.
#[cfg(test)]
pub fn lock_table_for_test() -> std::sync::MutexGuard<'static, ()> {
    static CAP_TABLE_TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());
    CAP_TABLE_TEST_MUTEX.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grant_and_check_succeeds() {
        let mut table = CapabilityTable::new();
        let id = table.grant(ObjectKind::File, CapabilityRights::READ).unwrap();
        assert!(table.check(id, ObjectKind::File, CapabilityRights::READ).is_ok());
    }

    #[test]
    fn forged_capability_rejected() {
        let table = CapabilityTable::new();
        let forged = CapabilityId::from_raw(0xDEAD_BEEF);
        assert!(table.get(forged).is_none());
    }

    #[test]
    fn check_rejects_insufficient_rights() {
        let mut table = CapabilityTable::new();
        let id = table.grant(ObjectKind::File, CapabilityRights::READ).unwrap();
        assert!(table.check(id, ObjectKind::File, CapabilityRights::WRITE).is_err());
    }

    #[test]
    fn enforce_syscall_requires_matching_capability() {
        let mut table = CapabilityTable::new();
        assert!(table.enforce_syscall(ObjectKind::File, CapabilityRights::WRITE).is_err());
        table.grant(ObjectKind::File, CapabilityRights::WRITE).unwrap();
        assert!(table.enforce_syscall(ObjectKind::File, CapabilityRights::WRITE).is_ok());
    }
}
