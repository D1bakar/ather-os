//! Intel 82540EM (e1000) driver stub for QEMU `-device e1000`.
//!
//! MMIO register access is not implemented until M3 I/O mapping exists.

use super::{Nic, NicId, NicKind};
use crate::net::ethernet::MacAddress;
use aether_types::{AetherError, AetherResult, ErrorCode};

/// QEMU default MAC for the first e1000 adapter.
const DEFAULT_MAC: MacAddress = MacAddress([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]);

/// Placeholder PCI BAR0 MMIO base (not mapped in M8).
pub const E1000_MMIO_BASE: u64 = 0xFEBC_0000;

/// e1000 register offsets (documentation anchors for future driver work).
pub mod regs {
    /// Device control register.
    pub const CTRL: u32 = 0x0000;
    /// Device status register.
    pub const STATUS: u32 = 0x0008;
    /// Receive control register.
    pub const RCTL: u32 = 0x0100;
    /// Transmit control register.
    pub const TCTL: u32 = 0x0400;
}

/// e1000 NIC stub — reports identity, no DMA.
pub struct E1000Stub {
    id: NicId,
    mac: MacAddress,
    link_up: bool,
}

impl E1000Stub {
    /// Creates an e1000 stub with the default QEMU MAC.
    #[must_use]
    pub const fn new(id: NicId) -> Self {
        Self { id, mac: DEFAULT_MAC, link_up: false }
    }

    /// Simulates link establishment after imaginary PHY init.
    pub const fn set_link_up(&mut self, up: bool) {
        self.link_up = up;
    }

    /// Returns whether the stub reports carrier detect.
    #[must_use]
    pub const fn link_up(&self) -> bool {
        self.link_up
    }
}

impl Nic for E1000Stub {
    fn id(&self) -> NicId {
        self.id
    }

    fn kind(&self) -> NicKind {
        NicKind::E1000
    }

    fn mac(&self) -> MacAddress {
        self.mac
    }

    fn poll_rx(&mut self, _buf: &mut [u8]) -> AetherResult<usize> {
        if !self.link_up {
            return Ok(0);
        }
        Ok(0)
    }

    fn send(&mut self, _frame: &[u8]) -> AetherResult<usize> {
        if !self.link_up {
            return Err(AetherError::new(ErrorCode::IoError));
        }
        Err(AetherError::new(ErrorCode::NotSupported))
    }
}
