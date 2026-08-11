//! Software loopback network interface (`lo`).

use super::{Nic, NicId, NicKind};
use crate::net::ethernet::{MacAddress, ETHERNET_HEADER_LEN};
use aether_types::{AetherError, AetherResult, ErrorCode};

/// Fixed loopback MAC (locally administered).
const LOOPBACK_MAC: MacAddress = MacAddress([0x02, 0x00, 0x00, 0x00, 0x00, 0x01]);

/// Single-slot loopback RX queue (one frame).
pub struct LoopbackNic {
    id: NicId,
    pending: Option<[u8; 1518]>,
    pending_len: usize,
}

impl LoopbackNic {
    /// Creates the loopback interface.
    #[must_use]
    pub const fn new(id: NicId) -> Self {
        Self { id, pending: None, pending_len: 0 }
    }
}

impl Nic for LoopbackNic {
    fn id(&self) -> NicId {
        self.id
    }

    fn kind(&self) -> NicKind {
        NicKind::Loopback
    }

    fn mac(&self) -> MacAddress {
        LOOPBACK_MAC
    }

    fn poll_rx(&mut self, buf: &mut [u8]) -> AetherResult<usize> {
        if let Some(frame) = self.pending.take() {
            let len = self.pending_len;
            if buf.len() < len {
                return Err(AetherError::new(ErrorCode::InvalidArgument));
            }
            buf[..len].copy_from_slice(&frame[..len]);
            self.pending_len = 0;
            Ok(len)
        } else {
            Ok(0)
        }
    }

    fn send(&mut self, frame: &[u8]) -> AetherResult<usize> {
        if frame.len() < ETHERNET_HEADER_LEN || frame.len() > 1518 {
            return Err(AetherError::new(ErrorCode::InvalidArgument));
        }
        let mut slot = [0_u8; 1518];
        slot[..frame.len()].copy_from_slice(frame);
        self.pending = Some(slot);
        self.pending_len = frame.len();
        Ok(frame.len())
    }
}
