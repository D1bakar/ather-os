//! Saved CPU register frame for context switches.
//!
//! Offsets are shared with the assembly in [`switch_context`] so host tests can
//! verify layout without executing bare-metal code.

/// Byte offset of `CpuContext::rbx` (used by assembly).
pub const CTX_RBX: usize = 0;
/// Byte offset of `CpuContext::rbp`.
pub const CTX_RBP: usize = 8;
/// Byte offset of `CpuContext::r12`.
pub const CTX_R12: usize = 16;
/// Byte offset of `CpuContext::r13`.
pub const CTX_R13: usize = 24;
/// Byte offset of `CpuContext::r14`.
pub const CTX_R14: usize = 32;
/// Byte offset of `CpuContext::r15`.
pub const CTX_R15: usize = 40;
/// Byte offset of `CpuContext::rsp`.
pub const CTX_RSP: usize = 48;
/// Byte offset of `CpuContext::rip`.
pub const CTX_RIP: usize = 56;
/// Byte offset of `CpuContext::cr3`.
pub const CTX_CR3: usize = 64;
/// Total size of a context frame in bytes.
pub const CTX_SIZE: usize = 72;

/// Callee-saved GPRs plus instruction pointer, stack pointer, and page-table root.
///
/// Volatile registers are not preserved across a voluntary switch; preemption
/// stubs will extend this frame in a follow-on milestone.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CpuContext {
    /// Callee-saved register RBX.
    pub rbx: u64,
    /// Frame pointer RBP.
    pub rbp: u64,
    /// Callee-saved register R12.
    pub r12: u64,
    /// Callee-saved register R13.
    pub r13: u64,
    /// Callee-saved register R14.
    pub r14: u64,
    /// Callee-saved register R15.
    pub r15: u64,
    /// Stack pointer at switch time.
    pub rsp: u64,
    /// Instruction pointer to resume at.
    pub rip: u64,
    /// CR3 value (physical address of PML4) for the target address space.
    pub cr3: u64,
}

impl CpuContext {
    /// Builds an initial context that begins execution at `entry` using `stack_top`.
    #[must_use]
    pub const fn for_entry(entry: u64, stack_top: u64, cr3: u64) -> Self {
        Self { rbx: 0, rbp: 0, r12: 0, r13: 0, r14: 0, r15: 0, rsp: stack_top, rip: entry, cr3 }
    }

    /// Reads the currently loaded CR3 (page-table root).
    #[cfg(all(not(feature = "host-stub"), target_arch = "x86_64"))]
    pub fn current_cr3() -> u64 {
        let value: u64;
        // SAFETY: Reading CR3 is always valid in ring 0.
        unsafe {
            core::arch::asm!("mov {}, cr3", out(reg) value, options(nomem, nostack));
        }
        value
    }

    /// Host stub — no page tables on the CI runner.
    #[cfg(feature = "host-stub")]
    pub fn current_cr3() -> u64 {
        0
    }

    /// Captures CR3 into this context (bare-metal helper).
    #[cfg(all(not(feature = "host-stub"), target_arch = "x86_64"))]
    pub fn capture_cr3(&mut self) {
        self.cr3 = Self::current_cr3();
    }
}

#[cfg(all(not(feature = "host-stub"), target_arch = "x86_64"))]
core::arch::global_asm!(
    ".global switch_context",
    "switch_context:",
    // rdi = current, rsi = next
    "mov [rdi + {rbx}], rbx",
    "mov [rdi + {rbp}], rbp",
    "mov [rdi + {r12}], r12",
    "mov [rdi + {r13}], r13",
    "mov [rdi + {r14}], r14",
    "mov [rdi + {r15}], r15",
    "mov [rdi + {rsp}], rsp",
    "mov rax, [rsp]",
    "mov [rdi + {rip}], rax",
    "mov rax, cr3",
    "mov [rdi + {cr3}], rax",
    "mov rbx, [rsi + {rbx}]",
    "mov rbp, [rsi + {rbp}]",
    "mov r12, [rsi + {r12}]",
    "mov r13, [rsi + {r13}]",
    "mov r14, [rsi + {r14}]",
    "mov r15, [rsi + {r15}]",
    "mov rsp, [rsi + {rsp}]",
    "mov rax, [rsi + {cr3}]",
    "mov cr3, rax",
    "mov rax, [rsi + {rip}]",
    "push rax",
    "ret",
    rbx = const CTX_RBX,
    rbp = const CTX_RBP,
    r12 = const CTX_R12,
    r13 = const CTX_R13,
    r14 = const CTX_R14,
    r15 = const CTX_R15,
    rsp = const CTX_RSP,
    rip = const CTX_RIP,
    cr3 = const CTX_CR3,
);

#[cfg(all(not(feature = "host-stub"), target_arch = "x86_64"))]
extern "sysv64" {
    /// Saves the current CPU state into `current` and restores from `next`.
    ///
    /// # Safety
    ///
    /// Both pointers must refer to valid, aligned [`CpuContext`] storage.
    /// `next` must contain a valid `rsp`, `rip`, and `cr3`.
    pub fn switch_context(current: *mut CpuContext, next: *const CpuContext);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_layout_matches_asm_offsets() {
        assert_eq!(core::mem::offset_of!(CpuContext, rbx), CTX_RBX);
        assert_eq!(core::mem::offset_of!(CpuContext, rbp), CTX_RBP);
        assert_eq!(core::mem::offset_of!(CpuContext, r12), CTX_R12);
        assert_eq!(core::mem::offset_of!(CpuContext, r13), CTX_R13);
        assert_eq!(core::mem::offset_of!(CpuContext, r14), CTX_R14);
        assert_eq!(core::mem::offset_of!(CpuContext, r15), CTX_R15);
        assert_eq!(core::mem::offset_of!(CpuContext, rsp), CTX_RSP);
        assert_eq!(core::mem::offset_of!(CpuContext, rip), CTX_RIP);
        assert_eq!(core::mem::offset_of!(CpuContext, cr3), CTX_CR3);
        assert_eq!(core::mem::size_of::<CpuContext>(), CTX_SIZE);
    }

    #[test]
    fn for_entry_sets_stack_and_rip() {
        let ctx = CpuContext::for_entry(0x1000, 0x2000, 0x3000);
        assert_eq!(ctx.rip, 0x1000);
        assert_eq!(ctx.rsp, 0x2000);
        assert_eq!(ctx.cr3, 0x3000);
    }
}
