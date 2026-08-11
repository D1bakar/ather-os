//! TCP state machine skeleton — minimal listen/connect without retransmission.

use crate::net::cap::{check_network_capability, NetworkCapability, NetworkOp};
use crate::net::ipv4::Ipv4Header;
use aether_types::{AetherError, AetherResult, ErrorCode};

/// TCP header length without options.
pub const TCP_HEADER_LEN: usize = 20;

/// TCP connection state (RFC 793 subset).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TcpState {
    /// Initial / fully closed.
    Closed,
    /// Passive open — waiting for SYN.
    Listen,
    /// Active open — SYN sent, awaiting SYN-ACK.
    SynSent,
    /// Passive open — SYN received, awaiting ACK.
    SynReceived,
    /// Data transfer state.
    Established,
}

/// Parsed TCP header (fixed 20-byte form, no options).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TcpHeader {
    /// Source port.
    pub src_port: u16,
    /// Destination port.
    pub dst_port: u16,
    /// Sequence number.
    pub seq: u32,
    /// Acknowledgment number.
    pub ack: u32,
    /// Data offset in 32-bit words.
    pub data_offset: u8,
    /// Control flags (SYN, ACK, FIN, …).
    pub flags: u8,
    /// Window size.
    pub window: u16,
}

impl TcpHeader {
    /// Parses a TCP header from the start of `data`.
    pub fn parse(data: &[u8]) -> AetherResult<Self> {
        if data.len() < TCP_HEADER_LEN {
            return Err(AetherError::new(ErrorCode::InvalidArgument));
        }
        let data_offset = (data[12] >> 4) * 4;
        if data_offset < TCP_HEADER_LEN as u8 || data.len() < data_offset as usize {
            return Err(AetherError::new(ErrorCode::InvalidArgument));
        }
        Ok(Self {
            src_port: u16::from_be_bytes([data[0], data[1]]),
            dst_port: u16::from_be_bytes([data[2], data[3]]),
            seq: u32::from_be_bytes([data[4], data[5], data[6], data[7]]),
            ack: u32::from_be_bytes([data[8], data[9], data[10], data[11]]),
            data_offset,
            flags: data[13],
            window: u16::from_be_bytes([data[14], data[15]]),
        })
    }

    /// Returns true if the SYN flag is set.
    #[must_use]
    pub const fn syn(&self) -> bool {
        self.flags & 0x02 != 0
    }

    /// Returns true if the ACK flag is set.
    #[must_use]
    pub const fn ack(&self) -> bool {
        self.flags & 0x10 != 0
    }
}

/// Minimal TCP socket with listen/connect state transitions only.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TcpSocket {
    /// Current FSM state.
    pub state: TcpState,
    /// Local IPv4 address.
    pub local_addr: [u8; 4],
    /// Local port.
    pub local_port: u16,
    /// Remote IPv4 address (meaningful after connect).
    pub remote_addr: [u8; 4],
    /// Remote port.
    pub remote_port: u16,
    /// Outbound sequence number (stub).
    pub iss: u32,
    /// Capability token for permission checks.
    pub capability: NetworkCapability,
}

impl TcpSocket {
    /// Creates a closed TCP socket.
    #[must_use]
    pub const fn new(capability: NetworkCapability) -> Self {
        Self {
            state: TcpState::Closed,
            local_addr: [0; 4],
            local_port: 0,
            remote_addr: [0; 4],
            remote_port: 0,
            iss: 0,
            capability,
        }
    }

    /// Passive open — transitions `Closed` → `Listen`.
    pub fn listen(&mut self, addr: [u8; 4], port: u16) -> AetherResult<()> {
        check_network_capability(self.capability, NetworkOp::Bind)?;
        if self.state != TcpState::Closed {
            return Err(AetherError::new(ErrorCode::InvalidArgument));
        }
        if port == 0 {
            return Err(AetherError::new(ErrorCode::InvalidArgument));
        }
        self.local_addr = addr;
        self.local_port = port;
        self.state = TcpState::Listen;
        Ok(())
    }

    /// Active open — transitions `Closed` → `SynSent` (no wire I/O yet).
    pub fn connect(&mut self, remote: [u8; 4], port: u16) -> AetherResult<()> {
        check_network_capability(self.capability, NetworkOp::Connect)?;
        if self.state != TcpState::Closed {
            return Err(AetherError::new(ErrorCode::InvalidArgument));
        }
        if port == 0 {
            return Err(AetherError::new(ErrorCode::InvalidArgument));
        }
        self.remote_addr = remote;
        self.remote_port = port;
        self.iss = 1; // stub ISN
        self.state = TcpState::SynSent;
        Ok(())
    }

    /// Accepts an inbound connection (stub — no pending queue).
    pub fn accept(&self) -> AetherResult<TcpSocket> {
        check_network_capability(self.capability, NetworkOp::Receive)?;
        if self.state != TcpState::Listen {
            return Err(AetherError::new(ErrorCode::InvalidArgument));
        }
        Err(AetherError::new(ErrorCode::NotFound))
    }

    /// Processes an inbound SYN while in `Listen` (internal/stub).
    pub fn on_syn_received(&mut self, header: &TcpHeader) -> AetherResult<()> {
        if self.state != TcpState::Listen {
            return Err(AetherError::new(ErrorCode::InvalidArgument));
        }
        if !header.syn() {
            return Err(AetherError::new(ErrorCode::InvalidArgument));
        }
        self.remote_addr = self.local_addr; // placeholder
        self.remote_port = header.src_port;
        self.state = TcpState::SynReceived;
        Ok(())
    }

    /// Completes active open when SYN-ACK arrives (stub).
    pub fn on_syn_ack(&mut self) -> AetherResult<()> {
        if self.state != TcpState::SynSent {
            return Err(AetherError::new(ErrorCode::InvalidArgument));
        }
        self.state = TcpState::Established;
        Ok(())
    }
}

/// Inbound TCP segment stub handler.
pub fn handle_segment(header: &Ipv4Header<'_>) -> AetherResult<()> {
    let tcp = TcpHeader::parse(header.payload)?;
    let _ = tcp;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::cap::NetworkCapability;

    #[test]
    fn listen_connect_fsm() {
        let cap = NetworkCapability::new(1, u32::MAX);
        let mut server = TcpSocket::new(cap);
        server.listen([0, 0, 0, 0], 443).unwrap();
        assert_eq!(server.state, TcpState::Listen);

        let mut client = TcpSocket::new(cap);
        client.connect([127, 0, 0, 1], 443).unwrap();
        assert_eq!(client.state, TcpState::SynSent);
        client.on_syn_ack().unwrap();
        assert_eq!(client.state, TcpState::Established);
    }
}
