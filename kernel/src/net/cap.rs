//! Network capability permission check hook (ADR-0004 scaffold).
//!
//! M8 provides a fail-closed gate for socket operations until the full capability
//! broker lands in post-M4 milestones.

use aether_types::{AetherError, AetherResult, ErrorCode};

/// Rights required for network operations (bit flags).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum NetworkRight {
    /// Permission to bind a local endpoint.
    Bind = 1 << 0,
    /// Permission to send datagrams/segments.
    Send = 1 << 1,
    /// Permission to receive inbound traffic.
    Receive = 1 << 2,
    /// Permission to initiate outbound connections.
    Connect = 1 << 3,
}

impl NetworkRight {
    /// All network rights (bootstrap / kernel-internal use).
    pub const ALL: u32 =
        Self::Bind as u32 | Self::Send as u32 | Self::Receive as u32 | Self::Connect as u32;
}

/// High-level network operation mapped to capability rights.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NetworkOp {
    /// Bind a local address/port.
    Bind,
    /// Send data.
    Send,
    /// Receive data.
    Receive,
    /// Connect to a remote endpoint.
    Connect,
}

impl NetworkOp {
    const fn required_right(self) -> NetworkRight {
        match self {
            Self::Bind => NetworkRight::Bind,
            Self::Send => NetworkRight::Send,
            Self::Receive => NetworkRight::Receive,
            Self::Connect => NetworkRight::Connect,
        }
    }
}

/// Opaque network capability token (kernel-issued; not forgeable from user space).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NetworkCapability {
    /// Kernel capability slot index (0 = invalid).
    pub slot: u32,
    /// Attenuated rights bitmask ([`NetworkRight`]).
    pub rights: u32,
}

impl NetworkCapability {
    /// Creates a capability with explicit rights.
    #[must_use]
    pub const fn new(slot: u32, rights: u32) -> Self {
        Self { slot, rights }
    }

    /// Kernel-internal unrestricted capability (slot 0 reserved).
    #[must_use]
    pub const fn kernel() -> Self {
        Self { slot: u32::MAX, rights: NetworkRight::ALL }
    }

    /// Returns true if `right` is granted.
    #[must_use]
    pub const fn allows(&self, right: NetworkRight) -> bool {
        self.rights & (right as u32) != 0
    }
}

/// Validates that `cap` authorizes `op`. Fail-closed when slot is 0 or rights missing.
pub fn check_network_capability(cap: NetworkCapability, op: NetworkOp) -> AetherResult<()> {
    if cap.slot == 0 {
        return Err(AetherError::new(ErrorCode::PermissionDenied));
    }
    let needed = op.required_right();
    if cap.allows(needed) {
        Ok(())
    } else {
        Err(AetherError::new(ErrorCode::PermissionDenied))
    }
}

/// Hook for the future capability broker — validates slot + rights bitfield.
pub fn check_network_capability_raw(slot: u32, rights: u32, op: NetworkOp) -> AetherResult<()> {
    check_network_capability(NetworkCapability::new(slot, rights), op)
}

/// Hook invoked before syscall layer delegates a network object (future M5).
pub fn grant_network_capability(slot: u32, rights: u32) -> NetworkCapability {
    NetworkCapability::new(slot, rights)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deny_zero_slot() {
        let cap = NetworkCapability::new(0, NetworkRight::ALL);
        assert!(check_network_capability(cap, NetworkOp::Send).is_err());
    }

    #[test]
    fn allow_with_rights() {
        let cap = NetworkCapability::new(3, NetworkRight::Send as u32);
        assert!(check_network_capability(cap, NetworkOp::Send).is_ok());
        assert!(check_network_capability(cap, NetworkOp::Bind).is_err());
    }
}
