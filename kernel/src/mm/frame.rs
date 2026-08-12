//! Bitmap physical frame allocator (4 KiB frames).
//!
//! Conventional RAM from [`BootInfo::memory_map`] is tracked in a static bitmap.
//! Reserved regions (kernel image, boot structures, page tables) are marked used
//! before the allocator accepts requests.

use aether_types::{BootInfo, MemoryMapEntry, PhysicalAddress, MEMORY_TYPE_CONVENTIONAL};

/// Size of one allocatable frame.
pub const FRAME_SIZE: u64 = 4096;

/// Maximum physical address tracked by the default bitmap (8 GiB).
pub const MAX_PHYS_BYTES: u64 = 8 * 1024 * 1024 * 1024;

const BITMAP_BITS: usize = (MAX_PHYS_BYTES / FRAME_SIZE) as usize;
const BITMAP_QWORDS: usize = BITMAP_BITS / 64;

/// Inclusive-exclusive physical range `[start, end)` reserved from allocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReservedRegion {
    /// First byte of the reserved span.
    pub start: u64,
    /// One past the last reserved byte.
    pub end: u64,
}

/// Bitmap-backed physical frame allocator.
pub struct FrameAllocator {
    bitmap: [u64; BITMAP_QWORDS],
    total_frames: usize,
    used_frames: usize,
}

impl Default for FrameAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameAllocator {
    /// Returns an empty allocator (all frames marked used until [`Self::init`]).
    pub const fn new() -> Self {
        Self { bitmap: [u64::MAX; BITMAP_QWORDS], total_frames: 0, used_frames: 0 }
    }

    /// Builds the allocator from a UEFI memory map and reserved regions.
    pub fn init(&mut self, entries: &[MemoryMapEntry], reserved: &[ReservedRegion]) {
        self.bitmap = [u64::MAX; BITMAP_QWORDS];
        self.total_frames = 0;
        self.used_frames = 0;

        for entry in entries {
            if entry.memory_type != MEMORY_TYPE_CONVENTIONAL {
                continue;
            }
            let start = align_up(entry.phys_start, FRAME_SIZE);
            let end = entry.phys_start + entry.page_count * FRAME_SIZE;
            let mut addr = start;
            while addr + FRAME_SIZE <= end && addr < MAX_PHYS_BYTES {
                self.add_free_frame(addr);
                addr += FRAME_SIZE;
            }
        }

        for region in reserved {
            self.mark_range_used(region.start, region.end);
        }
    }

    /// Allocates one free 4 KiB frame.
    pub fn allocate(&mut self) -> Option<PhysicalAddress> {
        for (word_idx, word) in self.bitmap.iter_mut().enumerate() {
            let free_mask = !*word;
            if free_mask == 0 {
                continue;
            }
            let free_bit = free_mask.trailing_zeros() as usize;
            *word |= 1 << free_bit;
            self.used_frames += 1;
            let frame_idx = word_idx * 64 + free_bit;
            return Some(PhysicalAddress::new(frame_idx as u64 * FRAME_SIZE));
        }
        None
    }

    /// Returns a frame to the free pool.
    pub fn deallocate(&mut self, addr: PhysicalAddress) {
        self.release(addr);
    }

    /// Marks a single frame as in use.
    pub fn mark_used(&mut self, addr: PhysicalAddress) {
        let frame = frame_index(addr.as_u64());
        if frame >= BITMAP_BITS {
            return;
        }
        let word_idx = frame / 64;
        let bit = frame % 64;
        let mask = 1_u64 << bit;
        if self.bitmap[word_idx] & mask == 0 {
            self.bitmap[word_idx] |= mask;
            self.used_frames += 1;
        }
    }

    /// Returns a frame to the free pool (clears the allocated bit).
    pub fn release(&mut self, addr: PhysicalAddress) {
        let frame = frame_index(addr.as_u64());
        if frame >= BITMAP_BITS {
            return;
        }
        let word_idx = frame / 64;
        let bit = frame % 64;
        let mask = 1_u64 << bit;
        if self.bitmap[word_idx] & mask != 0 {
            self.bitmap[word_idx] &= !mask;
            self.used_frames = self.used_frames.saturating_sub(1);
        }
    }

    fn add_free_frame(&mut self, addr: u64) {
        let frame = frame_index(addr);
        if frame >= BITMAP_BITS {
            return;
        }
        let word_idx = frame / 64;
        let bit = frame % 64;
        let mask = 1_u64 << bit;
        if self.bitmap[word_idx] & mask != 0 {
            self.bitmap[word_idx] &= !mask;
            self.total_frames += 1;
        }
    }

    /// Marks every frame overlapping `[start, end)` as used.
    pub fn mark_range_used(&mut self, start: u64, end: u64) {
        if end <= start {
            return;
        }
        let mut addr = align_down(start, FRAME_SIZE);
        while addr < end && addr < MAX_PHYS_BYTES {
            self.mark_used(PhysicalAddress::new(addr));
            addr += FRAME_SIZE;
        }
    }

    /// Number of frames currently marked free in the bitmap.
    #[must_use]
    pub fn free_frames(&self) -> usize {
        self.total_frames.saturating_sub(self.used_frames)
    }

    /// Number of frames marked in use.
    #[must_use]
    pub fn used_frames(&self) -> usize {
        self.used_frames
    }
}

static mut FRAME_ALLOCATOR: FrameAllocator = FrameAllocator::new();

/// Initializes the global frame allocator from boot information.
pub fn init(boot_info: &BootInfo) {
    let entries = memory_map_entries(boot_info);
    let reserved = reserved_regions(boot_info);
    // SAFETY: BSP early boot, single-threaded.
    unsafe {
        (*core::ptr::addr_of_mut!(FRAME_ALLOCATOR)).init(entries, &reserved);
    }
}

/// Returns a mutable reference to the global frame allocator.
///
/// # Safety
///
/// Caller must hold exclusive access (early boot or with interrupts disabled).
pub unsafe fn allocator() -> &'static mut FrameAllocator {
    // SAFETY: Mutable static access is coordinated by the caller.
    &mut *core::ptr::addr_of_mut!(FRAME_ALLOCATOR)
}

/// Allocates one physical frame from the global allocator.
pub fn allocate_frame() -> Option<PhysicalAddress> {
    // SAFETY: Single-threaded during early boot; later callers must synchronize.
    unsafe { allocator().allocate() }
}

/// Returns a frame to the global allocator.
pub fn deallocate_frame(addr: PhysicalAddress) {
    // SAFETY: Single-threaded during early boot; later callers must synchronize.
    unsafe {
        allocator().deallocate(addr);
    }
}

fn memory_map_entries(boot_info: &BootInfo) -> &[MemoryMapEntry] {
    if boot_info.memory_map.is_null() || boot_info.memory_map_len == 0 {
        return &FALLBACK_MEMORY_MAP;
    }
    // SAFETY: Boot loader guarantees the map remains valid after handoff.
    unsafe { core::slice::from_raw_parts(boot_info.memory_map, boot_info.memory_map_len) }
}

fn reserved_regions(boot_info: &BootInfo) -> [ReservedRegion; 4] {
    let (kernel_start, kernel_end) = kernel_image_bounds();
    let boot_info_start = boot_info as *const BootInfo as u64;
    let boot_info_end = boot_info_start + core::mem::size_of::<BootInfo>() as u64;

    [
        ReservedRegion { start: 0, end: 0x10_0000 },
        ReservedRegion { start: kernel_start, end: kernel_end },
        ReservedRegion { start: boot_info_start, end: boot_info_end },
        ReservedRegion { start: MAX_PHYS_BYTES, end: u64::MAX },
    ]
}

fn kernel_image_bounds() -> (u64, u64) {
    #[cfg(not(feature = "host-stub"))]
    {
        (core::ptr::addr_of!(__kernel_start) as u64, core::ptr::addr_of!(__kernel_end) as u64)
    }
    #[cfg(feature = "host-stub")]
    {
        (0, 0)
    }
}

static FALLBACK_MEMORY_MAP: [MemoryMapEntry; 1] = [MemoryMapEntry {
    phys_start: 0x100000,
    page_count: (512 * 1024 * 1024 - 0x100000) / FRAME_SIZE,
    memory_type: MEMORY_TYPE_CONVENTIONAL,
    attributes: 0,
}];

#[cfg(not(feature = "host-stub"))]
extern "C" {
    static __kernel_start: u8;
    static __kernel_end: u8;
}

const fn frame_index(addr: u64) -> usize {
    (addr / FRAME_SIZE) as usize
}

const fn align_up(addr: u64, align: u64) -> u64 {
    (addr + align - 1) & !(align - 1)
}

const fn align_down(addr: u64, align: u64) -> u64 {
    addr & !(align - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_map() -> [MemoryMapEntry; 1] {
        [MemoryMapEntry {
            phys_start: 0,
            page_count: 256,
            memory_type: MEMORY_TYPE_CONVENTIONAL,
            attributes: 0,
        }]
    }

    #[test]
    fn allocate_and_deallocate_round_trip() {
        let mut alloc = FrameAllocator::new();
        alloc.init(&sample_map(), &[]);
        let frame = alloc.allocate().expect("free frame");
        assert_eq!(frame.as_u64() % FRAME_SIZE, 0);
        let used = alloc.used_frames();
        alloc.deallocate(frame);
        assert_eq!(alloc.used_frames(), used - 1);
    }

    #[test]
    fn reserved_range_is_not_allocated() {
        let mut alloc = FrameAllocator::new();
        let reserved = [ReservedRegion { start: 0, end: 4096 }];
        alloc.init(&sample_map(), &reserved);
        let frame = alloc.allocate().expect("frame beyond reservation");
        assert!(frame.as_u64() >= 4096);
    }

    #[test]
    fn free_frames_accounting() {
        let mut alloc = FrameAllocator::new();
        alloc.init(&sample_map(), &[]);
        let initial_free = alloc.free_frames();
        let _ = alloc.allocate();
        assert_eq!(alloc.free_frames(), initial_free - 1);
    }
}
