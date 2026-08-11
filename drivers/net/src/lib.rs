//! Network driver stub — probes for Intel e1000 or virtio-net on PCI.

#![no_std]
#![deny(missing_docs)]

/// Intel vendor ID.
pub const INTEL_VENDOR_ID: u16 = 0x8086;

/// QEMU default Intel 82540EM (e1000) device ID.
pub const E1000_DEVICE_ID: u16 = 0x100E;

/// Red Hat / Virtio vendor ID.
pub const VIRTIO_VENDOR_ID: u16 = 0x1AF4;

/// Virtio network device ID.
pub const VIRTIO_NET_DEVICE_ID: u16 = 0x1000;

/// Describes a detected network adapter (stub metadata only in M9).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NetDevice {
    /// PCI vendor ID.
    pub vendor_id: u16,
    /// PCI device ID.
    pub device_id: u16,
    /// PCI bus number.
    pub bus: u8,
    /// PCI device number.
    pub device: u8,
    /// PCI function number.
    pub function: u8,
    /// Human-readable driver name selected for this device.
    pub driver_name: &'static str,
}

static mut DETECTED: Option<NetDevice> = None;

/// Probes PCI for e1000 or virtio-net.
pub fn probe(vendor_id: u16, device_id: u16, bus: u8, device: u8, function: u8) -> bool {
    let (matched, name) = match (vendor_id, device_id) {
        (INTEL_VENDOR_ID, E1000_DEVICE_ID) => (true, "e1000"),
        (VIRTIO_VENDOR_ID, VIRTIO_NET_DEVICE_ID) => (true, "virtio-net"),
        _ => (false, ""),
    };

    if matched {
        // SAFETY: Single-threaded early boot.
        unsafe {
            DETECTED =
                Some(NetDevice { vendor_id, device_id, bus, device, function, driver_name: name });
        }
        return true;
    }
    false
}

/// Returns the device found during [`probe`], if any.
pub fn detected() -> Option<NetDevice> {
    // SAFETY: Read-only during early boot.
    unsafe { DETECTED }
}

/// Initializes the network subsystem (stub — detection only in M9).
pub fn init() -> bool {
    detected().is_some()
}
