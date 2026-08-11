//! Aether OS UEFI boot loader (M1).
//!
//! Loads `aether/kernel.elf` from the EFI System Partition, constructs
//! [`BootInfo`], exits boot services, and jumps to the kernel entry point.

#![no_main]
#![no_std]

extern crate alloc;

use aether_types::{
    BootInfo, FramebufferInfo, MemoryMapEntry, SerialPortInfo, BOOT_INFO_MAGIC, BOOT_INFO_VERSION,
    MEMORY_TYPE_CONVENTIONAL,
};
use alloc::vec;
use alloc::vec::Vec;
use core::panic::PanicInfo;
use core::slice;
use log::info;
use uefi::boot::{self, AllocateType, MemoryType};
use uefi::cstr16;
use uefi::prelude::*;
use uefi::proto::media::file::{File, FileAttribute, FileMode, FileType};
use uefi::proto::media::fs::SimpleFileSystem;

const PT_LOAD: u32 = 1;
const KERNEL_PATH: &uefi::CStr16 = cstr16!("\\aether\\kernel.elf");
const PAGE_SIZE: u64 = 4096;

#[global_allocator]
static GLOBAL_ALLOCATOR: uefi::allocator::Allocator = uefi::allocator::Allocator;

#[entry]
fn efi_main() -> Status {
    uefi::helpers::init().unwrap();

    match run() {
        Ok(()) => {
            // Successful kernel transfer does not return.
            loop {
                core::hint::spin_loop();
            }
        }
        Err(status) => status,
    }
}

fn run() -> Result<(), Status> {
    info!("Aether boot loader starting");

    let kernel_bytes = read_kernel_file()?;
    let entry = load_elf(&kernel_bytes).map_err(|_| Status::LOAD_ERROR)?;
    let boot_info = prepare_boot_info()?;

    // SAFETY: BootInfo lives in loader-allocated pages that remain valid after
    // ExitBootServices until the kernel reclaims that memory.
    unsafe {
        let _memory_map = boot::exit_boot_services(MemoryType::LOADER_DATA);
        boot_to_kernel(entry, boot_info);
    }
}

fn read_kernel_file() -> Result<Vec<u8>, Status> {
    let handle =
        boot::get_handle_for_protocol::<SimpleFileSystem>().map_err(|_| Status::NOT_FOUND)?;

    let mut fs = boot::open_protocol_exclusive::<SimpleFileSystem>(handle)
        .map_err(|_| Status::DEVICE_ERROR)?;

    let mut root = fs.open_volume().map_err(|_| Status::DEVICE_ERROR)?;
    let file_handle = root
        .open(KERNEL_PATH, FileMode::Read, FileAttribute::empty())
        .map_err(|_| Status::NOT_FOUND)?;

    let mut file = match file_handle.into_type().map_err(|_| Status::DEVICE_ERROR)? {
        FileType::Regular(f) => f,
        FileType::Dir(_) => return Err(Status::NOT_FOUND),
    };

    let mut info_buf = [0u8; 256];
    let info = file
        .get_info::<uefi::proto::media::file::FileInfo>(&mut info_buf)
        .map_err(|_| Status::BUFFER_TOO_SMALL)?;
    let size = info.file_size() as usize;

    let mut buf = vec![0u8; size];
    file.read(&mut buf).map_err(|_| Status::DEVICE_ERROR)?;

    info!("Loaded kernel.elf ({} bytes)", size);
    Ok(buf)
}

fn load_elf(data: &[u8]) -> Result<u64, ()> {
    if data.len() < 64 || data[0..4] != [0x7F, b'E', b'L', b'F'] {
        return Err(());
    }
    if data[4] != 2 || data[5] != 1 {
        // ELF64, little-endian only.
        return Err(());
    }

    let e_entry = read_u64(data, 0x18);
    let e_phoff = read_u64(data, 0x20);
    let e_phnum = read_u16(data, 0x38);
    let e_phentsize = read_u16(data, 0x36);

    if e_phentsize < 56 {
        return Err(());
    }

    for i in 0..e_phnum {
        let off = e_phoff + u64::from(i) * u64::from(e_phentsize);
        if usize::try_from(off + 56).ok().filter(|&n| n <= data.len()).is_none() {
            return Err(());
        }

        let p_type = read_u32(data, off);
        if p_type != PT_LOAD {
            continue;
        }

        let p_offset = read_u64(data, off + 8);
        let p_vaddr = read_u64(data, off + 16);
        let p_filesz = read_u64(data, off + 32);
        let p_memsz = read_u64(data, off + 40);

        if p_memsz == 0 {
            continue;
        }

        let page_count = p_memsz.div_ceil(PAGE_SIZE) as usize;
        let dest_ptr = boot::allocate_pages(
            AllocateType::Address(p_vaddr),
            MemoryType::LOADER_DATA,
            page_count,
        )
        .map_err(|_| ())?;
        let dest = dest_ptr.as_ptr() as u64;

        let dest_slice = unsafe { slice::from_raw_parts_mut(dest as *mut u8, p_memsz as usize) };
        dest_slice.fill(0);

        if p_filesz > 0 {
            let src_start = p_offset as usize;
            let src_end = src_start + p_filesz as usize;
            if src_end > data.len() {
                return Err(());
            }
            dest_slice[..p_filesz as usize].copy_from_slice(&data[src_start..src_end]);
        }
    }

    info!("Kernel ELF loaded; entry at {:#x}", e_entry);
    Ok(e_entry)
}

fn prepare_boot_info() -> Result<*mut BootInfo, Status> {
    let boot_info_ptr = boot::allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, 1)
        .map_err(|_| Status::OUT_OF_RESOURCES)?;
    let map_ptr = boot::allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, 1)
        .map_err(|_| Status::OUT_OF_RESOURCES)?;

    let boot_info = boot_info_ptr.as_ptr() as *mut BootInfo;
    let memory_map = map_ptr.as_ptr() as *mut MemoryMapEntry;

    // M1: placeholder memory-map storage; full UEFI map copy planned for M2.
    unsafe {
        (*memory_map).phys_start = 0;
        (*memory_map).page_count = 0;
        (*memory_map).memory_type = MEMORY_TYPE_CONVENTIONAL;
        (*memory_map).attributes = 0;

        (*boot_info).magic = BOOT_INFO_MAGIC;
        (*boot_info).version = BOOT_INFO_VERSION;
        (*boot_info).flags = 0;
        (*boot_info).memory_map = memory_map;
        (*boot_info).memory_map_len = 0;
        (*boot_info).framebuffer = FramebufferInfo::default();
        (*boot_info).rsdp = 0;
        (*boot_info).serial_port = SerialPortInfo::default();
    }

    Ok(boot_info)
}

/// Transfers control to the kernel; does not return on success.
///
/// # Safety
///
/// `entry` must be a valid kernel entry point. `boot_info` must point to a
/// valid [`BootInfo`]. Boot services must already be exited.
unsafe fn boot_to_kernel(entry: u64, boot_info: *mut BootInfo) -> ! {
    let entry_fn: extern "sysv64" fn(*const BootInfo) -> ! = core::mem::transmute(entry as usize);

    entry_fn(boot_info)
}

fn read_u16(data: &[u8], offset: u64) -> u16 {
    let i = offset as usize;
    u16::from_le_bytes([data[i], data[i + 1]])
}

fn read_u32(data: &[u8], offset: u64) -> u32 {
    let i = offset as usize;
    u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]])
}

fn read_u64(data: &[u8], offset: u64) -> u64 {
    let i = offset as usize;
    u64::from_le_bytes([
        data[i],
        data[i + 1],
        data[i + 2],
        data[i + 3],
        data[i + 4],
        data[i + 5],
        data[i + 6],
        data[i + 7],
    ])
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("BOOT PANIC: {}", info);
    loop {
        core::hint::spin_loop();
    }
}
