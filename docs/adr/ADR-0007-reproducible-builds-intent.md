# ADR-0007: Reproducible Builds Intent

**Status:** Accepted (design intent — **partially implemented**)  
**Date:** 2026-08-11  
**Milestone:** M0

## Context

Operating system artifacts (boot loader, kernel, release images) must be auditable.
Reproducible builds allow third parties to verify that published binaries match
public source code, reducing supply-chain risk.

## Decision

Pursue **reproducible builds** as a project requirement with incremental adoption:

| Practice | M0 status | Target |
|----------|-----------|--------|
| Pin Rust toolchain (`rust-toolchain.toml`) | Implemented | Maintain per release |
| Commit `Cargo.lock` for workspace | Implemented | Required |
| Deterministic release profile (`lto`, `codegen-units = 1`) | Implemented in workspace | Kernel/boot use same profile |
| Document build environment (OS, QEMU, OVMF versions) | Partial | Full matrix at first release |
| `SOURCE_DATE_EPOCH` for timestamps | Planned | Release scripts |
| Build provenance attestation (SLSA-style) | Planned | Post-M1 |
| Bit-identical kernel ELF across machines | Planned | M1+ verification |

M0 does **not** claim bit-identical reproducibility for any boot artifact — no boot
binaries exist yet.

## Consequences

### Positive

- Establishes supply-chain hygiene from the first milestone.
- Simplifies debugging when CI and local builds match.
- Supports future signed release verification.

### Negative

- Reproducibility across OS hosts (Linux vs Windows) may require containerized builds.
- Some Rust/build metadata (paths, timestamps) requires explicit stripping or flags.
- Additional CI jobs needed to verify reproducibility once kernels are built.

### Follow-ups

- Add reproducibility check job when `kernel.elf` exists (M1).
- Document container image for canonical release builds.
- Integrate checksum publication in GitHub Releases workflow.
