//! Early console — delegates to the `aether-drv-serial` driver crate.

pub use aether_drv_serial::{init, init_default, write_byte, write_str};

/// Initializes COM1 using boot-loader serial metadata when available.
pub fn init_from_port(port: u16) {
    init(&aether_types::SerialPortInfo { port, baud_rate: 0 });
}
