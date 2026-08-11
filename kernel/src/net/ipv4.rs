//! IPv4 header parsing, building, and checksum helpers.

use crate::net::{arp, tcp, udp};
use aether_types::{AetherError, AetherResult, ErrorCode};

/// Fixed IPv4 header length when no options are present.
pub const IPV4_HEADER_LEN: usize = 20;

/// IPv4 next-header protocol number.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Ipv4Protocol {
    /// ICMP (not implemented in M8).
    Icmp = 1,
    /// Transmission Control Protocol.
    Tcp = 6,
    /// User Datagram Protocol.
    Udp = 17,
}

impl Ipv4Protocol {
    const fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::Icmp),
            6 => Some(Self::Tcp),
            17 => Some(Self::Udp),
            _ => None,
        }
    }
}

/// Parsed IPv4 header with payload slice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Ipv4Header<'a> {
    /// Version (must be 4) and IHL in 32-bit words.
    pub version_ihl: u8,
    /// Differentiated services / ECN field.
    pub dscp_ecn: u8,
    /// Total length including header.
    pub total_length: u16,
    /// Identification field.
    pub identification: u16,
    /// Flags and fragment offset.
    pub flags_fragment: u16,
    /// Time to live.
    pub ttl: u8,
    /// Next-header protocol.
    pub protocol: Ipv4Protocol,
    /// Header checksum (wire value).
    pub checksum: u16,
    /// Source address.
    pub src: [u8; 4],
    /// Destination address.
    pub dst: [u8; 4],
    /// Bytes following the fixed 20-byte header (options + payload).
    pub payload: &'a [u8],
}

impl<'a> Ipv4Header<'a> {
    /// Returns the IHL field (header length in 32-bit words).
    #[must_use]
    pub const fn ihl(&self) -> u8 {
        self.version_ihl & 0x0F
    }

    /// Returns the IP version nibble (expected 4).
    #[must_use]
    pub const fn version(&self) -> u8 {
        self.version_ihl >> 4
    }

    /// Header length in bytes derived from IHL.
    #[must_use]
    pub const fn header_len(&self) -> usize {
        (self.ihl() as usize) * 4
    }

    /// Parses an IPv4 datagram.
    pub fn parse(data: &'a [u8]) -> AetherResult<Self> {
        if data.len() < IPV4_HEADER_LEN {
            return Err(AetherError::new(ErrorCode::InvalidArgument));
        }

        let version_ihl = data[0];
        if version_ihl >> 4 != 4 {
            return Err(AetherError::new(ErrorCode::InvalidArgument));
        }

        let ihl = (version_ihl & 0x0F) as usize;
        let header_len = ihl * 4;
        if header_len < IPV4_HEADER_LEN || data.len() < header_len {
            return Err(AetherError::new(ErrorCode::InvalidArgument));
        }

        let checksum = u16::from_be_bytes([data[10], data[11]]);
        if checksum != 0 {
            let mut scratch = [0u8; IPV4_HEADER_LEN];
            scratch[..header_len].copy_from_slice(&data[..header_len]);
            scratch[10] = 0;
            scratch[11] = 0;
            if compute_checksum(&scratch[..header_len]) != checksum {
                return Err(AetherError::new(ErrorCode::InvalidArgument));
            }
        }

        let protocol_byte = data[9];
        let protocol = Ipv4Protocol::from_u8(protocol_byte)
            .ok_or(AetherError::new(ErrorCode::NotSupported))?;

        let mut src = [0_u8; 4];
        src.copy_from_slice(&data[12..16]);
        let mut dst = [0_u8; 4];
        dst.copy_from_slice(&data[16..20]);

        Ok(Self {
            version_ihl,
            dscp_ecn: data[1],
            total_length: u16::from_be_bytes([data[2], data[3]]),
            identification: u16::from_be_bytes([data[4], data[5]]),
            flags_fragment: u16::from_be_bytes([data[6], data[7]]),
            ttl: data[8],
            protocol,
            checksum,
            src,
            dst,
            payload: &data[header_len..],
        })
    }

    /// Builds a minimal IPv4 header (no options) into `out` and returns bytes written.
    pub fn build_minimal(
        out: &mut [u8],
        src: [u8; 4],
        dst: [u8; 4],
        protocol: Ipv4Protocol,
        payload_len: u16,
    ) -> AetherResult<usize> {
        if out.len() < IPV4_HEADER_LEN {
            return Err(AetherError::new(ErrorCode::InvalidArgument));
        }

        let total_length = IPV4_HEADER_LEN as u16 + payload_len;
        out[0] = 0x45; // version 4, IHL 5
        out[1] = 0;
        out[2..4].copy_from_slice(&total_length.to_be_bytes());
        out[4..6].copy_from_slice(&0_u16.to_be_bytes());
        out[6..8].copy_from_slice(&0_u16.to_be_bytes());
        out[8] = 64; // TTL
        out[9] = protocol as u8;
        out[10..12].copy_from_slice(&0_u16.to_be_bytes()); // checksum placeholder
        out[12..16].copy_from_slice(&src);
        out[16..20].copy_from_slice(&dst);

        let checksum = compute_checksum(&out[..IPV4_HEADER_LEN]);
        out[10..12].copy_from_slice(&checksum.to_be_bytes());
        Ok(IPV4_HEADER_LEN)
    }
}

/// Internet checksum (RFC 1071) over `data`.
#[must_use]
pub fn compute_checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut i = 0;
    while i + 1 < data.len() {
        sum += u32::from(u16::from_be_bytes([data[i], data[i + 1]]));
        i += 2;
    }
    if i < data.len() {
        sum += u32::from(u16::from_be_bytes([data[i], 0]));
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !sum as u16
}

/// Stub L3 dispatch — validates IPv4 and forwards to UDP/TCP stubs.
pub fn dispatch_ipv4(payload: &[u8]) -> AetherResult<()> {
    let header = Ipv4Header::parse(payload)?;
    match header.protocol {
        Ipv4Protocol::Udp => udp::handle_datagram(&header),
        Ipv4Protocol::Tcp => tcp::handle_segment(&header),
        Ipv4Protocol::Icmp => {
            // ICMP stub: accept valid IPv4 framing only.
            Ok(())
        }
    }
}

/// Resolves an IPv4 address to a MAC via ARP stub (always returns broadcast placeholder).
#[must_use]
pub fn resolve_next_hop_mac(_dst: [u8; 4]) -> arp::ArpPacket {
    arp::ArpPacket {
        sender_hw: crate::net::ethernet::MacAddress::ZERO,
        sender_proto: [0, 0, 0, 0],
        target_hw: crate::net::ethernet::MacAddress::BROADCAST,
        target_proto: _dst,
        operation: arp::ArpOperation::Request,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_and_parse_ipv4() {
        let mut buf = [0_u8; 40];
        let hdr_len =
            Ipv4Header::build_minimal(&mut buf, [10, 0, 0, 1], [10, 0, 0, 2], Ipv4Protocol::Udp, 8)
                .unwrap();
        let parsed = Ipv4Header::parse(&buf[..hdr_len + 8]).unwrap();
        assert_eq!(parsed.src, [10, 0, 0, 1]);
        assert_eq!(parsed.dst, [10, 0, 0, 2]);
        assert_eq!(parsed.protocol, Ipv4Protocol::Udp);
    }

    #[test]
    fn checksum_nonzero() {
        let mut buf = [0_u8; 20];
        Ipv4Header::build_minimal(
            &mut buf,
            [192, 168, 1, 1],
            [192, 168, 1, 2],
            Ipv4Protocol::Tcp,
            0,
        )
        .unwrap();
        assert_ne!(u16::from_be_bytes([buf[10], buf[11]]), 0);
    }
}
