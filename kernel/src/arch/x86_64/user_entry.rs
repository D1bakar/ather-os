//! Ring-3 entry via `IRETQ` (first transition from kernel to user mode).

use super::gdt::layout::{USER_CODE_SELECTOR, USER_DATA_SELECTOR};

core::arch::global_asm!(
    ".global enter_user_mode",
    "enter_user_mode:",
    // rdi = user RIP, rsi = user RSP, rdx = CR3
    "mov cr3, rdx",
    "push {user_ss}",
    "push rsi",
    "push 0x202",
    "push {user_cs}",
    "push rdi",
    "iretq",
    user_ss = const USER_DATA_SELECTOR as u32,
    user_cs = const USER_CODE_SELECTOR as u32,
);

extern "sysv64" {
    /// Loads `cr3`, then `IRETQ` to ring 3 at `user_rip` with `user_rsp`.
    ///
    /// # Safety
    ///
    /// `user_rip` and `user_rsp` must lie in mapped user pages; `cr3` must be a
    /// valid page-table root with those mappings and the kernel higher-half map.
    pub fn enter_user_mode(user_rip: u64, user_rsp: u64, cr3: u64) -> !;
}
