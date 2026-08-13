//! x86_64 four-level paging: identity map + higher-half direct map.
//!
//! Maps the first gibibyte of physical RAM with 2 MiB huge pages, then applies
//! 4 KiB mappings with W^X flags over the kernel image (.text RX, .rodata/.data
//! RW|NX). Enables the NX bit in `EFER` before loading `CR3`.

use crate::mm::{frame, HEAP_SIZE, HEAP_VIRTUAL_START};
use x86_64::registers::control::{Cr3, Cr3Flags, Efer, EferFlags};
use x86_64::structures::paging::page_table::{PageTable, PageTableEntry, PageTableFlags};
use x86_64::structures::paging::{PageSize, PhysFrame, Size2MiB, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

/// Physical RAM covered by the bootstrap 2 MiB map (1 GiB).
const MAP_PHYS_BYTES: u64 = 1024 * 1024 * 1024;

/// PML4 index for the higher-half direct map (`0xFFFF_8000_0000_0000`).
const HIGHER_HALF_PML4_INDEX: usize = 256;

static mut PML4: PageTable = PageTable::new();
static mut PDPT_IDENTITY: PageTable = PageTable::new();
static mut PDPT_HIGHER: PageTable = PageTable::new();
static mut PD_IDENTITY: PageTable = PageTable::new();
static mut PD_HIGHER: PageTable = PageTable::new();
static mut KERNEL_CR3: u64 = 0;

/// Builds page tables, enables NX, and loads CR3.
pub fn init() {
    // SAFETY: Static page-table storage is exclusively accessed during BSP init.
    unsafe {
        build_huge_page_maps();
        enable_nxe();
        let pml4_frame =
            PhysFrame::from_start_address(PhysAddr::new(core::ptr::addr_of!(PML4) as u64))
                .expect("PML4 frame aligned");
        Cr3::write(pml4_frame, Cr3Flags::empty());
        KERNEL_CR3 = pml4_frame.start_address().as_u64();
        apply_kernel_wx();
        map_heap_region();
    }
    crate::serial::write_str("  paging: CR3 loaded, W^X applied\r\n");
}

/// Returns the bootstrap kernel page-table root (physical CR3 value).
#[cfg(not(feature = "host-stub"))]
#[must_use]
pub fn kernel_cr3() -> u64 {
    // SAFETY: Written once during `init` before user tasks run.
    unsafe { KERNEL_CR3 }
}

unsafe fn build_huge_page_maps() {
    let pml4 = &mut *core::ptr::addr_of_mut!(PML4);
    let pdpt_id = &mut *core::ptr::addr_of_mut!(PDPT_IDENTITY);
    let pdpt_hi = &mut *core::ptr::addr_of_mut!(PDPT_HIGHER);
    let pd_id = &mut *core::ptr::addr_of_mut!(PD_IDENTITY);
    let pd_hi = &mut *core::ptr::addr_of_mut!(PD_HIGHER);

    pml4[0].set_addr(
        PhysAddr::new(core::ptr::addr_of!(PDPT_IDENTITY) as u64),
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
    );
    pml4[HIGHER_HALF_PML4_INDEX].set_addr(
        PhysAddr::new(core::ptr::addr_of!(PDPT_HIGHER) as u64),
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
    );

    pdpt_id[0].set_addr(
        PhysAddr::new(core::ptr::addr_of!(PD_IDENTITY) as u64),
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
    );
    pdpt_hi[0].set_addr(
        PhysAddr::new(core::ptr::addr_of!(PD_HIGHER) as u64),
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
    );

    let huge_flags = PageTableFlags::PRESENT
        | PageTableFlags::WRITABLE
        | PageTableFlags::HUGE_PAGE
        | PageTableFlags::GLOBAL;

    let huge_count = (MAP_PHYS_BYTES / Size2MiB::SIZE) as usize;
    for i in 0..huge_count {
        let phys = PhysAddr::new(i as u64 * Size2MiB::SIZE);
        pd_id[i].set_addr(phys, huge_flags);
        pd_hi[i].set_addr(phys, huge_flags);
    }
}

unsafe fn apply_kernel_wx() {
    let text_start = &__kernel_text_start as *const u8 as u64;
    let text_end = &__kernel_text_end as *const u8 as u64;
    let rodata_start = &__kernel_rodata_start as *const u8 as u64;
    let rodata_end = &__kernel_rodata_end as *const u8 as u64;
    let data_start = &__kernel_data_start as *const u8 as u64;
    let data_end = &__kernel_bss_end as *const u8 as u64;

    map_range_4k(text_start, text_end, PageTableFlags::PRESENT);
    map_range_4k(rodata_start, rodata_end, PageTableFlags::PRESENT | PageTableFlags::NO_EXECUTE);
    map_range_4k(
        data_start,
        data_end,
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE,
    );
}

unsafe fn map_heap_region() {
    let heap_pages = HEAP_SIZE.div_ceil(Size4KiB::SIZE as usize);
    let mut virt = HEAP_VIRTUAL_START;
    for _ in 0..heap_pages {
        let frame = frame::allocate_frame().expect("heap frame");
        map_4k_page(VirtAddr::new(virt), frame.as_u64(), heap_flags());
        virt += Size4KiB::SIZE;
    }
}

fn heap_flags() -> PageTableFlags {
    PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE
}

unsafe fn map_range_4k(start: u64, end: u64, flags: PageTableFlags) {
    let mut addr = start & !(Size4KiB::SIZE - 1);
    while addr < end {
        map_4k_page(VirtAddr::new(addr), addr, flags);
        addr += Size4KiB::SIZE;
    }
}

unsafe fn map_4k_page(virt: VirtAddr, phys: u64, flags: PageTableFlags) {
    let pml4 = &mut *core::ptr::addr_of_mut!(PML4);
    let p4_idx = index(virt.p4_index());
    let pdpt = table_at_entry(&mut pml4[p4_idx]);
    let pdpt_idx = index(virt.p3_index());
    let pd = table_at_entry(&mut pdpt[pdpt_idx]);
    let pd_idx = index(virt.p2_index());

    if pd[pd_idx].flags().contains(PageTableFlags::HUGE_PAGE) {
        split_huge_page(pd, pd_idx);
    }

    let pt = table_at_entry(&mut pd[pd_idx]);
    let pt_idx = index(virt.p1_index());
    pt[pt_idx].set_addr(PhysAddr::new(phys), flags | PageTableFlags::GLOBAL);
}

unsafe fn split_huge_page(pd: &mut PageTable, pd_idx: usize) {
    let base_phys = pd[pd_idx].addr().as_u64() & !(Size2MiB::SIZE - 1);
    let pt_frame = frame::allocate_frame().expect("PT frame for split");
    let pt = &mut *identity_ptr::<PageTable>(pt_frame.as_u64());
    zero_page_table(pt);

    for i in 0..512 {
        let phys = base_phys + i as u64 * Size4KiB::SIZE;
        pt[i].set_addr(
            PhysAddr::new(phys),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::GLOBAL,
        );
    }

    pd[pd_idx].set_addr(
        PhysAddr::new(pt_frame.as_u64()),
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
    );
}

unsafe fn table_at_entry(entry: &mut PageTableEntry) -> &mut PageTable {
    if !entry.flags().contains(PageTableFlags::PRESENT) {
        let frame = frame::allocate_frame().expect("page table frame");
        let table = &mut *identity_ptr::<PageTable>(frame.as_u64());
        zero_page_table(table);
        entry.set_addr(
            PhysAddr::new(frame.as_u64()),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
        );
        return table;
    }

    &mut *identity_ptr::<PageTable>(entry.addr().as_u64())
}

unsafe fn zero_page_table(table: &mut PageTable) {
    core::ptr::write_bytes(
        table as *mut PageTable as *mut u8,
        0,
        core::mem::size_of::<PageTable>(),
    );
}

fn enable_nxe() {
    // SAFETY: Setting EFER.NXE is required for the NO_EXECUTE PTE bit.
    unsafe {
        let mut efer = Efer::read();
        efer.insert(EferFlags::NO_EXECUTE_ENABLE);
        Efer::write(efer);
    }
}

fn index<I: Into<usize>>(value: I) -> usize {
    value.into()
}

fn identity_ptr<T>(phys: u64) -> *mut T {
    phys as *mut T
}

/// Returns a mutable pointer to a physical page via the identity map.
///
/// # Safety
///
/// `phys` must refer to an identity-mapped frame for the duration of the access.
pub(crate) unsafe fn identity_mut<T>(phys: u64) -> *mut T {
    identity_ptr(phys)
}

/// Returns a reference to the bootstrap kernel PML4 (physical address equals virtual).
#[cfg(not(feature = "host-stub"))]
pub(crate) fn kernel_pml4() -> &'static PageTable {
    // SAFETY: Static PML4 is initialized before user address spaces are created.
    unsafe { &*core::ptr::addr_of!(PML4) }
}

extern "C" {
    static __kernel_text_start: u8;
    static __kernel_text_end: u8;
    static __kernel_rodata_start: u8;
    static __kernel_rodata_end: u8;
    static __kernel_data_start: u8;
    static __kernel_bss_end: u8;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::KERNEL_VIRT_BASE;

    #[test]
    fn higher_half_index_matches_design_base() {
        let virt = VirtAddr::new(KERNEL_VIRT_BASE);
        assert_eq!(index(virt.p4_index()), HIGHER_HALF_PML4_INDEX);
    }

    #[test]
    fn heap_flags_are_wx_separated() {
        let flags = heap_flags();
        assert!(flags.contains(PageTableFlags::WRITABLE));
        assert!(flags.contains(PageTableFlags::NO_EXECUTE));
    }
}
