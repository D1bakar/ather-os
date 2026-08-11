//! Linked-list kernel heap allocator.
//!
//! Backing storage is mapped at [`super::HEAP_VIRTUAL_START`] by the paging
//! module. Uses [`core::alloc::GlobalAlloc`] for `alloc` crate consumers.

use crate::mm::{HEAP_SIZE, HEAP_VIRTUAL_START};
use aether_sync::SpinMutex;
use core::alloc::{GlobalAlloc, Layout};
use core::mem::size_of;
use core::ptr::NonNull;

struct ListNode {
    size: usize,
    next: Option<NonNull<ListNode>>,
}

/// Simple first-fit heap over a fixed virtual region.
pub struct LinkedListAllocator {
    head: ListNode,
}

// SAFETY: BSP-only heap; `SpinMutex` serializes access.
unsafe impl Send for LinkedListAllocator {}
unsafe impl Sync for LinkedListAllocator {}

impl LinkedListAllocator {
    /// Creates an empty allocator; [`Self::init`] must run before use.
    pub const fn new() -> Self {
        Self { head: ListNode { size: 0, next: None } }
    }

    /// Resets the free list to cover `[heap_start, heap_start + heap_size)`.
    ///
    /// # Safety
    ///
    /// The memory range must be mapped and writable.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.head.next = Some(NonNull::new_unchecked(heap_start as *mut ListNode));
        self.head.size = heap_size;
    }

    /// Allocates a block using first-fit.
    pub fn allocate_first_fit(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        let size = layout.size().max(size_of::<ListNode>());
        let align = layout.align();

        let mut current = NonNull::new(&mut self.head as *mut ListNode)?;

        loop {
            let node = unsafe { current.as_mut() };
            match node.next {
                None => return None,
                Some(mut next) => {
                    let next_node = unsafe { next.as_mut() };
                    if next_node.size >= size {
                        let addr = next.as_ptr() as usize;
                        let aligned = align_up(addr, align);
                        let padding = aligned.saturating_sub(addr);
                        if padding + size <= next_node.size {
                            let new_addr = aligned;
                            let new_remainder = next_node.size - padding - size;
                            if new_remainder > size_of::<ListNode>() {
                                let mut new_next = unsafe {
                                    NonNull::new_unchecked((new_addr + size) as *mut ListNode)
                                };
                                unsafe {
                                    new_next.as_mut().size = new_remainder;
                                    new_next.as_mut().next = next_node.next;
                                }
                                next_node.size = padding;
                                next_node.next = Some(new_next);
                            } else {
                                node.next = next_node.next;
                            }
                            return NonNull::new(new_addr as *mut u8);
                        }
                    }
                    current = next;
                }
            }
        }
    }

    /// Returns a block to the free list (coalescing is deferred in M3).
    ///
    /// # Safety
    ///
    /// `ptr` must have been allocated by this allocator with `layout`.
    pub unsafe fn deallocate(&mut self, ptr: NonNull<u8>, layout: Layout) {
        let size = layout.size().max(size_of::<ListNode>());
        let mut node = NonNull::new_unchecked(ptr.as_ptr() as *mut ListNode);
        node.as_mut().size = size;
        node.as_mut().next = self.head.next;
        self.head.next = Some(node);
    }
}

/// Thread-safe wrapper implementing [`GlobalAlloc`].
pub struct LockedHeap(SpinMutex<LinkedListAllocator>);

impl LockedHeap {
    /// Empty heap; call [`init`] before use.
    pub const fn new() -> Self {
        Self(SpinMutex::new(LinkedListAllocator::new()))
    }
}

unsafe impl GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0
            .lock()
            .allocate_first_fit(layout)
            .map(NonNull::as_ptr)
            .unwrap_or(core::ptr::null_mut())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if let Some(non_null) = NonNull::new(ptr) {
            self.0.lock().deallocate(non_null, layout);
        }
    }
}

#[global_allocator]
static HEAP: LockedHeap = LockedHeap::new();

/// Initializes the global linked-list heap over the mapped heap region.
pub fn init() {
    // SAFETY: Paging mapped HEAP_VIRTUAL_START..+HEAP_SIZE as RW|NX.
    unsafe {
        HEAP.0.lock().init(HEAP_VIRTUAL_START as usize, HEAP_SIZE);
    }
    crate::serial::write_str("  heap: linked-list allocator ready\r\n");
}

const fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_fit_splits_large_block() {
        let mut alloc = LinkedListAllocator::new();
        let backing = [0u8; 256];
        let start = backing.as_ptr() as usize;
        unsafe {
            alloc.init(start, backing.len());
        }
        let layout = Layout::from_size_align(32, 8).unwrap();
        let first = alloc.allocate_first_fit(layout).expect("first alloc");
        let second = alloc.allocate_first_fit(layout).expect("second alloc");
        assert_ne!(first.as_ptr(), second.as_ptr());
    }
}
