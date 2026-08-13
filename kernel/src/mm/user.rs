//! Per-process user address spaces (M6).
//!
//! User layout:
//! - ELF load base: `0x0040_0000` (see `user/linker.ld`)
//! - User stack top: [`USER_STACK_TOP`] (grows down, [`USER_STACK_SIZE`] bytes mapped)

use super::frame;
use super::paging;
use aether_types::{PhysicalAddress, USER_STACK_TOP as ABI_USER_STACK_TOP};
use x86_64::structures::paging::page_table::{PageTable, PageTableEntry, PageTableFlags};
use x86_64::structures::paging::{PageSize, Size4KiB};
use x86_64::VirtAddr;

/// Top of the user stack (initial RSP).
///
/// Avoids `0x8000_0000`, which often hosts the GOP framebuffer on QEMU/OVMF.
pub const USER_STACK_TOP: u64 = ABI_USER_STACK_TOP;
/// Bytes of user stack mapped below [`USER_STACK_TOP`].
pub const USER_STACK_SIZE: u64 = 64 * 1024;

const HIGHER_HALF_PML4_INDEX: usize = 256;

fn page_index<I: Into<usize>>(value: I) -> usize {
    value.into()
}

fn user_access() -> PageTableFlags {
    PageTableFlags::USER_ACCESSIBLE
}

/// Creates a fresh PML4 with the kernel higher-half direct map shared.
pub fn create_user_address_space() -> Option<PhysicalAddress> {
    let pml4_frame = frame::allocate_frame()?;
    // SAFETY: New frame is exclusively owned; identity-mapped during early boot.
    unsafe {
        let pml4 = &mut *paging::identity_mut::<PageTable>(pml4_frame.as_u64());
        core::ptr::write_bytes(
            pml4 as *mut PageTable as *mut u8,
            0,
            core::mem::size_of::<PageTable>(),
        );

        let kernel_pml4 = paging::kernel_pml4();
        pml4[HIGHER_HALF_PML4_INDEX] = kernel_pml4[HIGHER_HALF_PML4_INDEX].clone();
    }
    Some(pml4_frame)
}

/// Maps one 4 KiB user page in `cr3_root`.
pub fn map_user_page(
    cr3_root: PhysicalAddress,
    virt: u64,
    phys: u64,
    flags: PageTableFlags,
) -> Result<(), ()> {
    let user_flags = flags | user_access();
    // SAFETY: Page-table pages are accessed through the identity map during bring-up.
    unsafe {
        map_user_4k_page(cr3_root.as_u64(), virt, phys, user_flags)?;
    }
    Ok(())
}

/// Maps a contiguous user region and copies `data` starting at `virt`.
pub fn map_user_region(
    cr3_root: PhysicalAddress,
    virt: u64,
    data: &[u8],
    flags: PageTableFlags,
) -> Result<(), ()> {
    if data.is_empty() {
        return Ok(());
    }
    let page_size = Size4KiB::SIZE;
    let mut offset = 0usize;
    while offset < data.len() {
        let page_virt = virt + offset as u64;
        let page_base = page_virt & !(page_size - 1);
        let frame = frame::allocate_frame().ok_or(())?;
        // SAFETY: Fresh frame is exclusively owned and identity-mapped.
        unsafe {
            core::ptr::write_bytes(frame.as_u64() as *mut u8, 0, page_size as usize);
        }
        map_user_page(cr3_root, page_base, frame.as_u64(), flags)?;

        let page_off = (page_virt - page_base) as usize;
        let chunk = (page_size as usize - page_off).min(data.len() - offset);
        // SAFETY: `frame` is identity-mapped; bytes belong to this new mapping.
        unsafe {
            let dst = frame.as_u64().wrapping_add(page_off as u64) as *mut u8;
            core::ptr::copy_nonoverlapping(data.as_ptr().add(offset), dst, chunk);
        }
        offset += chunk;
    }
    Ok(())
}

/// Zero-fills `[virt, virt + len)` in the user address space.
pub fn zero_user_region(cr3_root: PhysicalAddress, virt: u64, len: u64) -> Result<(), ()> {
    if len == 0 {
        return Ok(());
    }
    let page_size = Size4KiB::SIZE;
    let start = virt & !(page_size - 1);
    let end = virt.saturating_add(len);
    let mut page = start;
    while page < end {
        let frame = frame::allocate_frame().ok_or(())?;
        // SAFETY: Fresh frame exclusively for this mapping.
        unsafe {
            core::ptr::write_bytes(frame.as_u64() as *mut u8, 0, page_size as usize);
        }
        map_user_page(
            cr3_root,
            page,
            frame.as_u64(),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE,
        )?;
        page += page_size;
    }
    Ok(())
}

/// Maps the standard user stack region.
pub fn map_user_stack(cr3_root: PhysicalAddress) -> Result<(), ()> {
    zero_user_region(cr3_root, USER_STACK_TOP - USER_STACK_SIZE, USER_STACK_SIZE)
}

unsafe fn map_user_4k_page(
    cr3_phys: u64,
    virt: u64,
    phys: u64,
    flags: PageTableFlags,
) -> Result<(), ()> {
    let virt = VirtAddr::new(virt);
    let pml4 = &mut *paging::identity_mut::<PageTable>(cr3_phys);
    let p4_idx = page_index(virt.p4_index());
    let pdpt = table_at_entry(&mut pml4[p4_idx])?;
    let pdpt_idx = page_index(virt.p3_index());
    let pd = table_at_entry(&mut pdpt[pdpt_idx])?;
    let pd_idx = page_index(virt.p2_index());

    if pd[pd_idx].flags().contains(PageTableFlags::HUGE_PAGE) {
        split_huge_page(pd, pd_idx)?;
    }

    let pt = table_at_entry(&mut pd[pd_idx])?;
    let pt_idx = page_index(virt.p1_index());
    pt[pt_idx].set_addr(x86_64::PhysAddr::new(phys), flags);
    Ok(())
}

unsafe fn table_at_entry(entry: &mut PageTableEntry) -> Result<&mut PageTable, ()> {
    if !entry.flags().contains(PageTableFlags::PRESENT) {
        let frame = frame::allocate_frame().ok_or(())?;
        let table = &mut *paging::identity_mut::<PageTable>(frame.as_u64());
        core::ptr::write_bytes(
            table as *mut PageTable as *mut u8,
            0,
            core::mem::size_of::<PageTable>(),
        );
        entry.set_addr(
            x86_64::PhysAddr::new(frame.as_u64()),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | user_access(),
        );
        return Ok(table);
    }
    Ok(&mut *paging::identity_mut::<PageTable>(entry.addr().as_u64()))
}

unsafe fn split_huge_page(pd: &mut PageTable, pd_idx: usize) -> Result<(), ()> {
    let base_phys = pd[pd_idx].addr().as_u64() & !(Size4KiB::SIZE * 512 - 1);
    let pt_frame = frame::allocate_frame().ok_or(())?;
    let pt = &mut *paging::identity_mut::<PageTable>(pt_frame.as_u64());
    core::ptr::write_bytes(pt as *mut PageTable as *mut u8, 0, core::mem::size_of::<PageTable>());

    for i in 0..512_usize {
        let phys = base_phys + i as u64 * Size4KiB::SIZE;
        pt[i].set_addr(
            x86_64::PhysAddr::new(phys),
            PageTableFlags::PRESENT
                | PageTableFlags::WRITABLE
                | user_access()
                | PageTableFlags::GLOBAL,
        );
    }

    pd[pd_idx].set_addr(
        x86_64::PhysAddr::new(pt_frame.as_u64()),
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | user_access(),
    );
    Ok(())
}
