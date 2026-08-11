# ADR-0003: Initial Target Hardware

**Status:** Accepted  
**Date:** 2026-08-11  
**Milestone:** M0

## Context

An OS project must choose an initial hardware platform to constrain architecture,
testing, and documentation. Supporting many architectures early dilutes effort and
delays a bootable milestone.

## Decision

**Tier 1 (primary development target):**

- **Architecture:** x86_64 (64-bit only; no 32-bit compatibility mode)
- **Environment:** QEMU `qemu-system-x86_64`
- **Firmware:** OVMF (UEFI)
- **Machine types:** `q35` (preferred), `i440fx` (secondary CI coverage)

**Tier 2 (experimental, not verified):**

- Real PC systems with UEFI firmware and x86_64 CPUs

**Not in initial scope:**

- Legacy BIOS boot
- ARM64, RISC-V, or other ISAs
- Embedded boards without UEFI

## Consequences

### Positive

- Abundant documentation (Intel SDM, OSDev wiki) and mature QEMU/GDB tooling.
- Matches contributor hardware and cloud CI runners.
- UEFI eliminates real-mode startup complexity.

### Negative

- x86_64 carries legacy interrupt and paging complexity (APIC, canonical addresses).
- Real hardware diversity (chipsets, firmware bugs) deferred until after QEMU boot works.
- Non-x86 contributors cannot run on native hardware initially.

### Follow-ups

- Maintain [docs/hardware/README.md](../hardware/README.md) as targets are verified.
- Add ARM64 only after M1–M3 milestones are stable on QEMU x86_64.
