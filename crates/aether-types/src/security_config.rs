//! Runtime security policy defaults shared by kernel and host tests.

/// Active security policy toggles for M5 bring-up.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SecurityDefaults {
    /// Reject unknown syscall numbers with a defined error.
    pub deny_unknown_syscalls: bool,
    /// Validate userspace pointer arguments on syscall entry.
    pub validate_user_pointers: bool,
    /// Reject capability ids without the kernel magic prefix.
    pub reject_forged_capabilities: bool,
    /// Require matching capabilities for I/O syscalls.
    pub require_capability_for_io: bool,
    /// Record audit events for denied access attempts.
    pub audit_denied_access: bool,
    /// Record audit events for granted capability checks.
    pub audit_granted_access: bool,
    /// Maximum bytes for userspace path pointers.
    pub max_user_path_bytes: u64,
}

impl SecurityDefaults {
    /// Returns the active policy (production-hardened defaults on host CI).
    #[must_use]
    pub const fn active() -> Self {
        Self {
            deny_unknown_syscalls: true,
            validate_user_pointers: true,
            reject_forged_capabilities: true,
            require_capability_for_io: true,
            audit_denied_access: true,
            audit_granted_access: false,
            max_user_path_bytes: 256,
        }
    }

    /// Returns whether an event with outcome `granted` should be audited.
    #[must_use]
    pub const fn should_audit(self, granted: bool) -> bool {
        if granted {
            self.audit_granted_access
        } else {
            self.audit_denied_access
        }
    }
}
