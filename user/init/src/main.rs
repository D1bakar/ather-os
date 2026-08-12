//! Aether OS init process — first ring-3 user task (M6).

#![cfg_attr(not(feature = "host"), no_std)]
#![cfg_attr(not(feature = "host"), no_main)]

use aether_rt::{write, StdFd};

/// Init entry for host builds.
#[cfg(feature = "host")]
fn main() -> ! {
    init_main()
}

/// Bare-metal entry point.
#[cfg(not(feature = "host"))]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    init_main()
}

#[cfg(not(feature = "host"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

fn init_main() -> ! {
    let _ = write(StdFd::Stdout.as_i32(), b"Aether init started\n");
    idle_forever();
}

fn idle_forever() -> ! {
    loop {
        core::hint::spin_loop();
    }
}
