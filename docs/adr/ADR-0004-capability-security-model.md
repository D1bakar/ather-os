# ADR-0004: Capability-Oriented Security Model

**Status:** Accepted (design intent — **not implemented**)  
**Date:** 2026-08-11  
**Milestone:** M0

## Context

Traditional Unix systems combine identity (UID), groups, and DAC permissions with
ambient authority: a process that can open a directory may access many resources
implicitly. Aether OS aims to be **security-first**, requiring an access model that
defaults to least privilege and supports auditable delegation.

## Decision

Adopt a **capability-oriented security model** as the long-term access control
foundation:

1. **Capabilities** are unforgeable tokens referencing kernel objects (files,
   devices, memory mappings, synchronization primitives).
2. Each process holds a **capability table**; there is no implicit "root" for
   ordinary programs.
3. **Delegation** explicitly copies or attenuates rights (e.g., read-only subset).
4. **Syscalls** validate that the caller holds required capabilities before
   performing operations — fail closed on missing or invalid capabilities.

M0 ships **policy documentation and type scaffolding only**. No capability table,
broker, or enforcement exists in the kernel.

## Consequences

### Positive

- Least privilege by construction rather than convention.
- Clear audit trail for delegation chains (planned).
- Reduces confused-deputy risk when syscalls mediate access.

### Negative

- Higher implementation complexity than Unix DAC for early milestones.
- Requires careful ABI design so capabilities are not forgeable from user space.
- Application ecosystems must adapt to capability passing (planned libc support).

### Follow-ups

- Specify capability wire format and syscall semantics in M4 ADR.
- Prototype capability table layout in kernel before user-space init (M6).
