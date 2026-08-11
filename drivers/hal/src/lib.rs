//! Port I/O and low-level helpers shared by in-kernel drivers.

#![no_std]
#![deny(missing_docs)]

/// Reads one byte from `port`.
///
/// # Safety
///
/// The port must be valid for the running platform.
#[inline]
pub unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!(
        "in al, dx",
        out("al") value,
        in("dx") port,
        options(nomem, nostack, preserves_flags)
    );
    value
}

/// Writes one byte to `port`.
///
/// # Safety
///
/// The port must be valid for the running platform.
#[inline]
pub unsafe fn outb(port: u16, value: u8) {
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nomem, nostack, preserves_flags)
    );
}

/// Reads one 32-bit value from `port`.
///
/// # Safety
///
/// The port must be valid for the running platform.
#[inline]
pub unsafe fn inl(port: u16) -> u32 {
    let value: u32;
    core::arch::asm!(
        "in eax, dx",
        out("eax") value,
        in("dx") port,
        options(nomem, nostack, preserves_flags)
    );
    value
}

/// Writes one 32-bit value to `port`.
///
/// # Safety
///
/// The port must be valid for the running platform.
#[inline]
pub unsafe fn outl(port: u16, value: u32) {
    core::arch::asm!(
        "out dx, eax",
        in("dx") port,
        in("eax") value,
        options(nomem, nostack, preserves_flags)
    );
}
