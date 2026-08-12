//! Bare-metal kernel entry for `x86_64-unknown-none`.

#![no_std]
#![no_main]

use aether_kernel::{arch, drivers, mm, sched, serial, syscall};
use aether_types::BootInfo;
use core::panic::PanicInfo;

/// Kernel entry called by the UEFI boot loader (System V AMD64: `RDI` = BootInfo).
#[no_mangle]
pub extern "sysv64" fn _start(boot_info: *const BootInfo) -> ! {
    serial::init();
    serial::write_str("Aether OS kernel started\r\n");

    if !boot_info.is_null() {
        // SAFETY: Boot loader guarantees BootInfo remains valid after handoff.
        let info = unsafe { &*boot_info };
        if info.is_valid() {
            drivers::set_boot_info(info);
            serial::write_str("BootInfo OK\r\n");
            mm::init(info);
        } else {
            serial::write_str("BootInfo invalid\r\n");
        }
    }

    drivers::register_builtin_drivers();
    drivers::init_drivers();

    arch::x86_64::gdt::init();
    arch::x86_64::idt::init();
    arch::x86_64::init_pic();
    arch::x86_64::register_irq_handlers();
    arch::x86_64::init_timer();

    sched::init();
    sched::spawn_worker_thread();
    syscall::init(sched::kernel_stack_top());
    aether_kernel::user::start_userland();

    serial::write_str("Aether OS M2: GDT/IDT/interrupts initialized\r\n");
    serial::write_str("Aether OS M5: syscalls initialized\r\n");
    log_driver_status();

    sched::start();
}

fn log_driver_status() {
    serial::write_str("[drivers] keyboard keys=");
    write_u32(aether_drv_keyboard::key_count());
    serial::write_str("\r\n");

    if let Some(dev) = aether_drv_storage::detected() {
        serial::write_str("[drivers] storage stub: block device detected\r\n");
        serial::write_str("  vendor=0x");
        write_hex16(dev.vendor_id);
        serial::write_str(" device=0x");
        write_hex16(dev.device_id);
        serial::write_str("\r\n");
    } else {
        serial::write_str("[drivers] storage stub: no block device\r\n");
    }

    if let Some(dev) = aether_drv_net::detected() {
        serial::write_str("[drivers] net stub: ");
        serial::write_str(dev.driver_name);
        serial::write_str(" detected\r\n");
    } else {
        serial::write_str("[drivers] net stub: no adapter\r\n");
    }

    let (devices, count) = drivers::enumerate();
    serial::write_str("[pci] enumerated ");
    write_u32(count as u32);
    serial::write_str(" devices\r\n");
    for pci in devices.iter().take(count) {
        serial::write_str("  ");
        write_hex16(pci.vendor_id);
        serial::write_str(":");
        write_hex16(pci.device_id);
        serial::write_str("\r\n");
    }
}

fn write_u32(mut value: u32) {
    if value == 0 {
        serial::write_str("0");
        return;
    }
    let mut buf = [0u8; 10];
    let mut index = buf.len();
    while value > 0 {
        index -= 1;
        buf[index] = b'0' + (value % 10) as u8;
        value /= 10;
    }
    serial::write_str(core::str::from_utf8(&buf[index..]).unwrap_or("?"));
}

fn write_hex16(value: u16) {
    serial::write_str("0x");
    write_hex_byte((value >> 8) as u8);
    write_hex_byte(value as u8);
}

fn write_hex_byte(value: u8) {
    serial::write_byte(hex_digit(value >> 4));
    serial::write_byte(hex_digit(value & 0x0F));
}

fn hex_digit(n: u8) -> u8 {
    if n < 10 {
        b'0' + n
    } else {
        b'a' + (n - 10)
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial::write_str("KERNEL PANIC\r\n");
    loop {
        core::hint::spin_loop();
    }
}
