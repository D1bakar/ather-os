# Architecture Decision Records

This directory contains Architecture Decision Records (ADRs) for Aether OS.

## Format

Each ADR includes:

- **Status** — Proposed, Accepted, Deprecated, or Superseded
- **Context** — forces and constraints
- **Decision** — what we chose
- **Consequences** — trade-offs and follow-ups

## Index

| ADR | Title | Status |
|-----|-------|--------|
| [ADR-0001](ADR-0001-modular-monolithic-kernel.md) | Modular monolithic kernel | Accepted |
| [ADR-0002](ADR-0002-rust-first-language-strategy.md) | Rust-first language strategy | Accepted |
| [ADR-0003](ADR-0003-initial-target-hardware.md) | Initial target hardware | Accepted |
| [ADR-0004](ADR-0004-capability-security-model.md) | Capability-oriented security model | Accepted (design intent) |
| [ADR-0005](ADR-0005-syscall-abi-strategy.md) | Syscall ABI strategy | Accepted (design intent) |
| [ADR-0006](ADR-0006-boot-architecture.md) | Boot architecture | Accepted (design intent) |
| [ADR-0007](ADR-0007-reproducible-builds-intent.md) | Reproducible builds intent | Accepted (design intent) |
| [ADR-0008](ADR-0008-interrupt-architecture.md) | x86_64 interrupt and timer architecture | Accepted — M2 shipped (PIC/PIT) |
| [ADR-0009](ADR-0009-atomic-update-architecture.md) | Atomic A/B update architecture | Accepted — M12 skeleton |
| [ADR-0010](ADR-0010-browser-vm-architecture.md) | Browser VM architecture (Universal Platform) | Accepted — Phase 1 scaffold |

## Historical document

The consolidated [001-initial-decisions.md](../architecture/001-initial-decisions.md)
predates numbered ADRs and remains for reference. New decisions should be added
here as individual ADRs.

## Proposing a change

1. Copy the template structure from an existing ADR.
2. Open a PR with status `Proposed`.
3. After review, maintainers set status to `Accepted` or `Superseded`.
