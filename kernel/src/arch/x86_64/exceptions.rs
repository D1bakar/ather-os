//! CPU exception and interrupt stub routines for the x86_64 IDT.

use crate::serial;
use core::sync::atomic::{AtomicBool, Ordering};

use super::direct_call::DirectCallSlot;

/// Set once ring-3 init is about to run — gates bring-up exception logs.
static USER_ENTRY_ARMED: AtomicBool = AtomicBool::new(false);

/// Higher-half aliases for Rust dispatchers called from IDT stubs under user CR3.
static mut EXCEPTION_DISPATCH_VIRT: DirectCallSlot = DirectCallSlot::empty();
static mut INTERRUPT_DISPATCH_VIRT: DirectCallSlot = DirectCallSlot::empty();

/// Records direct-virt handler addresses for exception/interrupt stubs.
pub(super) fn init_dispatch_targets() {
    // SAFETY: BSP-only early init before user CR3 is active.
    unsafe {
        EXCEPTION_DISPATCH_VIRT.set(exception_dispatch as u64);
        INTERRUPT_DISPATCH_VIRT.set(interrupt_dispatch as u64);
    }
}

/// Arms ring-3 exception logging before the first user instruction runs.
pub fn arm_user_exception_logging() {
    USER_ENTRY_ARMED.store(true, Ordering::Relaxed);
}

/// Human-readable names for vectors 0–31 (CPU exceptions).
const EXCEPTION_NAMES: [&str; 32] = [
    "#DE divide error",
    "#DB debug",
    "NMI",
    "#BP breakpoint",
    "#OF overflow",
    "#BR bound range",
    "#UD invalid opcode",
    "#NM device not available",
    "#DF double fault",
    "#CS coprocessor segment overrun",
    "#TS invalid TSS",
    "#NP segment not present",
    "#SS stack segment fault",
    "#GP general protection",
    "#PF page fault",
    "reserved",
    "#MF x87 floating-point",
    "#AC alignment check",
    "#MC machine check",
    "#XM SIMD floating-point",
    "#VE virtualization",
    "#CP control protection",
    "reserved",
    "reserved",
    "reserved",
    "reserved",
    "reserved",
    "reserved",
    "#HV hypervisor",
    "#VC VMM communication",
    "#SX security",
    "#CP reserved",
];

core::arch::global_asm!(
    ".section .text",
    ".align 16",
    ".macro exception_no_ec vector",
    "push 0",
    "push \\vector",
    "jmp exception_common",
    ".endm",
    ".macro exception_with_ec vector",
    "push \\vector",
    "jmp exception_common",
    ".endm",
    ".macro irq_stub vector",
    "push 0",
    "push \\vector",
    "jmp interrupt_common",
    ".endm",
    "exception_common:",
    "mov rdi, [rsp]",
    "mov rsi, [rsp + 8]",
    "mov rdx, [rsp + 16]",
    "add rsp, 16",
    "mov rax, qword ptr [rip + {exception_dispatch}]",
    "call rax",
    "cli",
    "1:",
    "hlt",
    "jmp 1b",
    "interrupt_common:",
    "mov rdi, [rsp]",
    "mov rsi, [rsp + 8]",
    "add rsp, 16",
    "mov rax, qword ptr [rip + {interrupt_dispatch}]",
    "call rax",
    "cli",
    "2:",
    "hlt",
    "jmp 2b",
    ".globl exception_stub_0",
    "exception_stub_0:",
    "exception_no_ec 0",
    ".globl exception_stub_1",
    "exception_stub_1:",
    "exception_no_ec 1",
    ".globl exception_stub_2",
    "exception_stub_2:",
    "exception_no_ec 2",
    ".globl exception_stub_3",
    "exception_stub_3:",
    "exception_no_ec 3",
    ".globl exception_stub_4",
    "exception_stub_4:",
    "exception_no_ec 4",
    ".globl exception_stub_5",
    "exception_stub_5:",
    "exception_no_ec 5",
    ".globl exception_stub_6",
    "exception_stub_6:",
    "exception_no_ec 6",
    ".globl exception_stub_7",
    "exception_stub_7:",
    "exception_no_ec 7",
    ".globl exception_stub_8",
    "exception_stub_8:",
    "exception_with_ec 8",
    ".globl exception_stub_10",
    "exception_stub_10:",
    "exception_with_ec 10",
    ".globl exception_stub_11",
    "exception_stub_11:",
    "exception_with_ec 11",
    ".globl exception_stub_12",
    "exception_stub_12:",
    "exception_with_ec 12",
    ".globl exception_stub_13",
    "exception_stub_13:",
    "exception_with_ec 13",
    ".globl exception_stub_14",
    "exception_stub_14:",
    "exception_with_ec 14",
    ".globl exception_stub_16",
    "exception_stub_16:",
    "exception_no_ec 16",
    ".globl exception_stub_17",
    "exception_stub_17:",
    "exception_with_ec 17",
    ".globl exception_stub_18",
    "exception_stub_18:",
    "exception_no_ec 18",
    ".globl exception_stub_19",
    "exception_stub_19:",
    "exception_no_ec 19",
    ".globl exception_stub_20",
    "exception_stub_20:",
    "exception_no_ec 20",
    ".globl exception_stub_21",
    "exception_stub_21:",
    "exception_no_ec 21",
    ".globl exception_stub_22",
    "exception_stub_22:",
    "exception_no_ec 22",
    ".globl exception_stub_23",
    "exception_stub_23:",
    "exception_no_ec 23",
    ".globl exception_stub_24",
    "exception_stub_24:",
    "exception_no_ec 24",
    ".globl exception_stub_25",
    "exception_stub_25:",
    "exception_no_ec 25",
    ".globl exception_stub_26",
    "exception_stub_26:",
    "exception_no_ec 26",
    ".globl exception_stub_27",
    "exception_stub_27:",
    "exception_no_ec 27",
    ".globl exception_stub_28",
    "exception_stub_28:",
    "exception_no_ec 28",
    ".globl exception_stub_29",
    "exception_stub_29:",
    "exception_no_ec 29",
    ".globl exception_stub_30",
    "exception_stub_30:",
    "exception_no_ec 30",
    ".globl exception_stub_31",
    "exception_stub_31:",
    "exception_no_ec 31",
    ".globl interrupt_stub_32",
    "interrupt_stub_32:",
    "irq_stub 32",
    ".globl interrupt_stub_33",
    "interrupt_stub_33:",
    "irq_stub 33",
    ".globl interrupt_stub_34",
    "interrupt_stub_34:",
    "irq_stub 34",
    ".globl interrupt_stub_35",
    "interrupt_stub_35:",
    "irq_stub 35",
    ".globl interrupt_stub_36",
    "interrupt_stub_36:",
    "irq_stub 36",
    ".globl interrupt_stub_37",
    "interrupt_stub_37:",
    "irq_stub 37",
    ".globl interrupt_stub_38",
    "interrupt_stub_38:",
    "irq_stub 38",
    ".globl interrupt_stub_39",
    "interrupt_stub_39:",
    "irq_stub 39",
    ".globl interrupt_stub_40",
    "interrupt_stub_40:",
    "irq_stub 40",
    ".globl interrupt_stub_41",
    "interrupt_stub_41:",
    "irq_stub 41",
    ".globl interrupt_stub_42",
    "interrupt_stub_42:",
    "irq_stub 42",
    ".globl interrupt_stub_43",
    "interrupt_stub_43:",
    "irq_stub 43",
    ".globl interrupt_stub_44",
    "interrupt_stub_44:",
    "irq_stub 44",
    ".globl interrupt_stub_45",
    "interrupt_stub_45:",
    "irq_stub 45",
    ".globl interrupt_stub_46",
    "interrupt_stub_46:",
    "irq_stub 46",
    ".globl interrupt_stub_47",
    "interrupt_stub_47:",
    "irq_stub 47",
    exception_dispatch = sym EXCEPTION_DISPATCH_VIRT,
    interrupt_dispatch = sym INTERRUPT_DISPATCH_VIRT,
);

/// Returns the address of the assembly stub for CPU exception `vector` (0–31).
pub(super) fn exception_stub_addr(vector: u8) -> u64 {
    match vector {
        0 => exception_stub_0 as u64,
        1 => exception_stub_1 as u64,
        2 => exception_stub_2 as u64,
        3 => exception_stub_3 as u64,
        4 => exception_stub_4 as u64,
        5 => exception_stub_5 as u64,
        6 => exception_stub_6 as u64,
        7 => exception_stub_7 as u64,
        8 => exception_stub_8 as u64,
        10 => exception_stub_10 as u64,
        11 => exception_stub_11 as u64,
        12 => exception_stub_12 as u64,
        13 => exception_stub_13 as u64,
        14 => exception_stub_14 as u64,
        16 => exception_stub_16 as u64,
        17 => exception_stub_17 as u64,
        18 => exception_stub_18 as u64,
        19 => exception_stub_19 as u64,
        20 => exception_stub_20 as u64,
        21 => exception_stub_21 as u64,
        22 => exception_stub_22 as u64,
        23 => exception_stub_23 as u64,
        24 => exception_stub_24 as u64,
        25 => exception_stub_25 as u64,
        26 => exception_stub_26 as u64,
        27 => exception_stub_27 as u64,
        28 => exception_stub_28 as u64,
        29 => exception_stub_29 as u64,
        30 => exception_stub_30 as u64,
        31 => exception_stub_31 as u64,
        _ => exception_stub_6 as u64, // #UD for reserved vectors 9, 15, etc.
    }
}

/// Returns the address of the assembly stub for hardware interrupt `vector` (32–47).
///
/// Vectors 48–255 reuse the vector-47 stub until dedicated handlers are added.
pub(super) fn interrupt_stub_addr(vector: u8) -> u64 {
    match vector {
        32 => interrupt_stub_32 as u64,
        33 => interrupt_stub_33 as u64,
        34 => interrupt_stub_34 as u64,
        35 => interrupt_stub_35 as u64,
        36 => interrupt_stub_36 as u64,
        37 => interrupt_stub_37 as u64,
        38 => interrupt_stub_38 as u64,
        39 => interrupt_stub_39 as u64,
        40 => interrupt_stub_40 as u64,
        41 => interrupt_stub_41 as u64,
        42 => interrupt_stub_42 as u64,
        43 => interrupt_stub_43 as u64,
        44 => interrupt_stub_44 as u64,
        45 => interrupt_stub_45 as u64,
        46 => interrupt_stub_46 as u64,
        47 => interrupt_stub_47 as u64,
        _ => interrupt_stub_47 as u64,
    }
}

extern "C" {
    fn exception_stub_0();
    fn exception_stub_1();
    fn exception_stub_2();
    fn exception_stub_3();
    fn exception_stub_4();
    fn exception_stub_5();
    fn exception_stub_6();
    fn exception_stub_7();
    fn exception_stub_8();
    fn exception_stub_10();
    fn exception_stub_11();
    fn exception_stub_12();
    fn exception_stub_13();
    fn exception_stub_14();
    fn exception_stub_16();
    fn exception_stub_17();
    fn exception_stub_18();
    fn exception_stub_19();
    fn exception_stub_20();
    fn exception_stub_21();
    fn exception_stub_22();
    fn exception_stub_23();
    fn exception_stub_24();
    fn exception_stub_25();
    fn exception_stub_26();
    fn exception_stub_27();
    fn exception_stub_28();
    fn exception_stub_29();
    fn exception_stub_30();
    fn exception_stub_31();
    fn interrupt_stub_32();
    fn interrupt_stub_33();
    fn interrupt_stub_34();
    fn interrupt_stub_35();
    fn interrupt_stub_36();
    fn interrupt_stub_37();
    fn interrupt_stub_38();
    fn interrupt_stub_39();
    fn interrupt_stub_40();
    fn interrupt_stub_41();
    fn interrupt_stub_42();
    fn interrupt_stub_43();
    fn interrupt_stub_44();
    fn interrupt_stub_45();
    fn interrupt_stub_46();
    fn interrupt_stub_47();
}

/// Dispatched from assembly for CPU exceptions (vectors 0–31).
#[no_mangle]
extern "C" fn exception_dispatch(vector: u64, error_code: u64, fault_rip: u64) -> ! {
    log_ring3_exception_context(fault_rip);
    log_user_bringup_exception(vector, fault_rip);
    match vector {
        0 => handle_divide_error(error_code),
        3 => handle_breakpoint(error_code),
        6 => handle_invalid_opcode(error_code),
        8 => handle_double_fault(error_code),
        13 => handle_general_protection(error_code),
        14 => handle_page_fault(error_code),
        _ => handle_exception(vector, error_code),
    }
}

/// Dispatched from assembly for hardware IRQs (vectors 32+).
#[no_mangle]
extern "C" fn interrupt_dispatch(vector: u64, _error_code: u64) -> ! {
    handle_unhandled_interrupt(vector)
}

fn handle_divide_error(error_code: u64) -> ! {
    serial::write_str("EXCEPTION: #DE divide error\r\n");
    write_error_code(error_code);
    halt_forever();
}

fn handle_breakpoint(error_code: u64) -> ! {
    serial::write_str("EXCEPTION: #BP breakpoint\r\n");
    write_error_code(error_code);
    halt_forever();
}

fn handle_invalid_opcode(error_code: u64) -> ! {
    serial::write_str("EXCEPTION: #UD invalid opcode\r\n");
    serial::write_str("  CR3: ");
    write_hex_u64(read_cr3());
    serial::write_str("\r\n");
    write_error_code(error_code);
    halt_forever();
}

fn handle_double_fault(error_code: u64) -> ! {
    serial::write_str("EXCEPTION: #DF double fault\r\n");
    write_error_code(error_code);
    halt_forever();
}

fn handle_general_protection(error_code: u64) -> ! {
    serial::write_str("EXCEPTION: #GP general protection\r\n");
    serial::write_str("  CR3: ");
    write_hex_u64(read_cr3());
    serial::write_str("\r\n");
    write_error_code(error_code);
    halt_forever();
}

fn handle_page_fault(error_code: u64) -> ! {
    let fault_addr = read_cr2();
    serial::write_str("EXCEPTION: #PF page fault\r\n");
    serial::write_str("  CR2 (fault address): ");
    write_hex_u64(fault_addr);
    serial::write_str("\r\n  CR3: ");
    write_hex_u64(read_cr3());
    serial::write_str("\r\n  error code: ");
    write_hex_u64(error_code);
    serial::write_str("\r\n");
    halt_forever();
}

fn handle_exception(vector: u64, error_code: u64) -> ! {
    serial::write_str("EXCEPTION: ");
    if vector < 32 {
        serial::write_str(EXCEPTION_NAMES[vector as usize]);
    } else {
        serial::write_str("unknown");
    }
    serial::write_str("\r\n  vector: ");
    write_hex_u64(vector);
    serial::write_str("\r\n");
    write_error_code(error_code);
    halt_forever();
}

fn handle_unhandled_interrupt(vector: u64) -> ! {
    serial::write_str("UNHANDLED INTERRUPT\r\n  vector: ");
    write_hex_u64(vector);
    serial::write_str("\r\n");
    halt_forever();
}

fn log_user_bringup_exception(vector: u64, fault_rip: u64) {
    if !USER_ENTRY_ARMED.load(Ordering::Relaxed) {
        return;
    }
    let label = match vector {
        6 => Some("#UD"),
        13 => Some("#GP"),
        14 => Some("#PF"),
        _ => None,
    };
    if let Some(name) = label {
        serial::write_str("[user-exn] ");
        serial::write_str(name);
        serial::write_str(" RIP=0x");
        write_hex_u64(fault_rip);
        serial::write_str("\r\n");
    }
}

/// Logs ring-3 fault context (RIP/CS/CR3) for bring-up diagnostics.
fn log_ring3_exception_context(fault_rip: u64) {
    let cs: u64;
    // SAFETY: Reading CS is valid in the exception handler.
    unsafe {
        core::arch::asm!("mov {0:x}, cs", out(reg) cs, options(nomem, nostack));
    }
    if (cs & 3) != 3 {
        return;
    }
    serial::write_str("[user] exception in ring 3\r\n  RIP: ");
    write_hex_u64(fault_rip);
    serial::write_str("\r\n  CS: ");
    write_hex_u64(cs);
    serial::write_str("\r\n  CR3: ");
    write_hex_u64(read_cr3());
    serial::write_str("\r\n");
}

fn write_error_code(error_code: u64) {
    serial::write_str("  error code: ");
    write_hex_u64(error_code);
    serial::write_str("\r\n");
}

fn write_hex_u64(value: u64) {
    serial::write_str("0x");
    let mut started = false;
    for shift in (0..64).step_by(4).rev() {
        let nibble = ((value >> shift) & 0xF) as u8;
        if nibble != 0 || started || shift == 0 {
            started = true;
            let digit = if nibble < 10 { b'0' + nibble } else { b'a' + (nibble - 10) };
            write_byte(digit);
        }
    }
}

fn write_byte(byte: u8) {
    let mut buf = [0u8; 1];
    buf[0] = byte;
    serial::write_str(core::str::from_utf8(&buf).unwrap_or("?"));
}

fn read_cr2() -> u64 {
    let value: u64;
    // SAFETY: Reading CR2 is safe at any time; it returns the faulting linear address.
    unsafe {
        core::arch::asm!("mov {}, cr2", out(reg) value, options(nomem, nostack));
    }
    value
}

fn read_cr3() -> u64 {
    let value: u64;
    // SAFETY: Reading CR3 is always valid in ring 0.
    unsafe {
        core::arch::asm!("mov {}, cr3", out(reg) value, options(nomem, nostack));
    }
    value
}

fn halt_forever() -> ! {
    loop {
        core::hint::spin_loop();
    }
}
