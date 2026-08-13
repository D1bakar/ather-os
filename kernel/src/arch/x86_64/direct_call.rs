//! Higher-half aliases for kernel functions invoked from assembly while user CR3 is active.
//!
//! IDT and SYSCALL stubs are installed at [`crate::mm::link_to_direct_virt`] addresses so
//! they remain reachable under a user page table. Rip-relative `call handler` targets still
//! resolve to link-time (identity-map) addresses and fault unless redirected here.

/// Slot holding the higher-half alias of a link-time kernel function pointer.
#[repr(C, align(8))]
pub struct DirectCallSlot(u64);

impl DirectCallSlot {
    /// Empty slot — must be filled before the corresponding stub runs under user CR3.
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Stores the higher-half alias of `link_addr`.
    pub fn set(&mut self, link_addr: u64) {
        self.0 = crate::mm::link_to_direct_virt(link_addr);
    }

    /// Returns the stored higher-half entry address.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::KERNEL_VIRT_BASE;

    fn sample_fn() {}

    #[test]
    fn direct_call_slot_adds_kernel_virt_base() {
        let link = sample_fn as *const () as u64;
        let mut slot = DirectCallSlot::empty();
        slot.set(link);
        assert_eq!(slot.get(), link.wrapping_add(KERNEL_VIRT_BASE));
    }
}
