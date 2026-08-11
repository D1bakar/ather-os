//! PCI configuration space enumeration stub for QEMU x86_64.

use aether_drv_hal::{inl, outl};
use aether_drv_net as net;
use aether_drv_storage as storage;

const CONFIG_ADDRESS: u16 = 0xCF8;
const CONFIG_DATA: u16 = 0xCFC;

/// Maximum PCI devices returned by [`enumerate`].
pub const MAX_PCI_DEVICES: usize = 32;

/// One PCI function discovered during enumeration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PciDevice {
    /// PCI bus number.
    pub bus: u8,
    /// PCI device number (0–31).
    pub device: u8,
    /// PCI function number (0–7).
    pub function: u8,
    /// Vendor ID.
    pub vendor_id: u16,
    /// Device ID.
    pub device_id: u16,
    /// Class code (base class in bits 23:16).
    pub class_code: u8,
    /// Subclass code (bits 15:8).
    pub subclass: u8,
}

/// Scans bus 0 devices 0–31, function 0, and returns up to [`MAX_PCI_DEVICES`] entries.
pub fn enumerate() -> ([PciDevice; MAX_PCI_DEVICES], usize) {
    let mut devices = [PciDevice {
        bus: 0,
        device: 0,
        function: 0,
        vendor_id: 0,
        device_id: 0,
        class_code: 0,
        subclass: 0,
    }; MAX_PCI_DEVICES];
    let mut count = 0usize;

    for dev in 0u8..32 {
        if let Some(pci) = read_device(0, dev, 0) {
            if count < MAX_PCI_DEVICES {
                devices[count] = pci;
                count += 1;
            }

            let _ = storage::probe(pci.vendor_id, pci.device_id, pci.bus, pci.device, pci.function);
            let _ = net::probe(pci.vendor_id, pci.device_id, pci.bus, pci.device, pci.function);
        }
    }

    (devices, count)
}

fn read_device(bus: u8, device: u8, function: u8) -> Option<PciDevice> {
    let vendor_id = pci_config_read16(bus, device, function, 0x00);
    if vendor_id == 0xFFFF {
        return None;
    }

    let device_id = pci_config_read16(bus, device, function, 0x02);
    let class_rev = pci_config_read32(bus, device, function, 0x08);

    Some(PciDevice {
        bus,
        device,
        function,
        vendor_id,
        device_id,
        class_code: ((class_rev >> 16) & 0xFF) as u8,
        subclass: ((class_rev >> 8) & 0xFF) as u8,
    })
}

fn pci_config_read16(bus: u8, device: u8, function: u8, offset: u8) -> u16 {
    let value = pci_config_read32(bus, device, function, offset & 0xFC);
    let shift = (offset & 2) * 8;
    ((value >> shift) & 0xFFFF) as u16
}

fn pci_config_read32(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    let address = 0x8000_0000u32
        | u32::from(bus) << 16
        | u32::from(device & 0x1F) << 11
        | u32::from(function & 0x07) << 8
        | u32::from(offset & 0xFC);

    // SAFETY: Standard x86 PCI config mechanism I/O ports.
    unsafe {
        outl(CONFIG_ADDRESS, address);
        inl(CONFIG_DATA)
    }
}
