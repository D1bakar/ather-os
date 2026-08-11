//! GDT layout helpers — host-testable encoding logic with no CPU instructions.

/// Size of a standard 8-byte GDT entry.
pub const GDT_ENTRY_SIZE: usize = 8;

/// Number of GDT entries: null, kernel code/data, two user placeholders, TSS (2).
pub const GDT_ENTRY_COUNT: usize = 7;

/// GDT index of the null descriptor.
#[allow(dead_code)]
pub const NULL_INDEX: u8 = 0;
/// GDT index of the 64-bit kernel code segment.
pub const KERNEL_CODE_INDEX: u8 = 1;
/// GDT index of the kernel data segment.
pub const KERNEL_DATA_INDEX: u8 = 2;
/// GDT index of the 64-bit user data segment (ring 3, SYSRET SS = STAR[63:48]+8).
pub const USER_DATA_INDEX: u8 = 3;
/// GDT index of the 64-bit user code segment (ring 3, SYSRET CS = STAR[63:48]+16).
pub const USER_CODE_INDEX: u8 = 4;

/// Kernel 64-bit code segment selector (GDT index 1, RPL 0).
pub const KERNEL_CODE_SELECTOR: u16 = (KERNEL_CODE_INDEX as u16) << 3;
/// Kernel data segment selector (GDT index 2, RPL 0).
pub const KERNEL_DATA_SELECTOR: u16 = (KERNEL_DATA_INDEX as u16) << 3;
/// User data segment selector (GDT index 3, RPL 3).
pub const USER_DATA_SELECTOR: u16 = ((USER_DATA_INDEX as u16) << 3) | 3;
/// User code segment selector (GDT index 4, RPL 3).
pub const USER_CODE_SELECTOR: u16 = ((USER_CODE_INDEX as u16) << 3) | 3;
/// TSS segment selector (GDT index 5, RPL 0).
#[allow(dead_code)]
pub const TSS_SELECTOR: u16 = (TSS_INDEX as u16) << 3;

/// GDT index of the TSS descriptor (spans this entry and the next).
pub const TSS_INDEX: u8 = 5;

/// `LGDT` pseudo-descriptor: limit (size − 1) and linear base of the GDT.
#[repr(C, packed)]
pub struct DescriptorTablePointer {
    /// Size of the GDT in bytes minus one.
    pub limit: u16,
    /// Linear address of the first GDT entry.
    pub base: u64,
}

/// Builds a 64-bit long-mode kernel code segment descriptor.
pub const fn kernel_code_descriptor() -> u64 {
    // Present | DPL0 | Code | Executable | Readable | L=1 | G=1
    0x00AF_9A00_0000_FFFF
}

/// Builds a 64-bit long-mode kernel data segment descriptor.
pub const fn kernel_data_descriptor() -> u64 {
    // Present | DPL0 | Data | Writable | G=1
    0x00CF_9200_0000_FFFF
}

/// Builds a 64-bit long-mode user code segment descriptor (ring 3).
pub const fn user_code_descriptor() -> u64 {
    // Present | DPL3 | Code | Executable | Readable | L=1 | G=1
    0x00AF_FA00_0000_FFFF
}

/// Builds a 64-bit long-mode user data segment descriptor (ring 3).
pub const fn user_data_descriptor() -> u64 {
    // Present | DPL3 | Data | Writable | G=1
    0x00CF_F200_0000_FFFF
}

/// Computes the byte limit operand for `LGDT` / `LLDT` (table size − 1).
pub const fn table_limit_bytes(entry_count: usize, entry_size: usize) -> u16 {
    (entry_count * entry_size - 1) as u16
}

/// Encodes a 64-bit available TSS system-segment descriptor (16 bytes total).
///
/// Returns `(low_qword, high_qword)` for consecutive GDT slots.
pub const fn tss_descriptor(base: u64, limit: u32) -> (u64, u64) {
    let limit_low = (limit & 0xFFFF) as u64;
    let limit_high = ((limit >> 16) & 0xF) as u64;
    let base_low = base & 0xFFFF;
    let base_mid = (base >> 16) & 0xFF;
    let base_high = (base >> 24) & 0xFF;
    let base_upper = base >> 32;

    // Present | DPL0 | System | Available 64-bit TSS (type 9).
    const ACCESS: u64 = 0x89;
    let flags = limit_high;

    let low = limit_low
        | (base_low << 16)
        | (base_mid << 32)
        | (ACCESS << 40)
        | (flags << 48)
        | (base_high << 56);

    (low, base_upper)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_limit_for_seven_entries() {
        assert_eq!(table_limit_bytes(7, 8), 55);
    }

    #[test]
    fn tss_descriptor_access_byte_is_available_64bit_tss() {
        let (low, _) = tss_descriptor(0x1000, 103);
        let access = ((low >> 40) & 0xFF) as u8;
        assert_eq!(access, 0x89);
    }
}
