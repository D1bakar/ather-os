//! UDP socket stub — bind/send/receive API without a real datagram queue.

use crate::net::cap::{check_network_capability, NetworkCapability, NetworkOp};
use crate::net::ipv4::Ipv4Header;
use aether_types::{AetherError, AetherResult, ErrorCode};

/// UDP header size in bytes.
pub const UDP_HEADER_LEN: usize = 8;

/// Parsed UDP header.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UdpHeader {
    /// Source port.
    pub src_port: u16,
    /// Destination port.
    pub dst_port: u16,
    /// Length field (header + payload).
    pub length: u16,
    /// Checksum (optional for IPv4).
    pub checksum: u16,
}

impl UdpHeader {
    /// Parses eight UDP header bytes.
    pub fn parse(data: &[u8]) -> AetherResult<Self> {
        if data.len() < UDP_HEADER_LEN {
            return Err(AetherError::new(ErrorCode::InvalidArgument));
        }
        Ok(Self {
            src_port: u16::from_be_bytes([data[0], data[1]]),
            dst_port: u16::from_be_bytes([data[2], data[3]]),
            length: u16::from_be_bytes([data[4], data[5]]),
            checksum: u16::from_be_bytes([data[6], data[7]]),
        })
    }
}

/// Minimal UDP socket (single local endpoint, no RX queue).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UdpSocket {
    /// Local IPv4 bind address.
    pub local_addr: [u8; 4],
    /// Local port (0 = unbound).
    pub local_port: u16,
    /// Capability token required for operations.
    pub capability: NetworkCapability,
}

impl UdpSocket {
    /// Creates an unbound UDP socket with the given capability.
    #[must_use]
    pub const fn new(capability: NetworkCapability) -> Self {
        Self { local_addr: [0; 4], local_port: 0, capability }
    }

    /// Binds the socket to `addr:port`.
    pub fn bind(&mut self, addr: [u8; 4], port: u16) -> AetherResult<()> {
        check_network_capability(self.capability, NetworkOp::Bind)?;
        if port == 0 {
            return Err(AetherError::new(ErrorCode::InvalidArgument));
        }
        self.local_addr = addr;
        self.local_port = port;
        Ok(())
    }

    /// Sends `payload` to `dst:port` (stub — no NIC TX).
    pub fn send_to(&self, dst: [u8; 4], port: u16, payload: &[u8]) -> AetherResult<usize> {
        check_network_capability(self.capability, NetworkOp::Send)?;
        if self.local_port == 0 {
            return Err(AetherError::new(ErrorCode::InvalidArgument));
        }
        if port == 0 {
            return Err(AetherError::new(ErrorCode::InvalidArgument));
        }
        let _ = (dst, payload);
        Ok(payload.len())
    }

    /// Receives up to `buf.len()` bytes (stub — always `NotFound`).
    pub fn recv_from(&self, buf: &mut [u8]) -> AetherResult<(usize, [u8; 4], u16)> {
        check_network_capability(self.capability, NetworkOp::Receive)?;
        let _ = buf;
        Err(AetherError::new(ErrorCode::NotFound))
    }
}

/// Inbound UDP datagram stub handler.
pub fn handle_datagram(header: &Ipv4Header<'_>) -> AetherResult<()> {
    let udp = UdpHeader::parse(header.payload)?;
    if udp.length as usize > header.payload.len() {
        return Err(AetherError::new(ErrorCode::InvalidArgument));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::cap::NetworkCapability;

    #[test]
    fn bind_and_send_stub() {
        let cap = NetworkCapability::new(1, u32::MAX);
        let mut sock = UdpSocket::new(cap);
        sock.bind([127, 0, 0, 1], 8080).unwrap();
        let sent = sock.send_to([127, 0, 0, 1], 53, b"ping").unwrap();
        assert_eq!(sent, 4);
    }
}
