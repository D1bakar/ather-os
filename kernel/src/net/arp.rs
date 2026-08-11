//! Address Resolution Protocol (ARP) stub.
//!
//! Handles IPv4-over-Ethernet ARP request/reply framing only; no cache or NIC TX yet.

use crate::net::ethernet::MacAddress;
use aether_types::{AetherError, AetherResult, ErrorCode};

/// ARP hardware / protocol constants for Ethernet + IPv4.
pub const ARP_HW_ETHERNET: u16 = 1;
/// ARP protocol type for IPv4 (`0x0800`).
pub const ARP_PROTO_IPV4: u16 = 0x0800;
/// Hardware address length for Ethernet (6 bytes).
pub const ARP_HW_ADDR_LEN: u8 = 6;
/// Protocol address length for IPv4 (4 bytes).
pub const ARP_PROTO_ADDR_LEN: u8 = 4;
/// Fixed ARP packet size for Ethernet/IPv4.
pub const ARP_PACKET_LEN: usize = 28;

/// ARP operation code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum ArpOperation {
    /// ARP request.
    Request = 1,
    /// ARP reply.
    Reply = 2,
}

impl ArpOperation {
    const fn from_be(value: u16) -> Option<Self> {
        match value {
            1 => Some(Self::Request),
            2 => Some(Self::Reply),
            _ => None,
        }
    }
}

/// Parsed ARP packet (Ethernet + IPv4).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArpPacket {
    /// Sender hardware (MAC) address.
    pub sender_hw: MacAddress,
    /// Sender protocol (IPv4) address.
    pub sender_proto: [u8; 4],
    /// Target hardware (MAC) address.
    pub target_hw: MacAddress,
    /// Target protocol (IPv4) address.
    pub target_proto: [u8; 4],
    /// Request or reply.
    pub operation: ArpOperation,
}

impl ArpPacket {
    /// Parses raw ARP payload (after the Ethernet header).
    pub fn parse(data: &[u8]) -> AetherResult<Self> {
        if data.len() < ARP_PACKET_LEN {
            return Err(AetherError::new(ErrorCode::InvalidArgument));
        }

        let hw_type = u16::from_be_bytes([data[0], data[1]]);
        let proto_type = u16::from_be_bytes([data[2], data[3]]);
        let hw_len = data[4];
        let proto_len = data[5];
        let op = u16::from_be_bytes([data[6], data[7]]);

        if hw_type != ARP_HW_ETHERNET
            || proto_type != ARP_PROTO_IPV4
            || hw_len != ARP_HW_ADDR_LEN
            || proto_len != ARP_PROTO_ADDR_LEN
        {
            return Err(AetherError::new(ErrorCode::InvalidArgument));
        }

        let operation =
            ArpOperation::from_be(op).ok_or(AetherError::new(ErrorCode::NotSupported))?;

        let mut sender_hw = [0_u8; 6];
        sender_hw.copy_from_slice(&data[8..14]);
        let mut sender_proto = [0_u8; 4];
        sender_proto.copy_from_slice(&data[14..18]);
        let mut target_hw = [0_u8; 6];
        target_hw.copy_from_slice(&data[18..24]);
        let mut target_proto = [0_u8; 4];
        target_proto.copy_from_slice(&data[24..28]);

        Ok(Self {
            sender_hw: MacAddress(sender_hw),
            sender_proto,
            target_hw: MacAddress(target_hw),
            target_proto,
            operation,
        })
    }

    /// Serializes this ARP packet into `out`.
    pub fn write_to(&self, out: &mut [u8]) -> AetherResult<usize> {
        if out.len() < ARP_PACKET_LEN {
            return Err(AetherError::new(ErrorCode::InvalidArgument));
        }

        out[0..2].copy_from_slice(&ARP_HW_ETHERNET.to_be_bytes());
        out[2..4].copy_from_slice(&ARP_PROTO_IPV4.to_be_bytes());
        out[4] = ARP_HW_ADDR_LEN;
        out[5] = ARP_PROTO_ADDR_LEN;
        out[6..8].copy_from_slice(&(self.operation as u16).to_be_bytes());
        out[8..14].copy_from_slice(&self.sender_hw.0);
        out[14..18].copy_from_slice(&self.sender_proto);
        out[18..24].copy_from_slice(&self.target_hw.0);
        out[24..28].copy_from_slice(&self.target_proto);
        Ok(ARP_PACKET_LEN)
    }
}

/// Stub inbound ARP handler — validates framing only.
pub fn handle_arp(payload: &[u8]) -> AetherResult<()> {
    let _packet = ArpPacket::parse(payload)?;
    // Future: update ARP cache, synthesize replies on loopback.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arp_request_roundtrip() {
        let packet = ArpPacket {
            sender_hw: MacAddress([0x02; 6]),
            sender_proto: [192, 168, 0, 1],
            target_hw: MacAddress::ZERO,
            target_proto: [192, 168, 0, 2],
            operation: ArpOperation::Request,
        };
        let mut buf = [0_u8; ARP_PACKET_LEN];
        packet.write_to(&mut buf).unwrap();
        let parsed = ArpPacket::parse(&buf).unwrap();
        assert_eq!(parsed, packet);
    }
}
