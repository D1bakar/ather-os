//! Security audit event types.

/// Severity of an audit record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuditSeverity {
    /// Informational event.
    Info,
    /// Access or syscall denied.
    Warning,
    /// Potential security violation.
    Critical,
}

/// Kind of security-relevant event recorded in the audit log.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuditEventKind {
    /// Syscall rejected as unknown or disallowed.
    SyscallDenied,
    /// Capability check failed.
    CapabilityDenied,
    /// Capability check succeeded (when auditing grants).
    CapabilityGranted,
    /// Invalid userspace pointer.
    BadUserPointer,
    /// Forged capability token rejected.
    ForgedCapability,
}

/// One audit log entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuditRecord {
    /// Monotonic sequence assigned by the audit log.
    pub sequence: u64,
    /// Timestamp stub (host CI uses 0).
    pub timestamp: u64,
    /// Event classification.
    pub kind: AuditEventKind,
    /// Process id associated with the event.
    pub pid: u32,
    /// Syscall number when applicable.
    pub syscall: u64,
    /// First syscall argument when applicable.
    pub arg0: u64,
    /// Stable error code outcome.
    pub error_code: i32,
}

impl AuditRecord {
    /// Creates a new record with sequence assigned later by the log.
    #[must_use]
    pub const fn new(timestamp: u64, kind: AuditEventKind, pid: u32) -> Self {
        Self { sequence: 0, timestamp, kind, pid, syscall: 0, arg0: 0, error_code: 0 }
    }

    /// Attaches syscall context to this record.
    #[must_use]
    pub const fn with_syscall(mut self, syscall: u64, arg0: u64, error_code: i32) -> Self {
        self.syscall = syscall;
        self.arg0 = arg0;
        self.error_code = error_code;
        self
    }
}
