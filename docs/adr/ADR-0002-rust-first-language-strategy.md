# ADR-0002: Rust-First Language Strategy

**Status:** Accepted  
**Date:** 2026-08-11  
**Milestone:** M0

## Context

The kernel, boot loader, and shared libraries need a systems language with strong
memory safety guarantees, bare-metal support, and mature tooling. C, C++, and Rust
are the primary candidates for a new OS project in 2026.

## Decision

Use **Rust** as the primary implementation language:

| Layer | Policy |
|-------|--------|
| Shared crates (`aether-types`, `aether-abi`, `aether-logger`) | `#![forbid(unsafe_code)]` |
| Boot loader and kernel | Rust with **targeted** `unsafe` where hardware requires it; invariants documented in code and reviews |
| Assembly | Minimal — only where Rust cannot express required instructions (e.g., early boot stubs) |
| Host tooling | Rust preferred; shell/PowerShell scripts for orchestration |

Toolchain is pinned in `rust-toolchain.toml` (currently **1.85.0**).

## Consequences

### Positive

- Eliminates large classes of memory errors at compile time in most code.
- First-class `no_std` and cross-compilation to `x86_64-unknown-none` and UEFI targets.
- `cargo`, `clippy`, and `rustfmt` integrate with CI from M0.

### Negative

- Steeper contributor onboarding for developers unfamiliar with Rust ownership.
- Bare-metal and UEFI ecosystems require careful dependency selection.
- Compile times for full kernel builds may be higher than equivalent C.

### Follow-ups

- Define kernel `unsafe` review checklist before M1 merge window.
- Evaluate `miri` and sanitizer usage for host-testable components.
