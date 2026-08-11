# QEMU AArch64 (`virt`) — M13 scaffold

This document describes the **planned** AArch64 development target for Aether OS.
It does **not** describe shipped behavior: there is no AArch64 boot loader, linker
script, or CI job yet.

## Status

| Item | Status |
|------|--------|
| `kernel/src/arch/aarch64/` module tree | **Scaffold** (M13 prep) |
| PL011 serial stub | **Scaffold** — not wired to an entry point |
| `aarch64-unknown-none` kernel binary | **Not built** |
| QEMU boot smoke test | **Not implemented** |
| UEFI firmware (AA64) | **Planned** |

**Aether OS does not boot on AArch64 yet.**

## Planned target

| Component | Value |
|-----------|-------|
| Emulator | `qemu-system-aarch64` |
| Machine | `virt` |
| CPU | `max` or `cortex-a72` (TBD at M13) |
| Firmware | UEFI AA64 (e.g. `QEMU_EFI.fd` / AAVMF) |
| Early console | PL011 UART @ `0x0900_0000` (QEMU `virt` UART0) |
| Rust target | `aarch64-unknown-none` |

## Cross-compilation target (future)

When the M13 boot path lands, developers will install:

```bash
rustup target add aarch64-unknown-none
```

Bare-metal builds will require `-Z build-std` (same pattern as the x86_64 kernel).
There is **no** Makefile target or CI job for AArch64 at this milestone prep stage.

## Illustrative QEMU invocation (not verified)

The following is a **design sketch** for a future smoke test. Do not expect it to
boot Aether OS today.

```bash
# Requires: qemu-system-aarch64, AA64 UEFI firmware, disk image with ESP (future)
qemu-system-aarch64 \
  -machine virt \
  -cpu max \
  -m 512M \
  -drive if=pflash,format=raw,readonly=on,file=QEMU_EFI.fd \
  -drive if=pflash,format=raw,file=AAVMF_VARS.fd \
  -drive file=aether-aarch64.img,format=raw \
  -serial stdio
```

## Kernel scaffold layout

```
kernel/src/arch/aarch64/
├── mod.rs      # module root
├── boot.rs     # early_init / WFI idle (not linked)
└── serial.rs   # PL011 MMIO stub
```

## Related documents

- [Hardware compatibility matrix](README.md)
- [BUILD.md](../BUILD.md) — toolchain targets
- [ADR-0003](../adr/ADR-0003-initial-target-hardware.md) — initial x86_64 focus; ARM64 deferred until M13+
