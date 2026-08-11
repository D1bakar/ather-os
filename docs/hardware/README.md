# Hardware Compatibility

This matrix tracks **verified** and **planned** hardware targets for Aether OS.
Entries marked *planned* or *experimental* are not compatibility guarantees.

## Legend

| Status | Meaning |
|--------|---------|
| **Verified** | Boot and basic serial output confirmed by project CI or maintainer |
| **Supported (dev)** | Primary development target; expected to work but not yet verified end-to-end |
| **Experimental** | May work; no testing; bugs expected |
| **Planned** | Design target only |
| **Not supported** | Out of scope |

## Compatibility matrix

| Platform | CPU | Firmware | Boot | Serial | Storage | Tier | Status | Last verified | Notes |
|----------|-----|----------|------|--------|---------|------|--------|---------------|-------|
| QEMU `pc-q35-8.0` | x86_64 | OVMF (UEFI) | Planned M1 | Planned M1 | virtio-blk (planned) | 1 | Supported (dev) | — | Primary CI and developer target |
| QEMU `pc-i440fx-8.0` | x86_64 | OVMF (UEFI) | Planned M1 | Planned M1 | virtio-blk (planned) | 1 | Supported (dev) | — | Legacy machine type for broader QEMU coverage |
| Generic PC | x86_64 | UEFI | — | — | — | 2 | Experimental | — | Real hardware untested until M1+ |
| Legacy BIOS PC | x86_64 | BIOS | — | — | — | — | Not supported | — | UEFI-only boot path (ADR-0006) |
| ARM64 / RISC-V | — | — | — | — | — | — | Not supported | — | Deferred; x86_64 first |

## Tier definitions

### Tier 1 — development targets

Platforms the project intends to support for daily development, documentation
examples, and CI boot tests once M1 lands.

### Tier 2 — experimental

Real hardware that may work but is not a project priority until Tier 1 QEMU boot
is stable. Issues may be closed as "needs hardware verification."

## QEMU x86_64 (Tier 1)

**Status:** design target — **Aether OS does not boot yet.**

Expected M1 invocation (illustrative, not verified):

```bash
# Requires: qemu-system-x86_64, OVMF_CODE.fd, OVMF_VARS.fd, disk image with ESP
qemu-system-x86_64 \
  -machine q35 \
  -cpu qemu64 \
  -m 512M \
  -drive if=pflash,format=raw,readonly=on,file=OVMF_CODE.fd \
  -drive if=pflash,format=raw,file=OVMF_VARS.fd \
  -drive file=aether.img,format=raw \
  -serial stdio
```

### Minimum requirements (planned)

| Resource | Minimum |
|----------|---------|
| RAM | 512 MiB |
| Firmware | UEFI x86_64 (OVMF) |
| Display | Not required for M1 (serial-only boot banner) |

## Adding a new entry

1. Confirm boot via documented steps (once boot exists).
2. Open a PR updating this table with **Verified** status and date.
3. Note firmware version, QEMU version, or hardware model in **Notes**.

## Related documents

- [ARCHITECTURE.md](../../ARCHITECTURE.md)
- [ADR-0003](../adr/ADR-0003-initial-target-hardware.md)
