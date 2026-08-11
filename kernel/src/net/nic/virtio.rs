//! VirtIO-net MMIO driver placeholder for QEMU `-device virtio-net-pci`.
//!
//! Virtqueue setup requires a heap and DMA; M8 exposes identity and no-op hooks only.

use super::{Nic, NicId, NicKind};
use crate::net::ethernet::MacAddress;
use aether_types::{AetherError, AetherResult, ErrorCode};

/// Default VirtIO-net MAC (QEMU-style locally administered).
const DEFAULT_MAC: MacAddress = MacAddress([0x52, 0x54, 0x00, 0x00, 0x00, 0x01]);

/// VirtIO MMIO magic value (`virt`).
pub const VIRTIO_MMIO_MAGIC: u32 = 0x7472_6976;

/// VirtIO device ID for network card.
pub const VIRTIO_DEV_NET: u32 = 1;

/// VirtIO-net stub without mapped MMIO or virtqueues.
pub struct VirtioNetStub {
    id: NicId,
    mac: MacAddress,
    features_ack: u64,
}

impl VirtioNetStub {
    /// Creates a VirtIO-net placeholder.
    #[must_use]
    pub const fn new(id: NicId) -> Self {
        Self { id, mac: DEFAULT_MAC, features_ack: 0 }
    }

    /// Records negotiated feature bits (stub).
    pub const fn acknowledge_features(&mut self, features: u64) {
        self.features_ack = features;
    }

    /// Returns acknowledged feature bits.
    #[must_use]
    pub const fn features(&self) -> u64 {
        self.features_ack
    }
}

impl Nic for VirtioNetStub {
    fn id(&self) -> NicId {
        self.id
    }

    fn kind(&self) -> NicKind {
        NicKind::VirtioNet
    }

    fn mac(&self) -> MacAddress {
        self.mac
    }

    fn poll_rx(&mut self, _buf: &mut [u8]) -> AetherResult<usize> {
        Ok(0)
    }

    fn send(&mut self, _frame: &[u8]) -> AetherResult<usize> {
        Err(AetherError::new(ErrorCode::NotSupported))
    }
}
