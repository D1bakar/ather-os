//! Bare-metal kernel entry for `x86_64-unknown-none`.

#![no_std]
#![no_main]

use aether_types::BootInfo;
use core::panic::PanicInfo;

mod serial;

/// Kernel entry called by the UEFI boot loader (System V AMD64: `RDI` = BootInfo).
#[no_mangle]
pub extern "sysv64" fn _start(boot_info: *const BootInfo) -> ! {
    serial::init();
    serial::write_str("Aether OS kernel started\r\n");

    if !boot_info.is_null() {
        // SAFETY: Boot loader guarantees BootInfo remains valid after handoff.
        let info = unsafe { &*boot_info };
        if info.is_valid() {
            serial::write_str("BootInfo OK\r\n");
        } else {
            serial::write_str("BootInfo invalid\r\n");
        }
    }

    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial::write_str("KERNEL PANIC\r\n");
    loop {
        core::hint::spin_loop();
    }
}
