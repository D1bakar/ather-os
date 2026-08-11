# ADR-0005: Syscall ABI Strategy

**Status:** Accepted (design intent — **dispatch not implemented**)  
**Date:** 2026-08-11  
**Milestone:** M0

## Context

User space and the kernel require a stable, explicit boundary. Ad hoc syscall
numbers or duplicated definitions across libc and kernel lead to subtle ABI drift
and security bugs.

## Decision

Define a **small, explicit syscall ABI** in the shared crate `aether-abi`:

| Aspect | Choice |
|--------|--------|
| Number registry | `SyscallNumber` enum in `aether-abi` — single source of truth |
| Calling convention (x86_64) | Number in `rax`; args in `rdi`, `rsi`, `rdx`, `r10`, `r8`, `r9` |
| Return value | `rax` — success as non-negative value or pointer; errors as negative `ErrorCode` |
| Unknown syscalls | Return defined error; kernel must not panic |
| ABI stability | Breaking changes require ADR + version bump |

M0 ships syscall **numbers, argument layout types, and tests** on the host.
Kernel syscall dispatch, user pointer validation, and capability checks are **planned M4**.

Initial syscall set (subject to extension via ADR):

- Process: `Exit`, `Yield` (stub for M1 cooperative scheduling)
- I/O: `Write`, `Read` (planned)
- Memory: `Map`, `Unmap` (planned)

## Consequences

### Positive

- libc, kernel, and tooling share one crate — no manual sync.
- `#![forbid(unsafe_code)]` in `aether-abi` keeps ABI definitions safe on host.
- Small surface area simplifies security review.

### Negative

- Every new syscall requires coordinated changes and documentation.
- x86_64-specific register layout needs separate ADR if other ISAs are added later.

### Follow-ups

- Implement `syscall` entry stub in kernel (M4).
- Add ABI compatibility tests between user libc and kernel.
