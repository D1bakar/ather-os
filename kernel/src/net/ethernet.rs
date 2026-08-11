//! IEEE 802.3 / Ethernet II frame parsing and building.

use aether_types::{AetherError, AetherResult, ErrorCode};

/// Minimum valid Ethernet frame length (excluding preamble/SFD and FCS).
pub const MIN_FRAME_LEN: usize = 14;
/// Standard Ethernet header size (destination + source + EtherType).
pub const ETHERNET_HEADER_LEN: usize = 14;

/// Six-byte MAC address.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct MacAddress(pub [u8; 6]);

impl MacAddress {
    /// Broadcast MAC (`ff:ff:ff:ff:ff:ff`).
    pub const BROADCAST: Self = Self([0xFF; 6]);

    /// All-zero MAC (invalid as a source address).
    pub const ZERO: Self = Self([0; 6]);

    /// Parses six raw bytes into a MAC address.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 6]) -> Self {
        Self(bytes)
    }
}

/// EtherType values used by the stack.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ethertype {
    /// Address Resolution Protocol.
    Arp,
    /// Internet Protocol version 4.
    Ipv4,
    /// Unrecognized EtherType.
    Unknown(u16),
}

impl Ethertype {
    const ARP: u16 = 0x0806;
    const IPV4: u16 = 0x0800;

    /// Converts a big-endian EtherType field to the enum.
    #[must_use]
    pub const fn from_be(value: u16) -> Self {
        match value {
            Self::ARP => Self::Arp,
            Self::IPV4 => Self::Ipv4,
            other => Self::Unknown(other),
        }
    }

    /// Returns the big-endian wire value.
    #[must_use]
    pub const fn to_be(self) -> u16 {
        match self {
            Self::Arp => Self::ARP,
            Self::Ipv4 => Self::IPV4,
            Self::Unknown(v) => v,
        }
    }
}

/// Parsed Ethernet II frame (header + payload slice).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EthernetFrame<'a> {
    /// Destination MAC.
    pub dst: MacAddress,
    /// Source MAC.
    pub src: MacAddress,
    /// L3 protocol selector.
    pub ethertype: Ethertype,
    /// Bytes following the 14-byte header.
    pub payload: &'a [u8],
}

impl<'a> EthernetFrame<'a> {
    /// Parses `data` as an Ethernet II frame.
    pub fn parse(data: &'a [u8]) -> AetherResult<Self> {
        if data.len() < ETHERNET_HEADER_LEN {
            return Err(AetherError::new(ErrorCode::InvalidArgument));
        }

        let mut dst = [0_u8; 6];
        dst.copy_from_slice(&data[0..6]);
        let mut src = [0_u8; 6];
        src.copy_from_slice(&data[6..12]);
        let ethertype_raw = u16::from_be_bytes([data[12], data[13]]);

        Ok(Self {
            dst: MacAddress(dst),
            src: MacAddress(src),
            ethertype: Ethertype::from_be(ethertype_raw),
            payload: &data[ETHERNET_HEADER_LEN..],
        })
    }

    /// Serializes this frame into `out`, returning the number of bytes written.
    pub fn write_to(&self, out: &mut [u8]) -> AetherResult<usize> {
        let total = ETHERNET_HEADER_LEN + self.payload.len();
        if out.len() < total {
            return Err(AetherError::new(ErrorCode::InvalidArgument));
        }

        out[0..6].copy_from_slice(&self.dst.0);
        out[6..12].copy_from_slice(&self.src.0);
        let ethertype = self.ethertype.to_be().to_be_bytes();
        out[12..14].copy_from_slice(&ethertype);
        out[14..total].copy_from_slice(self.payload);
        Ok(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ipv4_ethertype() {
        let mut frame = [0_u8; 20];
        frame[12..14].copy_from_slice(&0x0800_u16.to_be_bytes());
        let parsed = EthernetFrame::parse(&frame).unwrap();
        assert_eq!(parsed.ethertype, Ethertype::Ipv4);
    }

    #[test]
    fn roundtrip_write() {
        let payload = [1_u8, 2, 3];
        let eth = EthernetFrame {
            dst: MacAddress::BROADCAST,
            src: MacAddress([0x02; 6]),
            ethertype: Ethertype::Arp,
            payload: &payload,
        };
        let mut buf = [0_u8; 32];
        let n = eth.write_to(&mut buf).unwrap();
        assert_eq!(n, ETHERNET_HEADER_LEN + payload.len());
        let reparsed = EthernetFrame::parse(&buf[..n]).unwrap();
        assert_eq!(reparsed.dst, eth.dst);
        assert_eq!(reparsed.ethertype, Ethertype::Arp);
        assert_eq!(reparsed.payload, &payload);
    }
}
