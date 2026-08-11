//! Static driver registry with probe-then-init lifecycle.

use aether_drv_framebuffer as framebuffer;
use aether_drv_keyboard as keyboard;
use aether_drv_net as net;
use aether_drv_serial as serial;
use aether_drv_storage as storage;
use aether_types::{BootInfo, FramebufferInfo, SerialPortInfo};

use super::pci;

/// Result type for driver initialization.
pub type DriverResult = Result<(), DriverError>;

/// Driver initialization failure reason.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DriverError {
    /// [`DriverOps::probe`] returned false.
    ProbeFailed,
    /// [`DriverOps::init`] failed.
    InitFailed,
    /// Device or feature not supported on this platform.
    NotSupported,
}

/// Function-pointer vtable for a statically registered driver.
#[derive(Clone, Copy)]
pub struct DriverOps {
    /// Short driver name for diagnostics.
    pub name: &'static str,
    /// Returns `true` when hardware or boot metadata indicates the driver should load.
    pub probe: fn() -> bool,
    /// Initializes the driver after a successful probe.
    pub init: fn() -> DriverResult,
}

static mut BOOT_INFO: Option<BootInfoSnapshot> = None;

#[derive(Clone, Copy)]
struct BootInfoSnapshot {
    serial_port: SerialPortInfo,
    framebuffer: FramebufferInfo,
}

static mut REGISTRY: [Option<DriverOps>; 8] = [None; 8];
static mut REGISTRY_LEN: usize = 0;

/// Registers a driver in the static table (early boot, single-threaded).
pub fn register_driver(ops: DriverOps) {
    // SAFETY: Called sequentially during driver registration before init.
    unsafe {
        if REGISTRY_LEN < REGISTRY.len() {
            REGISTRY[REGISTRY_LEN] = Some(ops);
            REGISTRY_LEN += 1;
        }
    }
}

/// Stores boot metadata used by probe/init routines.
pub fn set_boot_info(info: &BootInfo) {
    // SAFETY: Single-threaded before driver init.
    unsafe {
        BOOT_INFO =
            Some(BootInfoSnapshot { serial_port: info.serial_port, framebuffer: info.framebuffer });
    }
}

fn boot_info() -> Option<BootInfoSnapshot> {
    // SAFETY: Read after set_boot_info during init.
    unsafe { BOOT_INFO }
}

fn probe_serial() -> bool {
    true
}

fn init_serial() -> DriverResult {
    if let Some(info) = boot_info() {
        serial::init(&info.serial_port);
    } else {
        serial::init_default();
    }
    Ok(())
}

fn probe_keyboard() -> bool {
    true
}

fn init_keyboard() -> DriverResult {
    keyboard::init();
    Ok(())
}

fn probe_framebuffer() -> bool {
    boot_info().map(|b| b.framebuffer.base != 0 && b.framebuffer.width != 0).unwrap_or(false)
}

fn init_framebuffer() -> DriverResult {
    let info = boot_info().ok_or(DriverError::ProbeFailed)?;
    if framebuffer::init(&info.framebuffer) {
        Ok(())
    } else {
        Err(DriverError::InitFailed)
    }
}

fn probe_storage() -> bool {
    storage::detected().is_some()
}

fn init_storage() -> DriverResult {
    if storage::init() {
        Ok(())
    } else {
        Err(DriverError::ProbeFailed)
    }
}

fn probe_net() -> bool {
    net::detected().is_some()
}

fn init_net() -> DriverResult {
    if net::init() {
        Ok(())
    } else {
        Err(DriverError::ProbeFailed)
    }
}

/// Registers built-in drivers and runs PCI enumeration before probing.
pub fn register_builtin_drivers() {
    let _ = pci::enumerate();

    register_driver(DriverOps { name: "serial", probe: probe_serial, init: init_serial });
    register_driver(DriverOps { name: "keyboard", probe: probe_keyboard, init: init_keyboard });
    register_driver(DriverOps {
        name: "framebuffer",
        probe: probe_framebuffer,
        init: init_framebuffer,
    });
    register_driver(DriverOps { name: "storage", probe: probe_storage, init: init_storage });
    register_driver(DriverOps { name: "net", probe: probe_net, init: init_net });
}

/// Probes and initializes all registered drivers in registration order.
pub fn init_drivers() {
    // SAFETY: Single-threaded early boot.
    unsafe {
        for entry in REGISTRY.iter().take(REGISTRY_LEN).flatten() {
            if (entry.probe)() {
                let _ = (entry.init)();
            }
        }
    }
}
