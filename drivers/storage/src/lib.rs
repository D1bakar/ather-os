//! Block storage driver stub — probes for virtio-blk on PCI.

#![no_std]
#![deny(missing_docs)]

/// Red Hat / Virtio vendor ID.
pub const VIRTIO_VENDOR_ID: u16 = 0x1AF4;

/// Virtio block device ID.
pub const VIRTIO_BLK_DEVICE_ID: u16 = 0x1001;

/// Intel vendor ID (legacy IDE/AHCI in QEMU).
pub const INTEL_VENDOR_ID: u16 = 0x8086;

/// Describes a detected block device (stub metadata only in M9).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StorageDevice {
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
}

static mut DETECTED: Option<StorageDevice> = None;

/// Probes PCI for a virtio-blk or placeholder storage controller.
pub fn probe(vendor_id: u16, device_id: u16, bus: u8, device: u8, function: u8) -> bool {
    let is_virtio_blk = vendor_id == VIRTIO_VENDOR_ID && device_id == VIRTIO_BLK_DEVICE_ID;
    if is_virtio_blk {
        // SAFETY: Single-threaded early boot.
        unsafe {
            DETECTED = Some(StorageDevice { vendor_id, device_id, bus, device, function });
        }
        return true;
    }
    false
}

/// Returns the device found during [`probe`], if any.
pub fn detected() -> Option<StorageDevice> {
    // SAFETY: Read-only during early boot.
    unsafe { DETECTED }
}

/// Initializes the storage subsystem (stub — logs detection only).
pub fn init() -> bool {
    detected().is_some()
}
