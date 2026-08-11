//! Virtual and hardware NIC abstractions.

pub mod e1000;
pub mod loopback;
pub mod virtio;

use crate::net::ethernet::{EthernetFrame, Ethertype, MacAddress, ETHERNET_HEADER_LEN};
use aether_types::{AetherError, AetherResult, ErrorCode};

/// Identifies a registered network interface.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NicId(pub u8);

/// Kind of network interface.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NicKind {
    /// Software loopback (`lo`).
    Loopback,
    /// Intel e1000 placeholder (QEMU `-device e1000`).
    E1000,
    /// VirtIO-net MMIO placeholder (QEMU `-device virtio-net-pci`).
    VirtioNet,
}

/// Trait implemented by each network interface driver stub.
pub trait Nic {
    /// Returns the interface identifier.
    fn id(&self) -> NicId;
    /// Returns the interface kind.
    fn kind(&self) -> NicKind;
    /// Returns the interface MAC address.
    fn mac(&self) -> MacAddress;
    /// Polls for one inbound frame into `buf`, returning byte count.
    fn poll_rx(&mut self, buf: &mut [u8]) -> AetherResult<usize>;
    /// Queues one outbound frame (stub — may discard).
    fn send(&mut self, frame: &[u8]) -> AetherResult<usize>;
}

/// Maximum number of statically registered NIC stubs.
#[allow(dead_code)]
const MAX_NICS: usize = 2;

struct NicRegistry {
    loopback: loopback::LoopbackNic,
    hw: HwNicSlot,
}

enum HwNicSlot {
    None,
    E1000(e1000::E1000Stub),
    Virtio(virtio::VirtioNetStub),
}

impl NicRegistry {
    const fn new() -> Self {
        Self { loopback: loopback::LoopbackNic::new(NicId(0)), hw: HwNicSlot::None }
    }
}

static mut NIC_REGISTRY: NicRegistry = NicRegistry::new();

/// Initializes loopback and selects a hardware NIC placeholder for QEMU.
pub fn init_nics() {
    // SAFETY: called once from single-threaded boot before interrupts.
    unsafe {
        (*core::ptr::addr_of_mut!(NIC_REGISTRY)).hw = select_hw_nic();
    }
}

fn select_hw_nic() -> HwNicSlot {
    #[cfg(all(not(feature = "host-stub"), feature = "bare-metal"))]
    {
        if let Some(dev) = aether_drv_net::detected() {
            if dev.driver_name == "virtio-net" {
                return HwNicSlot::Virtio(virtio::VirtioNetStub::new(NicId(1)));
            }
        }
    }
    HwNicSlot::E1000(e1000::E1000Stub::new(NicId(1)))
}

/// Polls all registered interfaces (loopback first).
pub fn poll_nics() {
    let mut buf = [0_u8; 1518];
    // SAFETY: boot-time single CPU; no concurrent NIC access yet.
    unsafe {
        let registry = core::ptr::addr_of_mut!(NIC_REGISTRY);
        let _ = (*registry).loopback.poll_rx(&mut buf);
        match &mut (*registry).hw {
            HwNicSlot::E1000(nic) => {
                let _ = nic.poll_rx(&mut buf);
            }
            HwNicSlot::Virtio(nic) => {
                let _ = nic.poll_rx(&mut buf);
            }
            HwNicSlot::None => {}
        }
    }
}

/// Sends a frame on the loopback interface (stub TX path).
pub fn loopback_send(frame: &[u8]) -> AetherResult<usize> {
    // SAFETY: single-threaded stub path.
    unsafe { (*core::ptr::addr_of_mut!(NIC_REGISTRY)).loopback.send(frame) }
}

/// Builds a loopback IPv4 Ethernet frame around `payload` and injects it locally.
pub fn inject_loopback_ipv4(payload: &[u8]) -> AetherResult<()> {
    let eth = EthernetFrame {
        dst: MacAddress([0x02; 6]),
        src: MacAddress([0x02; 6]),
        ethertype: Ethertype::Ipv4,
        payload,
    };
    let mut frame = [0_u8; 1518];
    let n = eth.write_to(&mut frame)?;
    loopback_send(&frame[..n])?;
    Ok(())
}

/// Returns the loopback NIC MAC.
#[must_use]
pub fn loopback_mac() -> MacAddress {
    // SAFETY: read-only after init.
    unsafe { (*core::ptr::addr_of_mut!(NIC_REGISTRY)).loopback.mac() }
}

/// Switches the hardware NIC placeholder to VirtIO-net (testing hook).
pub fn use_virtio_net_stub() {
    // SAFETY: test/boot single-threaded.
    unsafe {
        (*core::ptr::addr_of_mut!(NIC_REGISTRY)).hw =
            HwNicSlot::Virtio(virtio::VirtioNetStub::new(NicId(1)));
    }
}

/// Returns an error if `frame` is shorter than the Ethernet header.
pub fn validate_frame_len(frame: &[u8]) -> AetherResult<()> {
    if frame.len() < ETHERNET_HEADER_LEN {
        Err(AetherError::new(ErrorCode::InvalidArgument))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::ipv4::{Ipv4Header, Ipv4Protocol};

    #[test]
    fn loopback_inject_roundtrip() {
        init_nics();
        let mut ip_buf = [0_u8; 28];
        Ipv4Header::build_minimal(
            &mut ip_buf,
            [127, 0, 0, 1],
            [127, 0, 0, 1],
            Ipv4Protocol::Udp,
            8,
        )
        .unwrap();
        inject_loopback_ipv4(&ip_buf).unwrap();
    }
}
