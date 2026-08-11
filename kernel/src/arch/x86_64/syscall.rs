//! SYSCALL/SYSRET entry via model-specific registers (preferred x86_64 path).

use super::gdt::layout::{KERNEL_CODE_SELECTOR, USER_DATA_SELECTOR};
use aether_abi::SyscallArgs;

/// IA32_EFER — extended feature enable register.
const MSR_EFER: u32 = 0xC000_0080;
/// IA32_STAR — segment selectors for SYSCALL/SYSRET.
const MSR_STAR: u32 = 0xC000_0081;
/// IA32_LSTAR — 64-bit SYSCALL target RIP.
const MSR_LSTAR: u32 = 0xC000_0082;
/// IA32_FMASK — RFLAGS mask applied on SYSCALL entry.
const MSR_FMASK: u32 = 0xC000_0084;

/// EFER.SCE — enable SYSCALL/SYSRET in 64-bit mode.
const EFER_SCE: u64 = 1 << 0;
/// Clear IF (bit 9) on syscall entry so the handler runs with interrupts off.
const FMASK_IF: u64 = 1 << 9;

/// Scratch space for the per-CPU kernel stack pointer loaded on syscall entry.
static mut SYSCALL_KERNEL_STACK: u64 = 0;

core::arch::global_asm!(
    ".global syscall_entry_stub",
    "syscall_entry_stub:",
    // Hardware maps RIP→RCX, RFLAGS→R11 on SYSCALL entry.
    "swapgs",
    "mov gs:[0], rsp",
    "mov rsp, gs:[8]",
    "push 0x23",
    "push rcx",
    "push 0x202",
    "push 0x1B",
    "push r11",
    "push r15",
    "push r14",
    "push r13",
    "push r12",
    "push rbp",
    "push rbx",
    "push r10",
    "push rdx",
    "push rsi",
    "push rdi",
    "push r8",
    "push r9",
    "push rax",
    "mov rdi, rsp",
    "call syscall_dispatch_rust",
    "mov [rsp], rax",
    "pop rax",
    "pop r9",
    "pop r8",
    "pop rdi",
    "pop rsi",
    "pop rdx",
    "pop r10",
    "pop rbx",
    "pop rbp",
    "pop r12",
    "pop r13",
    "pop r14",
    "pop r15",
    "pop r11",
    "add rsp, 32",
    "pop rsp",
    "swapgs",
    "sysretq",
);

extern "sysv64" {
    /// Low-level SYSCALL entry trampoline; address loaded into IA32_LSTAR.
    pub fn syscall_entry_stub();
}

/// Installs SYSCALL MSRs and per-CPU scratch GS slots.
///
/// Must run after the GDT contains user and kernel segments and a kernel stack
/// has been assigned for syscall handlers.
pub fn init(kernel_stack_top: u64) {
    // SAFETY: BSP-only early init; MSRs are not yet accessed concurrently.
    unsafe {
        SYSCALL_KERNEL_STACK = kernel_stack_top;
        install_msrs();
        install_gs_scratch();
    }
}

/// Returns the address of the SYSCALL entry stub (for IDT-less MSR path tests).
#[cfg(test)]
#[must_use]
pub fn entry_stub_address() -> u64 {
    syscall_entry_stub as u64
}

#[cfg(all(not(feature = "host-stub"), target_arch = "x86_64"))]
unsafe fn install_msrs() {
    let efer = read_msr(MSR_EFER);
    write_msr(MSR_EFER, efer | EFER_SCE);

    // SYSRET: CS = STAR[63:48]+16, SS = STAR[63:48]+8
    let star_base = (USER_DATA_SELECTOR as u64).wrapping_sub(8);
    let star = (star_base << 48) | ((KERNEL_CODE_SELECTOR as u64) << 32);
    write_msr(MSR_STAR, star);
    write_msr(MSR_LSTAR, syscall_entry_stub as u64);
    write_msr(MSR_FMASK, FMASK_IF);
}

#[cfg(all(not(feature = "host-stub"), target_arch = "x86_64"))]
unsafe fn install_gs_scratch() {
    // gs:[0] = saved user RSP, gs:[8] = kernel RSP for syscall entry
    static mut SCRATCH: [u64; 2] = [0; 2];
    SCRATCH[1] = SYSCALL_KERNEL_STACK;
    write_gs_base(core::ptr::addr_of!(SCRATCH) as u64);
}

#[cfg(all(not(feature = "host-stub"), target_arch = "x86_64"))]
unsafe fn write_gs_base(base: u64) {
    let low = base as u32;
    let high = (base >> 32) as u32;
    // IA32_KERNEL_GS_BASE
    write_msr(0xC000_0102, base);
    // Also set IA32_GS_BASE for the syscall swapgs path
    core::arch::asm!(
        "wrmsr",
        in("ecx") 0xC000_0101u32,
        in("eax") low,
        in("edx") high,
        options(nomem, nostack)
    );
}

#[cfg(all(not(feature = "host-stub"), target_arch = "x86_64"))]
unsafe fn read_msr(msr: u32) -> u64 {
    let low: u32;
    let high: u32;
    core::arch::asm!(
        "rdmsr",
        in("ecx") msr,
        out("eax") low,
        out("edx") high,
        options(nomem, nostack)
    );
    u64::from(high) << 32 | u64::from(low)
}

#[cfg(all(not(feature = "host-stub"), target_arch = "x86_64"))]
unsafe fn write_msr(msr: u32, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;
    core::arch::asm!(
        "wrmsr",
        in("ecx") msr,
        in("eax") low,
        in("edx") high,
        options(nomem, nostack)
    );
}

/// Builds the IA32_STAR value for the installed user/kernel selectors.
#[cfg(test)]
#[must_use]
pub const fn star_msr_value(kernel_cs: u16, user_data_selector: u16) -> u64 {
    let star_base = (user_data_selector as u64).wrapping_sub(8);
    (star_base << 48) | ((kernel_cs as u64) << 32)
}

/// Rust-side syscall demux called from the assembly stub.
#[no_mangle]
extern "C" fn syscall_dispatch_rust(frame: *const SyscallTrapFrame) -> i64 {
    // SAFETY: The assembly stub passes a valid frame pointer on the kernel stack.
    let frame = unsafe { &*frame };
    let args = SyscallArgs::new(frame.rdi, frame.rsi, frame.rdx, frame.r10, frame.r8, frame.r9);
    crate::syscall::dispatch(frame.rax, args)
}

/// Saved register frame pushed by [`syscall_entry_stub`].
#[repr(C)]
pub struct SyscallTrapFrame {
    /// Syscall number (RAX on entry).
    pub rax: u64,
    /// Argument / preserved registers.
    pub r9: u64,
    pub r8: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub r10: u64,
    pub rbx: u64,
    pub rbp: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub r11: u64,
    pub user_cs: u64,
    pub user_rflags: u64,
    pub user_rip: u64,
    pub user_ss: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arch::x86_64::gdt::layout::{
        KERNEL_CODE_SELECTOR, USER_CODE_SELECTOR, USER_DATA_SELECTOR,
    };

    #[test]
    fn star_msr_encodes_sysret_selectors() {
        let star = star_msr_value(KERNEL_CODE_SELECTOR, USER_DATA_SELECTOR);
        let base = (star >> 48) as u16;
        assert_eq!(base.wrapping_add(8), USER_DATA_SELECTOR);
        assert_eq!(base.wrapping_add(16), USER_CODE_SELECTOR);
    }
}
