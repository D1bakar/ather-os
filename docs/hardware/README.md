# Hardware Compatibility

This matrix tracks **verified** and **planned** hardware targets for Aether OS.
Entries marked *experimental* or *planned* are not compatibility guarantees.

**Current milestone:** M2 — QEMU UEFI boot with GDT/IDT/PIC/PIT verified in CI (optional job).

## Legend

| Status | Meaning |
|--------|---------|
| **Verified** | Boot and documented behavior confirmed by project CI or maintainer |
| **Supported (dev)** | Primary development target; expected to work with documented steps |
| **Experimental** | May work; no project testing; bugs expected |
| **Planned** | Design target only; no verification |
| **Not supported** | Out of scope for current architecture |

## Tier definitions

| Tier | Description |
|------|-------------|
| **Tier 1** | Development targets — CI, documentation examples, daily developer workflow |
| **Tier 2** | Experimental real hardware — may work; not a project priority until Tier 1 is stable |
| **Tier 3** | Future / out of scope — deferred or unsupported architectures |

---

## Primary compatibility matrix

### Virtual machines (Tier 1)

| Platform | CPU | Firmware | Boot | Serial | Interrupts | Storage | Status | Last verified | Notes |
|----------|-----|----------|------|--------|------------|---------|--------|---------------|-------|
| QEMU `q35` | x86_64 | OVMF (UEFI) | **Verified** | **Verified** | **Verified** (PIC/PIT) | FAT ESP (build script) | **Verified** | 2026-08 | Primary CI/dev target; `-machine q35` |
| QEMU `pc-i440fx` | x86_64 | OVMF (UEFI) | Supported (dev) | Supported (dev) | Supported (dev) | FAT ESP | Supported (dev) | — | Legacy machine type; not in CI |
| VMware Workstation | x86_64 | UEFI | Planned | Planned | Planned | — | Planned | — | Post Tier 1 stabilization |
| Hyper-V Gen 2 | x86_64 | UEFI | Planned | Planned | Planned | — | Planned | — | Serial over COM port TBD |
| VirtualBox | x86_64 | UEFI | Planned | Planned | Planned | — | Planned | — | OVMF support varies by host |

### Physical hardware (Tier 2)

| Platform | CPU | Firmware | Boot | Serial | Interrupts | Status | Last verified | Notes |
|----------|-----|----------|------|--------|------------|--------|---------------|-------|
| Generic UEFI PC | x86_64 | UEFI | Experimental | Experimental | Experimental | **Experimental** | — | **Untested** — design target per ADR-0003 |
| Laptop (UEFI) | x86_64 | UEFI | Experimental | Experimental | Experimental | **Experimental** | — | Serial often unavailable; USB-serial adapter recommended |
| Apple hardware (Boot Camp / Hackintosh) | x86_64 | UEFI | Not supported | — | — | **Not supported** | — | Out of scope |
| Legacy BIOS PC | x86_64 | BIOS | — | — | — | **Not supported** | — | UEFI-only boot path ([ADR-0006](../adr/ADR-0006-boot-architecture.md)) |

### Architectures (Tier 3 — deferred / scaffold)

| Platform | Status | Notes |
|----------|--------|-------|
| ARM64 (AArch64) | **Planned** (M13 scaffold) | `kernel/src/arch/aarch64/` stub only — **not bootable**; see [qemu-aarch64.md](qemu-aarch64.md) |
| RISC-V | **Not supported** | Deferred |
| 32-bit x86 (i686) | **Not supported** | 64-bit long mode required |

---

## Feature support by platform

| Feature | QEMU q35 + OVMF | Generic UEFI PC |
|---------|-----------------|-----------------|
| UEFI boot loader | **Verified** | Experimental |
| Kernel serial (COM1) | **Verified** | Experimental (if COM1 exposed) |
| GDT / IDT init | **Verified** | Experimental |
| 8259 PIC + PIT timer | **Verified** | Experimental (hardware PIC may differ) |
| APIC timer | Planned M4 | Planned M4 |
| Paging / heap | Planned M3 | Planned M3 |
| VirtIO block | Planned M3+ | N/A (QEMU) |
| VirtIO net | Planned M7 | N/A |
| Secure Boot | **Not supported** | Planned M10 |
| USB boot media | Planned M10 | Planned M10 |

---

## QEMU x86_64 (Tier 1 — verified)

### Minimum requirements

| Resource | Minimum |
|----------|---------|
| RAM | 512 MiB |
| Firmware | UEFI x86_64 (OVMF) |
| Display | Not required (serial-only diagnostics) |
| CPU | Any x86_64 with long mode (`qemu64` default) |

### Verified invocation

The project scripts wrap QEMU configuration. Manual equivalent:

```bash
# After: make boot
bash scripts/run-qemu.sh
# Serial log: build/qemu-serial.log
```

Illustrative manual command (paths vary by host):

```bash
qemu-system-x86_64 \
  -machine q35 \
  -cpu qemu64 \
  -m 512M \
  -drive if=pflash,format=raw,readonly=on,file=OVMF_CODE.fd \
  -drive if=pflash,format=raw,file=OVMF_VARS.fd \
  -drive file=build/aether.img,format=raw \
  -serial file:build/qemu-serial.log \
  -display none
```

### Expected M2 serial output

```
Aether OS kernel started
BootInfo OK
Aether OS M2: GDT/IDT/interrupts initialized
[timer] tick 100
[timer] tick 200
```

---

## Firmware requirements

| Requirement | M2 status | Notes |
|-------------|-----------|-------|
| UEFI boot services | **Required** | Boot loader exits boot services before kernel entry |
| GOP / framebuffer | Not required | Serial-only bring-up |
| ACPI tables | Planned M3+ | RSDP in `BootInfo` for APIC migration (M4) |
| Secure Boot | Not supported | Signature verification planned M10 |
| CSM (Legacy BIOS) | Not supported | UEFI-only |

---

## Serial console

| Parameter | Value |
|-----------|-------|
| Port | COM1 (I/O `0x3F8`) |
| Baud | 115200 |
| Data bits | 8 |
| Parity | None |
| Stop bits | 1 |
| Flow control | None |

On real hardware, connect a USB-to-serial adapter to the COM1 header or use a serial-capable
BMC if available. Many laptops do not expose COM1 — QEMU remains the primary diagnostic path.

---

## Storage layout (boot artifacts)

| Path on ESP | File | Status |
|-------------|------|--------|
| `EFI/BOOT/BOOTX64.EFI` | UEFI boot loader | **Shipped** |
| `aether/kernel.elf` | Bare-metal kernel | **Shipped** |

Future layout (planned):

| Path | Purpose | Milestone |
|------|---------|-----------|
| `aether/config/` | Boot configuration | M10 |
| `aether/updates/` | Update bundles | M10 |

---

## Adding a verified entry

1. Boot using documented steps ([INSTALL.md](../INSTALL.md), [BUILD.md](../BUILD.md)).
2. Capture serial log showing M2 banner and timer ticks.
3. Open a PR updating this table with **Verified** status, date, and environment details:
   - QEMU version / hardware model
   - OVMF or firmware version
   - Host OS used for the test

---

## Related documents

- [ARCHITECTURE.md](../../ARCHITECTURE.md) — supported hardware summary
- [ADR-0003](../adr/ADR-0003-initial-target-hardware.md) — initial target hardware decision
- [ADR-0006](../adr/ADR-0006-boot-architecture.md) — boot architecture
- [qemu-aarch64.md](qemu-aarch64.md) — AArch64 QEMU scaffold (M13; not bootable)
- [INSTALL.md](../INSTALL.md) — developer setup
- [DEPLOYMENT.md](../DEPLOYMENT.md) — deployment (future real hardware)
