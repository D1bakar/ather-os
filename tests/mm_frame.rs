//! Host integration tests for M3 physical frame allocator bitmap logic.

use aether_kernel::mm::frame::{FrameAllocator, ReservedRegion, FRAME_SIZE};
use aether_types::{MemoryMapEntry, MEMORY_TYPE_CONVENTIONAL};

fn sample_map() -> [MemoryMapEntry; 1] {
    [MemoryMapEntry {
        phys_start: 0,
        page_count: 512,
        memory_type: MEMORY_TYPE_CONVENTIONAL,
        attributes: 0,
    }]
}

#[test]
fn frame_allocator_respects_low_memory_reservation() {
    let mut alloc = FrameAllocator::new();
    let reserved = [ReservedRegion { start: 0, end: 0x10_0000 }];
    alloc.init(&sample_map(), &reserved);
    let frame = alloc.allocate().expect("frame above 1 MiB hole");
    assert!(frame.as_u64() >= 0x10_0000);
    assert_eq!(frame.as_u64() % FRAME_SIZE, 0);
}

#[test]
fn frame_allocator_tracks_used_frames() {
    let mut alloc = FrameAllocator::new();
    alloc.init(&sample_map(), &[]);
    let before = alloc.free_frames();
    let frame = alloc.allocate().expect("alloc");
    assert_eq!(alloc.free_frames(), before - 1);
    alloc.deallocate(frame);
    assert_eq!(alloc.free_frames(), before);
}

#[test]
fn non_conventional_memory_is_excluded() {
    let mut alloc = FrameAllocator::new();
    let map = [MemoryMapEntry { phys_start: 0, page_count: 64, memory_type: 1, attributes: 0 }];
    alloc.init(&map, &[]);
    assert!(alloc.allocate().is_none());
}
