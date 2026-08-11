# ADR-0001: Modular Monolithic Kernel

**Status:** Accepted  
**Date:** 2026-08-11  
**Milestone:** M0

## Context

Aether OS needs a kernel architecture before implementing boot, memory management,
or scheduling. Microkernel, hybrid, and monolithic designs each trade isolation,
performance, and implementation complexity. The project is a small early-stage team
with a Rust codebase and a goal of shipping a bootable system in M1.

## Decision

Adopt a **modular monolithic kernel**:

- All core subsystems (scheduler, memory manager, VFS, syscall layer) run in a
  single privileged address space.
- Subsystems are organized as **explicit Rust modules/crates** with documented
  boundaries and minimal shared mutable state.
- Drivers may start in-kernel for M1–M5; selected drivers may move to user space
  later if isolation requirements justify the IPC cost.

This is **not** a microkernel. It is a monolith with strict internal modularity.

## Consequences

### Positive

- Direct function calls between subsystems — low latency for common paths.
- Shared types from `aether-types` without cross-address-space serialization.
- Simpler bring-up for M1 (no IPC or multiserver infrastructure required).
- Aligns with Rust's module system for incremental growth.

### Negative

- A defect in one subsystem can corrupt the entire kernel address space.
- Must rely on Rust safety, code review, and testing until optional driver isolation.
- Porting to hard isolation later requires explicit refactoring of driver boundaries.

### Follow-ups

- Document module boundaries in `ARCHITECTURE.md` as subsystems land.
- Revisit driver isolation in a future ADR if hardware or security requirements demand it.
