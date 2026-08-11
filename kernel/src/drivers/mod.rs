//! Driver registration, probing, and initialization.

#![cfg(not(feature = "host-stub"))]

pub mod pci;
mod registry;

pub use pci::{enumerate, PciDevice};
pub use registry::{
    init_drivers, register_builtin_drivers, register_driver, set_boot_info, DriverError, DriverOps,
    DriverResult,
};

/// Core driver lifecycle: probe hardware, then initialize.
pub trait Driver {
    /// Short driver name for diagnostics.
    fn name(&self) -> &'static str;
    /// Returns `true` when this driver should bind to detected hardware.
    fn probe(&self) -> bool;
    /// Initializes the driver after a successful probe.
    fn init(&mut self) -> DriverResult;
}
