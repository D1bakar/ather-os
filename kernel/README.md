# Kernel

> **Status:** Placeholder — implementation begins in **Milestone M1**.

The Aether kernel is a monolithic, Rust-native kernel for x86_64. It will:

1. Receive control from the UEFI boot loader.
2. Initialize serial output, GDT, and IDT.
3. Enter `kmain()` and print a boot banner.
4. Halt or enter an idle loop.

See [ADR 001](../docs/architecture/001-initial-decisions.md) for architecture decisions.

## M1 Deliverables

- `kernel/Cargo.toml` with `x86_64-unknown-none` target
- Kernel entry point and `kmain`
- Serial port driver (COM1) wired to `aether-logger`
- Minimal panic handler

## Directory Structure (M1+)

```
kernel/
├── src/
│   ├── main.rs          # Entry point, kmain
│   ├── arch/x86_64/     # GDT, IDT, CPU init
│   ├── mm/              # Memory management (M2)
│   ├── sched/           # Scheduler (M3)
│   ├── syscall/         # Syscall dispatch (M4)
│   └── fs/              # VFS (M5)
└── Cargo.toml
```
