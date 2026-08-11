//! Network stack foundations (M8).
//!
//! Hand-rolled protocol stubs suitable for `#![no_std]` without a heap.
//! [`smoltcp`] is intentionally not used until M3 provides a kernel allocator.
//!
//! [`smoltcp`]: https://docs.rs/smoltcp

pub mod arp;
pub mod cap;
pub mod ethernet;
pub mod ipv4;
pub mod nic;
pub mod tcp;
pub mod udp;

pub use arp::{ArpOperation, ArpPacket};
pub use cap::{check_network_capability, NetworkCapability, NetworkOp};
pub use ethernet::{EthernetFrame, Ethertype, MacAddress, ETHERNET_HEADER_LEN, MIN_FRAME_LEN};
pub use ipv4::{Ipv4Header, Ipv4Protocol, IPV4_HEADER_LEN};
pub use nic::{init_nics, poll_nics, Nic, NicId, NicKind};
pub use tcp::{TcpSocket, TcpState};
pub use udp::{UdpSocket, UDP_HEADER_LEN};

use aether_types::AetherResult;

/// Well-known loopback IPv4 address (`127.0.0.1`).
pub const LOOPBACK_IPV4: [u8; 4] = [127, 0, 0, 1];

/// Initializes the virtual NIC layer (loopback + driver placeholder).
pub fn init() {
    nic::init_nics();
}

/// Polls all registered NICs for inbound frames (stub — no hardware DMA yet).
pub fn poll() {
    nic::poll_nics();
}

/// Parses an inbound L2 frame and dispatches to the appropriate L3 handler stub.
pub fn dispatch_frame(frame: &[u8]) -> AetherResult<()> {
    let eth = EthernetFrame::parse(frame)?;
    match eth.ethertype {
        Ethertype::Arp => arp::handle_arp(eth.payload),
        Ethertype::Ipv4 => ipv4::dispatch_ipv4(eth.payload),
        Ethertype::Unknown(_) => {
            Err(aether_types::AetherError::new(aether_types::ErrorCode::NotSupported))
        }
    }
}
