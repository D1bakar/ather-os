//! In-memory audit log stub for security-relevant events.
//!
//! M5 records events in a fixed-size ring buffer. Persistence and user-space
//! export are planned for a later milestone.

use aether_sync::SpinMutex;
use aether_types::{AuditEventKind, AuditRecord, SecurityDefaults};

/// Ring buffer capacity for audit records.
pub const AUDIT_LOG_CAPACITY: usize = 64;

/// Fixed-size audit log — no heap allocation.
#[derive(Debug)]
pub struct AuditLog {
    entries: [Option<AuditRecord>; AUDIT_LOG_CAPACITY],
    head: usize,
    count: usize,
    sequence: u64,
}

impl AuditLog {
    /// Creates an empty audit log.
    #[must_use]
    pub const fn new() -> Self {
        Self { entries: [const { None }; AUDIT_LOG_CAPACITY], head: 0, count: 0, sequence: 0 }
    }

    /// Appends a record, overwriting the oldest entry when full.
    pub fn record(&mut self, mut event: AuditRecord) {
        self.sequence = self.sequence.wrapping_add(1);
        event.sequence = self.sequence;
        self.entries[self.head] = Some(event);
        self.head = (self.head + 1) % AUDIT_LOG_CAPACITY;
        if self.count < AUDIT_LOG_CAPACITY {
            self.count += 1;
        }
    }

    /// Number of records currently stored (at most [`AUDIT_LOG_CAPACITY`]).
    #[must_use]
    pub const fn len(&self) -> usize {
        self.count
    }

    /// Returns `true` if the log contains no records.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Returns the most recently written record, if any.
    #[must_use]
    pub fn latest(&self) -> Option<AuditRecord> {
        if self.count == 0 {
            return None;
        }
        let index = self.head.wrapping_sub(1) % AUDIT_LOG_CAPACITY;
        self.entries[index]
    }
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new()
    }
}

static AUDIT_LOG: SpinMutex<AuditLog> = SpinMutex::new(AuditLog::new());

/// Records a security event when policy enables auditing for this outcome.
pub fn record_event(kind: AuditEventKind, pid: u32, syscall: u64, arg0: u64, error_code: i32) {
    let config = SecurityDefaults::active();
    let granted = matches!(kind, AuditEventKind::CapabilityGranted);
    if !config.should_audit(granted) {
        return;
    }

    let event = AuditRecord::new(0, kind, pid).with_syscall(syscall, arg0, error_code);
    AUDIT_LOG.lock().record(event);
}

/// Returns a snapshot of the most recent audit record (for tests and diagnostics).
#[must_use]
pub fn latest_record() -> Option<AuditRecord> {
    AUDIT_LOG.lock().latest()
}

/// Returns the number of stored audit records.
#[must_use]
pub fn record_count() -> usize {
    AUDIT_LOG.lock().len()
}

/// Clears the audit log (test helper).
pub fn clear() {
    *AUDIT_LOG.lock() = AuditLog::new();
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_types::ErrorCode;

    #[test]
    fn record_and_retrieve_latest() {
        clear();
        record_event(
            AuditEventKind::CapabilityDenied,
            1,
            1,
            0xAE7E,
            ErrorCode::PermissionDenied.as_i32(),
        );
        assert_eq!(record_count(), 1);
        let latest = latest_record().expect("record");
        assert_eq!(latest.kind, AuditEventKind::CapabilityDenied);
        assert_eq!(latest.pid, 1);
    }

    #[test]
    fn ring_buffer_overwrites_oldest() {
        let mut log = AuditLog::new();
        for index in 0..=AUDIT_LOG_CAPACITY {
            let event = AuditRecord::new(0, AuditEventKind::SyscallDenied, index as u32)
                .with_syscall(0, 0, -1);
            log.record(event);
        }
        assert_eq!(log.len(), AUDIT_LOG_CAPACITY);
    }
}
