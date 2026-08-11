//! Aether OS init process — first user task (M6 stub).
//!
//! Host builds run an interactive demo that delegates to the shell binary.
//! Bare-metal builds emit serial diagnostics via the write syscall and idle
//! until M5 ring-3 transition is implemented.

#![cfg_attr(not(feature = "host"), no_std)]
#![cfg_attr(not(feature = "host"), no_main)]

use aether_rt::{print, println, write, StdFd};

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
    banner();
    launch_shell_stub();
    idle_forever();
}

fn banner() {
    println("Aether init: starting (M6 stub)");
    let pid = aether_rt::getpid();
    print("  pid=");
    write_decimal(pid);
    println("");
}

fn launch_shell_stub() {
    println("Aether init: shell launch stub (M5 exec not implemented)");
    #[cfg(feature = "host")]
    println("  run separately: cargo run -p aether-shell --bin shell");
    #[cfg(not(feature = "host"))]
    println("  embedded shell ELF pending kernel loader");
}

fn idle_forever() -> ! {
    println("Aether init: entering idle loop");
    loop {
        #[cfg(not(feature = "host"))]
        {
            core::hint::spin_loop();
        }
        #[cfg(feature = "host")]
        {
            std::thread::sleep(std::time::Duration::from_secs(3600));
        }
    }
}

fn write_decimal(mut n: i32) {
    if n == 0 {
        let _ = write(StdFd::Stdout.as_i32(), b"0");
        return;
    }
    if n < 0 {
        let _ = write(StdFd::Stdout.as_i32(), b"-");
        n = -n;
    }
    let mut buf = [0u8; 12];
    let mut i = buf.len();
    while n > 0 {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    let _ = write(StdFd::Stdout.as_i32(), &buf[i..]);
}
