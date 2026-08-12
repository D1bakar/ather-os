//! Aether OS UEFI boot loader (M1).
//!
//! Loads `aether/kernel.elf` from the EFI System Partition, constructs
//! [`BootInfo`], exits boot services, and jumps to the kernel entry point.

#![no_main]
#![no_std]

extern crate alloc;

mod signature;

use aether_types::{
    BootInfo, FramebufferInfo, MemoryMapEntry, SerialPortInfo, BOOT_INFO_MAGIC, BOOT_INFO_VERSION,
};
use alloc::vec;
use alloc::vec::Vec;
use core::mem::{self, size_of};
use core::panic::PanicInfo;
use core::slice;
use log::{error, info, warn};
use uefi::boot::{self, AllocateType, MemoryType};
use uefi::cstr16;
use uefi::mem::memory_map::MemoryMap;
use uefi::prelude::*;
use uefi::proto::console::gop::{GraphicsOutput, PixelFormat};
use uefi::proto::media::file::{File, FileAttribute, FileMode, FileType};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::system;
use uefi::table::cfg::{ACPI2_GUID, ACPI_GUID};

const PT_LOAD: u32 = 1;
const KERNEL_PATH: &uefi::CStr16 = cstr16!("\\aether\\kernel.elf");
const PAGE_SIZE: u64 = 4096;
/// Matches [`BOOT_INFO_FLAG_FRAMEBUFFER`] in `aether-types`.
const BOOT_INFO_FLAG_FRAMEBUFFER: u32 = 1 << 0;
/// Extra UEFI memory-map slots reserved for loader allocations before exit.
const MEMORY_MAP_ENTRY_SLACK: usize = 8;

/// Boot-loader allocated handoff state populated before `ExitBootServices`.
struct PreparedBoot {
    boot_info: *mut BootInfo,
    map_entries: *mut MemoryMapEntry,
    map_capacity: usize,
}

/// Typed boot failure with a human-readable log line and UEFI status mapping.
#[derive(Clone, Copy, Debug)]
enum BootError {
    NoFilesystem,
    VolumeOpenFailed,
    KernelNotFound,
    KernelIsDirectory,
    KernelInfoFailed,
    KernelReadFailed,
    InvalidKernelSignature,
    InvalidElf,
    ElfSegmentOutOfBounds,
    ElfSegmentAllocFailed,
    MemoryMapQueryFailed,
    BootInfoAllocFailed,
    MemoryMapAllocFailed,
}

impl BootError {
    fn log(self) {
        let msg = match self {
            Self::NoFilesystem => "no SimpleFileSystem protocol handle found on any device",
            Self::VolumeOpenFailed => "failed to open EFI volume or file protocol",
            Self::KernelNotFound => "kernel not found at \\aether\\kernel.elf on ESP",
            Self::KernelIsDirectory => "kernel path is a directory, not a regular file",
            Self::KernelInfoFailed => "failed to query kernel.elf file size",
            Self::KernelReadFailed => "failed to read kernel.elf from ESP",
            Self::InvalidKernelSignature => "kernel signature stub rejected image (invalid ELF)",
            Self::InvalidElf => "kernel.elf is not a valid ELF64 little-endian executable",
            Self::ElfSegmentOutOfBounds => "ELF PT_LOAD segment extends past file bounds",
            Self::ElfSegmentAllocFailed => "failed to allocate pages for ELF PT_LOAD segment",
            Self::MemoryMapQueryFailed => "GetMemoryMap failed while sizing handoff buffer",
            Self::BootInfoAllocFailed => "failed to allocate BootInfo structure",
            Self::MemoryMapAllocFailed => "failed to allocate stable memory-map buffer",
        };
        error!("boot error: {msg}");
    }

    fn status(self) -> Status {
        match self {
            Self::NoFilesystem | Self::KernelNotFound | Self::KernelIsDirectory => {
                Status::NOT_FOUND
            }
            Self::InvalidKernelSignature | Self::InvalidElf | Self::ElfSegmentOutOfBounds => {
                Status::LOAD_ERROR
            }
            Self::MemoryMapQueryFailed
            | Self::BootInfoAllocFailed
            | Self::MemoryMapAllocFailed
            | Self::ElfSegmentAllocFailed => Status::OUT_OF_RESOURCES,
            Self::VolumeOpenFailed | Self::KernelReadFailed => Status::DEVICE_ERROR,
            Self::KernelInfoFailed => Status::BUFFER_TOO_SMALL,
        }
    }
}

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

    let kernel_bytes = read_kernel_file().map_err(|e| {
        e.log();
        e.status()
    })?;

    signature::verify_kernel_signature(&kernel_bytes).map_err(|()| {
        BootError::InvalidKernelSignature.log();
        BootError::InvalidKernelSignature.status()
    })?;

    let entry = load_elf(&kernel_bytes).map_err(|e| {
        e.log();
        e.status()
    })?;

    let prepared = prepare_boot_info().map_err(|e| {
        e.log();
        e.status()
    })?;

    // SAFETY: BootInfo and memory-map storage live in loader-allocated pages that
    // remain valid after ExitBootServices until the kernel reclaims that memory.
    // No boot-services protocol references remain after exit_boot_services.
    unsafe {
        let final_map = boot::exit_boot_services(MemoryType::LOADER_DATA);
        copy_memory_map_to_boot_info(&final_map, &prepared);
        boot_to_kernel(entry, prepared.boot_info);
    }
}

fn read_kernel_file() -> Result<Vec<u8>, BootError> {
    let handle =
        boot::get_handle_for_protocol::<SimpleFileSystem>().map_err(|_| BootError::NoFilesystem)?;

    let mut fs = boot::open_protocol_exclusive::<SimpleFileSystem>(handle)
        .map_err(|_| BootError::VolumeOpenFailed)?;

    let mut root = fs.open_volume().map_err(|_| BootError::VolumeOpenFailed)?;
    let file_handle = root
        .open(KERNEL_PATH, FileMode::Read, FileAttribute::empty())
        .map_err(|_| BootError::KernelNotFound)?;

    let mut file = match file_handle.into_type().map_err(|_| BootError::VolumeOpenFailed)? {
        FileType::Regular(f) => f,
        FileType::Dir(_) => return Err(BootError::KernelIsDirectory),
    };

    let mut info_buf = [0u8; 256];
    let info = file
        .get_info::<uefi::proto::media::file::FileInfo>(&mut info_buf)
        .map_err(|_| BootError::KernelInfoFailed)?;
    let size = info.file_size() as usize;

    let mut buf = vec![0u8; size];
    file.read(&mut buf).map_err(|_| BootError::KernelReadFailed)?;

    info!("Loaded kernel.elf ({} bytes)", size);
    Ok(buf)
}

/// Returns true when `[start, end)` lies entirely inside a prior loader allocation.
fn range_already_allocated(ranges: &[(u64, u64)], start: u64, end: u64) -> bool {
    ranges.iter().any(|&(alloc_start, alloc_end)| alloc_start <= start && alloc_end >= end)
}

fn load_elf(data: &[u8]) -> Result<u64, BootError> {
    if data.len() < 64 || data[0..4] != [0x7F, b'E', b'L', b'F'] {
        return Err(BootError::InvalidElf);
    }
    if data[4] != 2 || data[5] != 1 {
        return Err(BootError::InvalidElf);
    }

    let e_entry = read_u64(data, 0x18);
    let e_phoff = read_u64(data, 0x20);
    let e_phnum = read_u16(data, 0x38);
    let e_phentsize = read_u16(data, 0x36);

    if e_phentsize < 56 {
        return Err(BootError::InvalidElf);
    }

    let mut allocated_ranges: Vec<(u64, u64)> = Vec::new();

    for i in 0..e_phnum {
        let off = e_phoff + u64::from(i) * u64::from(e_phentsize);
        if usize::try_from(off + 56).ok().filter(|&n| n <= data.len()).is_none() {
            return Err(BootError::InvalidElf);
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

        // UEFI AllocatePages requires a page-aligned physical address. Linker
        // scripts may place small .data/.bss PT_LOAD segments at unaligned
        // offsets within a page already covered by an earlier segment.
        let page_base = p_vaddr & !(PAGE_SIZE - 1);
        let in_page_offset = (p_vaddr - page_base) as usize;
        let span_bytes = in_page_offset as u64 + p_memsz;
        let page_count = span_bytes.div_ceil(PAGE_SIZE) as usize;
        let region_end = page_base + page_count as u64 * PAGE_SIZE;

        if !range_already_allocated(&allocated_ranges, page_base, region_end) {
            boot::allocate_pages(
                AllocateType::Address(page_base),
                MemoryType::LOADER_DATA,
                page_count,
            )
            .map_err(|_| {
                error!(
                    "PT_LOAD segment {} alloc failed: vaddr={:#x} pages={} (page_base={:#x})",
                    i, p_vaddr, page_count, page_base
                );
                BootError::ElfSegmentAllocFailed
            })?;
            allocated_ranges.push((page_base, region_end));
        }

        let dest_slice =
            unsafe { slice::from_raw_parts_mut(page_base as *mut u8, span_bytes as usize) };
        dest_slice[in_page_offset..].fill(0);
        let segment_slice = &mut dest_slice[in_page_offset..in_page_offset + p_memsz as usize];

        if p_filesz > 0 {
            let src_start = p_offset as usize;
            let src_end = src_start + p_filesz as usize;
            if src_end > data.len() {
                return Err(BootError::ElfSegmentOutOfBounds);
            }
            segment_slice[..p_filesz as usize].copy_from_slice(&data[src_start..src_end]);
        }
    }

    info!("Kernel ELF loaded; entry at {:#x}", e_entry);
    Ok(e_entry)
}

fn prepare_boot_info() -> Result<PreparedBoot, BootError> {
    let snapshot =
        boot::memory_map(MemoryType::LOADER_DATA).map_err(|_| BootError::MemoryMapQueryFailed)?;
    let map_capacity = snapshot.len().saturating_add(MEMORY_MAP_ENTRY_SLACK);
    mem::forget(snapshot);

    let map_bytes = map_capacity.saturating_mul(size_of::<MemoryMapEntry>());
    let map_pages =
        map_bytes.saturating_add(PAGE_SIZE as usize - 1).div_ceil(PAGE_SIZE as usize).max(1);

    let boot_info_ptr = boot::allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, 1)
        .map_err(|_| BootError::BootInfoAllocFailed)?;
    let map_ptr = boot::allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, map_pages)
        .map_err(|_| BootError::MemoryMapAllocFailed)?;

    let boot_info = boot_info_ptr.as_ptr() as *mut BootInfo;
    let map_entries = map_ptr.as_ptr() as *mut MemoryMapEntry;

    let rsdp = locate_rsdp();
    let (framebuffer, fb_flag) = detect_framebuffer();

    unsafe {
        (*boot_info).magic = BOOT_INFO_MAGIC;
        (*boot_info).version = BOOT_INFO_VERSION;
        (*boot_info).flags = fb_flag;
        (*boot_info).memory_map = map_entries;
        (*boot_info).memory_map_len = 0;
        (*boot_info).framebuffer = framebuffer;
        (*boot_info).rsdp = rsdp;
        (*boot_info).serial_port = SerialPortInfo::default();
    }

    if rsdp != 0 {
        info!("ACPI RSDP located at {:#x}", rsdp);
    } else {
        warn!("ACPI RSDP not found in UEFI configuration table");
    }

    if fb_flag & BOOT_INFO_FLAG_FRAMEBUFFER != 0 {
        info!(
            "framebuffer: {:#x} {}x{} stride={} px_fmt={}",
            framebuffer.base,
            framebuffer.width,
            framebuffer.height,
            framebuffer.stride,
            framebuffer.pixel_format
        );
    } else {
        info!("framebuffer: GOP unavailable or blt-only mode");
    }

    Ok(PreparedBoot { boot_info, map_entries, map_capacity })
}

fn copy_memory_map_to_boot_info(
    uefi_map: &uefi::mem::memory_map::MemoryMapOwned,
    prepared: &PreparedBoot,
) {
    let count = uefi_map.len().min(prepared.map_capacity);
    if count == 0 {
        error!("boot error: UEFI memory map is empty after ExitBootServices");
        return;
    }

    if uefi_map.len() > prepared.map_capacity {
        warn!(
            "memory map truncated: {} entries available, {} slots allocated",
            uefi_map.len(),
            prepared.map_capacity
        );
    }

    for (i, desc) in uefi_map.entries().take(count).enumerate() {
        unsafe {
            *prepared.map_entries.add(i) = MemoryMapEntry {
                phys_start: desc.phys_start,
                page_count: desc.page_count,
                memory_type: desc.ty.0,
                attributes: desc.att.bits(),
            };
        }
    }

    unsafe {
        (*prepared.boot_info).memory_map = prepared.map_entries;
        (*prepared.boot_info).memory_map_len = count;
    }

    info!("memory map copied to BootInfo ({} entries, capacity {})", count, prepared.map_capacity);
}

fn locate_rsdp() -> u64 {
    system::with_config_table(|entries| {
        if let Some(entry) = entries.iter().find(|e| e.guid == ACPI2_GUID) {
            return entry.address as u64;
        }
        entries.iter().find(|e| e.guid == ACPI_GUID).map(|entry| entry.address as u64).unwrap_or(0)
    })
}

fn detect_framebuffer() -> (FramebufferInfo, u32) {
    let handle = match boot::get_handle_for_protocol::<GraphicsOutput>() {
        Ok(h) => h,
        Err(_) => return (FramebufferInfo::default(), 0),
    };

    let mut gop = match boot::open_protocol_exclusive::<GraphicsOutput>(handle) {
        Ok(g) => g,
        Err(_) => return (FramebufferInfo::default(), 0),
    };

    let mode_info = gop.current_mode_info();
    if mode_info.pixel_format() == PixelFormat::BltOnly {
        return (FramebufferInfo::default(), 0);
    }

    let (width, height) = mode_info.resolution();
    let fb_ptr = gop.frame_buffer().as_mut_ptr() as u64;
    if fb_ptr == 0 || width == 0 || height == 0 {
        return (FramebufferInfo::default(), 0);
    }

    let info = FramebufferInfo {
        base: fb_ptr,
        width: width.min(u32::MAX as usize) as u32,
        height: height.min(u32::MAX as usize) as u32,
        stride: mode_info.stride().min(u32::MAX as usize) as u32,
        pixel_format: encode_pixel_format(mode_info.pixel_format()),
    };

    (info, BOOT_INFO_FLAG_FRAMEBUFFER)
}

fn encode_pixel_format(format: PixelFormat) -> u32 {
    match format {
        PixelFormat::Rgb => 1,
        PixelFormat::Bgr => 2,
        PixelFormat::Bitmask => 3,
        PixelFormat::BltOnly => 0,
    }
}

/// Transfers control to the kernel; does not return on success.
///
/// # Safety
///
/// `entry` must be a valid kernel entry point. `boot_info` must point to a
/// valid [`BootInfo`]. Boot services must already be exited.
unsafe fn boot_to_kernel(entry: u64, boot_info: *mut BootInfo) -> ! {
    let entry_fn: extern "sysv64" fn(*const BootInfo) -> ! = mem::transmute(entry as usize);

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
    error!("BOOT PANIC: {}", info);
    loop {
        core::hint::spin_loop();
    }
}
