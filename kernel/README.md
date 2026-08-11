# Kernel (`aether-kernel`)

> **Status:** M0 host stub — bare-metal entry planned for **M1**.

The Aether kernel is a modular monolithic, Rust-native kernel for x86_64. M0 ships
a host-buildable library crate so CI can compile and test workspace members. M1
replaces the stub with `#![no_std]` entry, panic handler, and serial output.

## M0 (current)

- Crate: `aether-kernel` with `host-stub` feature (default)
- Exports version constants and documents planned `kmain` API
- Runs unit tests on the host via `cargo test -p aether-kernel`

## M1 deliverables

- `#![no_std]` kernel with `x86_64-unknown-none` target
- Kernel entry point receiving `BootInfo` from the UEFI boot loader
- Serial port (COM1) wired to `aether-logger`
- Minimal panic handler and boot banner

## Build

```bash
# M0 — host (CI default)
cargo build -p aether-kernel
cargo test -p aether-kernel

# M1 — bare metal
rustup target add x86_64-unknown-none
cargo build -p aether-kernel --no-default-features --target x86_64-unknown-none
```

## Planned directory layout (M1+)

```
kernel/
├── src/
│   ├── lib.rs           # Public kernel API (M0)
│   ├── main.rs          # Entry point, kmain (M1)
│   ├── arch/x86_64/     # GDT, IDT, CPU init
│   ├── mm/              # Memory management (M2)
│   ├── sched/           # Scheduler (M3)
│   ├── syscall/         # Syscall dispatch (M4)
│   └── fs/              # VFS (M5)
└── Cargo.toml
```

## Related documents

- [ARCHITECTURE.md](../ARCHITECTURE.md)
- [ADR-0001](../docs/adr/ADR-0001-modular-monolithic-kernel.md)
- [ADR-0006](../docs/adr/ADR-0006-boot-architecture.md)
