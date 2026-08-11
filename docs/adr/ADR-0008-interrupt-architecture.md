# ADR-0008: x86_64 Interrupt and Timer Architecture

**Status:** Accepted — **M2 shipped** (PIC + PIT); APIC migration planned  
**Date:** 2026-08-11  
**Milestone:** M2 (CPU bring-up), M4 (APIC timer + preemption)

## Context

After M1, the kernel ran with interrupts disabled and no GDT/IDT. M2 requires exception
handling, device IRQ delivery, and a periodic timer for tick accounting and future scheduling.

x86_64 platforms expose legacy 8259 PIC, I/O APIC, and local APIC. Aether OS targets QEMU
`q35` and modern UEFI PCs. M2 uses the **legacy PIC + PIT** path for minimal bring-up; APIC
is deferred until ACPI RSDP is populated in `BootInfo`.

## Decision

### GDT (M2 — shipped)

Install a minimal **64-bit flat GDT** before IDT setup:

| Index | Segment | Purpose |
|-------|---------|---------|
| 0 | Null | Required null descriptor |
| 1 | Kernel code (64-bit, DPL 0) | Exception and IRQ entry |
| 2 | Kernel data (64-bit, DPL 0) | Long-mode data segment |
| 5–6 | TSS (64-bit) | Placeholder for future IST stacks |

Implementation: `kernel/src/arch/x86_64/gdt.rs`, `gdt/layout.rs`.

### IDT (M2 — shipped)

**256-entry IDT** with 64-bit interrupt gates (DPL 0):

| Vector range | Handler behavior | Status |
|--------------|------------------|--------|
| 0–31 | CPU exceptions — serial log, halt | **Shipped** |
| 32 | Timer IRQ (PIT / IRQ 0 after PIC remap) | **Shipped** |
| 33–47 | Generic unhandled IRQ stub | **Shipped** (stub) |
| 48–255 | Reuse vector 47 stub | **Shipped** (stub) |

Key exceptions: #DE, #UD, #GP, #PF (with CR2), #DF — each prints diagnostics to COM1.

Implementation: `kernel/src/arch/x86_64/idt.rs`, `exceptions.rs`.

### Interrupt controller (M2 — shipped: legacy PIC)

1. **8259 PIC** remapped so hardware IRQ 0–15 map to CPU vectors 32–47.
2. All IRQ lines masked on init; timer unmasks IRQ 0 only.
3. **EOI** sent to master (and slave when applicable) from the timer handler.

**Future (M4+):** Disable legacy PIC; enable local APIC + I/O APIC using `BootInfo` ACPI RSDP.

Implementation: `kernel/src/arch/x86_64/pic.rs`.

### Timer (M2 — shipped: PIT)

| Phase | Source | Rate | Purpose | Status |
|-------|--------|------|---------|--------|
| M1 | None | — | Busy spin / HLT | Shipped |
| M2 | PIT channel 0 (IRQ 0) | ~100 Hz | Tick counter, serial diagnostics | **Shipped** |
| M4 | Local APIC timer | ~100 Hz | Preemptive scheduler | Planned (PIT preemption **shipped**) |

- Divisor: `1193182 / 100` (integer rounding → ~100.003 Hz effective).
- Tick counter: `AtomicU64`; serial log every 100 ticks (~1 s).

Implementation: `kernel/src/arch/x86_64/timer.rs`, `interrupts.rs`.

### Boot init sequence (M2 — shipped)

1. Serial init
2. Validate `BootInfo`
3. `gdt::init()` → `idt::init()` → `init_interrupts()` (PIC + timer handler + PIT)
4. `enable_interrupts()` (`STI`)
5. Idle loop with `HLT`

Expected serial: `Aether OS M2: GDT/IDT/interrupts initialized` then `[timer] tick N`.

### Interrupt handling discipline

- Handlers are short — no allocation in IRQ context in M2.
- Unexpected CPU exceptions halt after logging.
- Unhandled device IRQs log vector and halt (only timer is unmasked in M2).

## Consequences

### Positive

- Minimal dependencies — works in QEMU without ACPI table parsing.
- Host-testable GDT/IDT descriptor encoding (`tests/arch_gdt.rs`, `tests/arch_idt.rs`).
- Clear migration path to APIC documented below.

### Negative

- Legacy PIC is deprecated on modern hardware; real-PC support may require APIC sooner.
- `BootInfo.rsdp = 0` in M1 boot loader limits APIC auto-config until boot loader follow-up.
- Double-fault has no dedicated IST stack until M4 user-mode work.

### Follow-ups

- Boot loader: populate `BootInfo.rsdp` from UEFI ACPI tables (M3).
- `kernel/src/arch/x86_64/apic.rs` — local APIC init and timer (M4).
- Wire timer IRQ to scheduler (**M4 shipped** with PIT); context switch in `switch.rs` (**M4 shipped**).
- Local APIC timer replaces PIT for preemption (follow-up).

## References

- [Intel SDM Volume 3](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html)
- [ARCHITECTURE.md](../../ARCHITECTURE.md) — CPU and interrupt architecture
- [ADR-0003](ADR-0003-initial-target-hardware.md) — QEMU q35 target
- [ADR-0006](ADR-0006-boot-architecture.md) — boot handoff order
