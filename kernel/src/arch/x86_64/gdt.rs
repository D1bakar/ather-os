//! Global Descriptor Table (GDT) for x86_64 long mode.
//!
//! # Long-mode segmentation
//!
//! In 64-bit mode the CPU largely ignores segment bases and limits; flat kernel
//! code/data segments suffice. A 64-bit TSS entry is still required before an
//! IDT can reference IST stacks (planned follow-on work).
//!
//! # Initialization sequence
//!
//! 1. Fill static GDT entries (null, kernel code/data, TSS placeholder).
//! 2. Execute `LGDT` with a [`layout::DescriptorTablePointer`].
//! 3. Reload data segments (`DS`, `ES`, `SS`, `FS`, `GS`) and perform a far
//!    return to reload `CS`.
//!
//! # Per-CPU considerations
//!
//! Each logical CPU needs its own TSS because IST stack pointers are per-core.
//! This module installs a **BSP-only** static TSS. When SMP is added, call
//! [`init`] on every AP after allocating a per-CPU TSS and updating GDT entries
//! 5–6 (or use a per-CPU GDT copy). Do not share IST pointers across cores.

pub mod layout;

pub use layout::{
    kernel_code_descriptor, kernel_data_descriptor, user_code_descriptor, user_data_descriptor,
    DescriptorTablePointer, GDT_ENTRY_COUNT, KERNEL_CODE_SELECTOR, KERNEL_DATA_SELECTOR,
    TSS_INDEX, TSS_SELECTOR, USER_CODE_SELECTOR, USER_DATA_SELECTOR,
};

#[cfg(all(not(feature = "host-stub"), target_arch = "x86_64"))]
use layout::{table_limit_bytes, tss_descriptor};

/// Minimal 64-bit Task State Segment (TSS) for IST placeholder.
///
/// IST slots are zero until the interrupt subsystem wires them up.
#[repr(C, packed)]
pub struct TaskStateSegment {
    _reserved0: u32,
    /// Stack pointer for ring-0 interrupts when entering from ring-3.
    pub rsp0: u64,
    /// Ring-0 stack for privilege level 1 (unused until rings are enabled).
    pub rsp1: u64,
    /// Ring-0 stack for privilege level 2 (unused until rings are enabled).
    pub rsp2: u64,
    _reserved1: u64,
    /// Interrupt Stack Table entries (IST1–IST7) for exception stacks.
    pub ist1: u64,
    /// IST2 stack pointer (placeholder).
    pub ist2: u64,
    /// IST3 stack pointer (placeholder).
    pub ist3: u64,
    /// IST4 stack pointer (placeholder).
    pub ist4: u64,
    /// IST5 stack pointer (placeholder).
    pub ist5: u64,
    /// IST6 stack pointer (placeholder).
    pub ist6: u64,
    /// IST7 stack pointer (placeholder).
    pub ist7: u64,
    _reserved2: u64,
    _reserved3: u16,
    /// Byte offset of the I/O permission bitmap; set to TSS size when unused.
    pub iomap_base: u16,
}

impl Default for TaskStateSegment {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskStateSegment {
    /// Returns a zeroed TSS with `iomap_base` pointing past the structure.
    pub const fn new() -> Self {
        Self {
            _reserved0: 0,
            rsp0: 0,
            rsp1: 0,
            rsp2: 0,
            _reserved1: 0,
            ist1: 0,
            ist2: 0,
            ist3: 0,
            ist4: 0,
            ist5: 0,
            ist6: 0,
            ist7: 0,
            _reserved2: 0,
            _reserved3: 0,
            iomap_base: core::mem::size_of::<Self>() as u16,
        }
    }
}

/// Byte size of the in-memory TSS (used as the TSS descriptor limit).
pub const fn tss_size() -> u32 {
    core::mem::size_of::<TaskStateSegment>() as u32
}

#[cfg(all(not(feature = "host-stub"), target_arch = "x86_64"))]
#[repr(C, align(8))]
struct Gdt {
    entries: [u64; GDT_ENTRY_COUNT],
}

#[cfg(all(not(feature = "host-stub"), target_arch = "x86_64"))]
static mut GDT: Gdt = Gdt { entries: [0; GDT_ENTRY_COUNT] };

#[cfg(all(not(feature = "host-stub"), target_arch = "x86_64"))]
static mut TSS: TaskStateSegment = TaskStateSegment::new();

/// Loads the GDT and reloads segment registers for the BSP.
///
/// Must be called once during early boot before installing an IDT. On SMP
/// systems, APs need their own TSS/GDT setup when they come online.
#[cfg(all(not(feature = "host-stub"), target_arch = "x86_64"))]
pub fn init() {
    // SAFETY: Single-threaded BSP early boot; no concurrent GDT access yet.
    unsafe {
        populate_gdt();
        let pointer = gdt_pointer();
        load_gdt(&pointer);
        reload_segments();
        load_task_register();
    }
}

/// Updates the BSP TSS ring-0 stack used when entering from ring 3.
#[cfg(all(not(feature = "host-stub"), target_arch = "x86_64"))]
pub fn set_kernel_stack(stack_top: u64) {
    // SAFETY: BSP-only early init.
    unsafe {
        TSS.rsp0 = stack_top;
    }
}

#[cfg(all(not(feature = "host-stub"), target_arch = "x86_64"))]
unsafe fn populate_gdt() {
    GDT.entries[layout::NULL_INDEX as usize] = 0;
    GDT.entries[1] = kernel_code_descriptor();
    GDT.entries[2] = kernel_data_descriptor();
    GDT.entries[layout::USER_DATA_INDEX as usize] = user_data_descriptor();
    GDT.entries[layout::USER_CODE_INDEX as usize] = user_code_descriptor();

    let tss_addr = core::ptr::addr_of!(TSS) as u64;
    let (tss_low, tss_high) = tss_descriptor(tss_addr, tss_size());
    GDT.entries[TSS_INDEX as usize] = tss_low;
    GDT.entries[TSS_INDEX as usize + 1] = tss_high;
}

#[cfg(all(not(feature = "host-stub"), target_arch = "x86_64"))]
unsafe fn gdt_pointer() -> DescriptorTablePointer {
    DescriptorTablePointer {
        limit: table_limit_bytes(GDT_ENTRY_COUNT, layout::GDT_ENTRY_SIZE),
        base: core::ptr::addr_of!(GDT) as u64,
    }
}

#[cfg(all(not(feature = "host-stub"), target_arch = "x86_64"))]
unsafe fn load_gdt(pointer: &DescriptorTablePointer) {
    // SAFETY: `pointer` references a valid, fully initialized GDT for the BSP.
    core::arch::asm!(
        "lgdt [{0}]",
        in(reg) pointer,
        options(readonly, nostack, preserves_flags)
    );
}

#[cfg(all(not(feature = "host-stub"), target_arch = "x86_64"))]
unsafe fn load_task_register() {
    let tss_sel = u64::from(TSS_SELECTOR);
    // SAFETY: TSS descriptor is present in the GDT loaded above.
    core::arch::asm!(
        "ltr {0:x}",
        in(reg) tss_sel,
        options(nomem, nostack, preserves_flags)
    );
}

#[cfg(all(not(feature = "host-stub"), target_arch = "x86_64"))]
unsafe fn reload_segments() {
    let data_sel = u64::from(KERNEL_DATA_SELECTOR);
    let code_sel = u64::from(KERNEL_CODE_SELECTOR);

    // SAFETY: Selectors refer to present GDT entries installed above.
    core::arch::asm!(
        "mov ds, {ds}",
        "mov es, {ds}",
        "mov ss, {ds}",
        "mov fs, {zero}",
        "mov gs, {zero}",
        "push {cs}",
        "lea {tmp}, [rip + 2f]",
        "push {tmp}",
        "retfq",
        "2:",
        ds = in(reg) data_sel,
        cs = in(reg) code_sel,
        zero = in(reg) 0_u64,
        tmp = lateout(reg) _,
        options(nomem)
    );
}

#[cfg(test)]
mod tests {
    use super::layout::*;
    use super::*;

    #[test]
    fn kernel_segment_selectors_match_gdt_indices() {
        assert_eq!(KERNEL_CODE_SELECTOR, (KERNEL_CODE_INDEX as u16) << 3);
        assert_eq!(KERNEL_DATA_SELECTOR, (KERNEL_DATA_INDEX as u16) << 3);
        assert_eq!(TSS_SELECTOR, (TSS_INDEX as u16) << 3);
    }

    #[test]
    fn kernel_descriptors_are_present() {
        assert_ne!(kernel_code_descriptor(), 0);
        assert_ne!(kernel_data_descriptor(), 0);
        assert_eq!(kernel_code_descriptor() & (1 << 47), 1 << 47);
        assert_eq!(kernel_data_descriptor() & (1 << 47), 1 << 47);
    }

    #[test]
    fn user_segment_descriptors_are_dpl3() {
        use layout::{user_code_descriptor, user_data_descriptor};
        assert_eq!((user_data_descriptor() >> 45) & 0b11, 0b11);
        assert_eq!((user_code_descriptor() >> 45) & 0b11, 0b11);
    }

    #[test]
    fn lgdt_limit_is_size_minus_one() {
        assert_eq!(table_limit_bytes(GDT_ENTRY_COUNT, GDT_ENTRY_SIZE), 55);
    }

    #[test]
    fn tss_descriptor_encodes_limit_and_base() {
        let base = 0x0000_1234_5678_9000;
        let limit = tss_size();
        let (low, high) = tss_descriptor(base, limit);

        assert_ne!(low, 0);
        assert_eq!(high, base >> 32);

        let encoded_limit = (low & 0xFFFF) | (((low >> 48) & 0xF) << 16);
        assert_eq!(encoded_limit, limit as u64);
    }

    #[test]
    fn tss_default_iomap_base_is_structure_size() {
        let tss = TaskStateSegment::new();
        let iomap_base = tss.iomap_base;
        assert_eq!(iomap_base, core::mem::size_of::<TaskStateSegment>() as u16);
    }
}
