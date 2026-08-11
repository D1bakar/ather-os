# Rust Code Style and Kernel Conventions

This document defines coding standards for Aether OS. It supplements [CONTRIBUTING.md](../../CONTRIBUTING.md)
with kernel-specific rules. Host tooling and shared crates follow the same baseline unless noted.

## Toolchain and formatting

- Use the pinned toolchain in [rust-toolchain.toml](../../rust-toolchain.toml).
- Run `cargo fmt --all` before committing; CI enforces `cargo fmt --all -- --check`.
- Resolve all Clippy warnings: `cargo clippy --workspace --all-targets -- -D warnings`.
- For the UEFI boot loader: `cargo clippy -p aether-boot --target x86_64-unknown-uefi -- -D warnings`.

## Crate categories

| Category | Examples | `no_std` | `unsafe` policy |
|----------|----------|----------|-----------------|
| **Shared libraries** | `aether-types`, `aether-abi`, `aether-logger` | Optional (`std` for host tests) | `#![forbid(unsafe_code)]` |
| **Boot loader** | `aether-boot` | Yes (UEFI) | Minimal; document every `unsafe` block |
| **Kernel** | `aether-kernel` | Yes | Allowed with mandatory SAFETY comments |
| **Host / tests** | Integration tests | No | Follow normal Rust guidelines |

## Documentation requirements

- Library crates use `#![deny(missing_docs)]` on public items.
- Every public function, struct, and module needs a `///` doc comment.
- Kernel `unsafe` blocks require a `// SAFETY:` comment explaining invariants.
- Non-obvious design choices belong in ADRs ([docs/adr/](../adr/)), not long inline comments.

## Naming conventions

| Item | Convention | Example |
|------|------------|---------|
| Crates | `aether-*` prefix, kebab-case | `aether-types` |
| Modules | snake_case | `boot_info`, `frame_allocator` |
| Types | PascalCase | `PhysicalAddress`, `BootInfo` |
| Functions | snake_case | `map_page`, `init_serial` |
| Constants | SCREAMING_SNAKE_CASE | `BOOT_INFO_MAGIC`, `PAGE_SIZE` |
| MMIO / ports | Named constants, not raw literals | `COM1_PORT = 0x3F8` |

## Type and API design

- Prefer **newtypes** over raw integers for addresses, frames, and syscall numbers
  (see `aether-types`).
- Use `#[repr(C)]` for structures shared across boot loader, kernel, or user ABI boundaries.
- Version shared layouts with explicit `version` fields and validation helpers (`is_valid()`).
- Prefer `Result<T, Error>` over panics in code paths that may receive external input.
- Early boot and panic paths may halt or spin; document why recovery is impossible.

## `unsafe` guidelines (kernel and boot loader)

1. **Minimize scope** — keep `unsafe` blocks as small as possible.
2. **Document invariants** — every block starts with `// SAFETY: ...` explaining why it is sound.
3. **Centralize hardware access** — wrap port I/O, MMIO, and inline assembly in dedicated modules
   (e.g. `serial.rs`, future `arch/x86_64/`).
4. **No `unsafe` in shared crates** — use `#![forbid(unsafe_code)]` unless an ADR approves an exception.
5. **Avoid `unsafe` in public APIs** — expose safe wrappers; keep `unsafe fn` internal when possible.

Example:

```rust
// SAFETY: Boot loader guarantees BootInfo remains valid and aligned after handoff.
let info = unsafe { &*boot_info };
```

## Error handling

- Define error types in `aether-types` or subsystem modules; avoid stringly-typed errors in ABI.
- Syscall errors use stable codes from `aether-abi` (planned M4).
- Panic handler writes to serial and halts — do not rely on panics for recoverable errors in
  interrupt context (planned M2+).

## Architecture-specific code layout (planned)

Kernel subsystems will live under predictable paths:

```
kernel/src/
├── main.rs              # Entry, early init sequence
├── arch/x86_64/         # GDT, IDT, CPU, context switch
├── mm/                  # Physical frames, page tables, heap
├── sched/               # Tasks, scheduler
├── syscall/             # Dispatch and handlers
└── fs/                  # VFS (M5)
```

- **`arch/`** — all target-specific code; other modules call through thin arch APIs.
- **`mm/`** — no direct port I/O; request device access through driver boundaries (future).
- **Cross-subsystem types** — prefer `aether-types`; avoid circular module dependencies.

## Boot and ABI stability

- **`BootInfo`** — append-only field evolution; bump `BOOT_INFO_VERSION` on breaking layout changes.
- **Syscall numbers** — never reuse; deprecate via ADR ([ADR-0005](../adr/ADR-0005-syscall-abi-strategy.md)).
- **Calling convention** — x86_64 System V AMD64 for kernel entry; syscall ABI documented in `aether-abi`.

## Testing conventions

| Layer | Approach |
|-------|----------|
| Shared crates | Unit tests on host (`#[cfg(test)]`) |
| Kernel logic | Host-testable pure functions where feasible; `host-stub` feature for M0 patterns |
| Boot path | QEMU integration test (`tests/qemu_boot.rs`), marked `#[ignore]` for optional CI |
| `#![no_std]` code | Prefer compile-time and layout tests; avoid host-only assumptions in kernel modules |

Do not merge changes that break existing tests.

## Commits and pull requests

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>
```

Types: `feat`, `fix`, `docs`, `chore`, `ci`, `refactor`, `test`, `build`.

Examples:

- `feat(kernel): add physical frame allocator`
- `docs: add ADR-0008 interrupt architecture`
- `fix(boot): validate memory map pointer before handoff`

Keep pull requests focused — one logical change per PR.

## Status-aware documentation

When documenting or implementing a subsystem, state clearly whether behavior is
**shipped**, **partial**, or **planned**. Do not document unimplemented features as
if they exist at runtime.

## Related documents

- [CONTRIBUTING.md](../../CONTRIBUTING.md) — workflow and review
- [docs/development/getting-started.md](getting-started.md) — environment setup
- [ARCHITECTURE.md](../../ARCHITECTURE.md) — system design
- [docs/adr/ADR-0002](../adr/ADR-0002-rust-first-language-strategy.md) — Rust-first strategy
