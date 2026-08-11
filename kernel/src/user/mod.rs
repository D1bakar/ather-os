//! User-space bootstrap stub (M6).
//!
//! Parses embedded init ELF bytes and logs load metadata. Ring-3 execution
//! remains blocked until M5 syscall dispatch and user page tables exist.

mod embedded;

use crate::elf::{load_elf_stub, ElfError};
use crate::serial;

/// Kernel-thread stub: validate embedded init ELF and report status on serial.
pub fn init_userspace_stub() {
    serial::write_str("M6: userspace init stub\r\n");

    let image = embedded::init_elf();
    if image.is_empty() {
        serial::write_str("M6: no embedded init ELF (run scripts/build-user first)\r\n");
        serial::write_str("M6: ring-3 transition blocked (M5 paging/syscalls)\r\n");
        return;
    }

    serial::write_str("M6: parsing embedded init ELF...\r\n");
    match load_elf_stub(image) {
        Ok(entry) => {
            serial::write_str("M6: ELF OK entry=0x");
            write_hex_u64(entry);
            serial::write_str("\r\n");
            serial::write_str("M6: load deferred — paging/userspace incomplete\r\n");
        }
        Err(err) => {
            serial::write_str("M6: ELF parse failed: ");
            serial::write_str(err.as_str());
            serial::write_str("\r\n");
        }
    }
}

/// Formats ELF errors for host tests.
#[cfg(feature = "host-stub")]
pub fn describe_elf_error(err: ElfError) -> &'static str {
    err.as_str()
}

fn write_hex_u64(mut value: u64) {
    if value == 0 {
        serial::write_byte(b'0');
        return;
    }
    let mut started = false;
    for shift in (0..16).rev() {
        let nibble = ((value >> (shift * 4)) & 0xF) as u8;
        if nibble != 0 || started {
            started = true;
            let ch = if nibble < 10 { b'0' + nibble } else { b'a' + (nibble - 10) };
            serial::write_byte(ch);
        }
    }
}
