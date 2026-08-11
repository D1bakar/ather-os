//! User virtual address space bounds.

/// Lowest canonical user virtual address (inclusive).
pub const USER_SPACE_MIN: u64 = 0x0000_0000_0000_1000;
/// Highest user virtual address (exclusive upper bound for validation).
pub const USER_SPACE_MAX: u64 = 0x0000_7FFF_FFFF_F000;
/// Typical user stack top (design intent).
pub const USER_STACK_TOP: u64 = USER_SPACE_MAX;

/// Returns `true` if `addr` is in the canonical user range.
#[must_use]
pub const fn is_user_address(addr: u64) -> bool {
    addr >= USER_SPACE_MIN && addr < USER_SPACE_MAX
}

/// Returns `true` if `addr` is in the canonical kernel half.
#[must_use]
pub const fn is_kernel_address(addr: u64) -> bool {
    addr >= 0xFFFF_8000_0000_0000
}
