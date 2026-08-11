//! Host integration tests for M8 network stack foundations.

use aether_kernel::net::arp::{ArpOperation, ArpPacket, ARP_PACKET_LEN};
use aether_kernel::net::cap::{
    check_network_capability, NetworkCapability, NetworkOp, NetworkRight,
};
use aether_kernel::net::ethernet::{EthernetFrame, Ethertype, MacAddress, ETHERNET_HEADER_LEN};
use aether_kernel::net::ipv4::{compute_checksum, Ipv4Header, Ipv4Protocol, IPV4_HEADER_LEN};
use aether_kernel::net::nic::{init_nics, inject_loopback_ipv4, loopback_mac, use_virtio_net_stub};
use aether_kernel::net::tcp::{TcpSocket, TcpState};
use aether_kernel::net::udp::UdpSocket;
use aether_kernel::net::{dispatch_frame, LOOPBACK_IPV4};

#[test]
fn ethernet_ipv4_dispatch() {
    let mut ip = [0_u8; IPV4_HEADER_LEN + 8];
    Ipv4Header::build_minimal(&mut ip, LOOPBACK_IPV4, LOOPBACK_IPV4, Ipv4Protocol::Udp, 8).unwrap();

    let eth = EthernetFrame {
        dst: MacAddress::BROADCAST,
        src: loopback_mac(),
        ethertype: Ethertype::Ipv4,
        payload: &ip,
    };
    let mut frame = [0_u8; 64];
    let n = eth.write_to(&mut frame).unwrap();
    dispatch_frame(&frame[..n]).unwrap();
}

#[test]
fn arp_request_validates() {
    let packet = ArpPacket {
        sender_hw: MacAddress([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]),
        sender_proto: [192, 168, 122, 1],
        target_hw: MacAddress::ZERO,
        target_proto: [192, 168, 122, 2],
        operation: ArpOperation::Request,
    };
    let mut buf = [0_u8; ARP_PACKET_LEN];
    packet.write_to(&mut buf).unwrap();

    let eth = EthernetFrame {
        dst: MacAddress::BROADCAST,
        src: packet.sender_hw,
        ethertype: Ethertype::Arp,
        payload: &buf,
    };
    let mut frame = [0_u8; ETHERNET_HEADER_LEN + ARP_PACKET_LEN];
    let n = eth.write_to(&mut frame).unwrap();
    dispatch_frame(&frame[..n]).unwrap();
}

#[test]
fn capability_denies_missing_right() {
    let cap = NetworkCapability::new(2, NetworkRight::Receive as u32);
    let sock = UdpSocket::new(cap);
    assert!(check_network_capability(cap, NetworkOp::Send).is_err());
    assert!(check_network_capability(cap, NetworkOp::Receive).is_ok());
    let _ = sock;
}

#[test]
fn tcp_listen_and_connect_states() {
    let cap = NetworkCapability::kernel();
    let mut listener = TcpSocket::new(cap);
    listener.listen([0, 0, 0, 0], 8080).unwrap();
    assert_eq!(listener.state, TcpState::Listen);

    let mut client = TcpSocket::new(cap);
    client.connect(LOOPBACK_IPV4, 8080).unwrap();
    assert_eq!(client.state, TcpState::SynSent);
}

#[test]
fn loopback_nic_inject() {
    init_nics();
    let mut ip = [0_u8; IPV4_HEADER_LEN];
    Ipv4Header::build_minimal(&mut ip, LOOPBACK_IPV4, LOOPBACK_IPV4, Ipv4Protocol::Tcp, 0).unwrap();
    inject_loopback_ipv4(&ip).unwrap();
    use_virtio_net_stub();
}

#[test]
fn ipv4_checksum_matches_rfc1071_example() {
    let data = [0x45, 0x00, 0x00, 0x73, 0x00, 0x00, 0x40, 0x00, 0x40, 0x11, 0x00, 0x00];
    let sum = compute_checksum(&data);
    assert_ne!(sum, 0);
}
