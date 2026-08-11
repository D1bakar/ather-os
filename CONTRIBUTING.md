# Contributing to Aether OS

Thank you for your interest in contributing! This document covers environment setup,
the pre-commit checklist, commit conventions, and review expectations.

## Getting Started

1. Fork the repository and clone your fork.
2. Install [rustup](https://rustup.rs/) — the pinned toolchain in
   `rust-toolchain.toml` is selected automatically when you run `cargo` or `rustup`.
3. Run the developer setup script (installs components, cross targets, and prints QEMU hints):

   **Unix / macOS / WSL / Dev Container**

   ```bash
   make setup
   # or: bash scripts/setup-dev.sh
   ```

   **Windows (PowerShell)**

   ```powershell
   make setup
   # or: .\scripts\setup-dev.ps1
   ```

4. Optional — boot in QEMU (requires `qemu-system-x86_64` and OVMF firmware):

   ```bash
   make run
   ```

### Prerequisites

| Tool | Purpose |
|------|---------|
| Rust 1.85.0 (via `rust-toolchain.toml`) | Build and test |
| `rust-src`, `rustfmt`, `clippy`, `llvm-tools-preview` | Kernel `build-std`, formatting, lint, coverage tools |
| `x86_64-unknown-uefi`, `x86_64-unknown-none` targets | Boot loader and bare-metal kernel |
| QEMU + OVMF | `make run` smoke test |
| GNU Make (optional on Windows) | `make` targets; scripts work standalone |

### Dev Container

Open the repository in a [Dev Container](https://containers.dev/) (VS Code / Cursor).
The image installs Rust, QEMU, OVMF, and runs `scripts/setup-dev.sh` on create.

### Editor setup

Recommended extensions are listed in `.vscode/extensions.json` (`rust-analyzer`, etc.).
`.vscode/settings.json` enables format-on-save for Rust.

## Development Workflow

1. Create a feature branch from `main`.
2. Make focused, incremental changes.
3. Write or update tests for new behavior.
4. Run the pre-commit checklist (below).
5. Open a pull request using the provided template.

## Pre-commit Checklist

Run these before every commit or PR:

```bash
make test          # fmt + clippy + workspace tests
make boot          # cross-target boot artifacts (when touching boot/kernel)
make run           # QEMU serial smoke test (when boot path changes; needs QEMU/OVMF)
make m2-check      # full gate: fmt --check, clippy, tests, boot build
```

Individual targets:

| Target | Command |
|--------|---------|
| Format | `make fmt` |
| Lint | `make clippy` |
| Host build | `make build` |
| Boot ESP | `make boot` |
| Clean | `make clean` |

Host-only quick check (no cross-target boot):

```bash
cargo fmt --all -- --check
cargo clippy --workspace --exclude aether-boot --all-targets -- -D warnings
cargo clippy -p aether-boot --target x86_64-unknown-uefi -- -D warnings
cargo test --workspace
```

Integration test (QEMU, ignored in default CI quality job):

```bash
cargo test -p aether-integration-tests -- --ignored
```

## Commit Messages

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]
```

**Types:** `feat`, `fix`, `docs`, `chore`, `ci`, `refactor`, `test`, `build`.

**Examples:**

- `feat(kernel): add physical frame allocator`
- `fix(boot): validate kernel.elf size before load`
- `docs: update ADR for scheduler design`
- `build: improve developer tooling and cross-platform scripts`
- `ci: add cross-target build matrix`

Keep the subject line imperative and under 72 characters. Reference issues in the body when relevant (`Fixes #123`).

## Code Standards

- Run `cargo fmt` before committing (`make fmt`).
- All Clippy warnings must be resolved (`make clippy`).
- Public API items require doc comments (`#![deny(missing_docs)]` in library crates).
- Prefer `#![forbid(unsafe_code)]` in shared crates; kernel `unsafe` must document invariants.
- Keep PRs small and reviewable — one logical change per PR.
- Do not edit kernel `arch/` code unless the PR scope explicitly requires it.

## License headers

New source files must include the MIT license header at the top of the file:

```rust
// Copyright (c) 2026 Aether OS Contributors
// SPDX-License-Identifier: MIT
```

For non-Rust files, use the equivalent comment syntax for the language (for example `#` in shell
scripts, `//` in C/Rust, `#` in TOML is not applicable — use a comment block in README-only dirs).

The full license text is in [LICENSE](LICENSE). By contributing, you agree that your
contributions are licensed under the same MIT license unless otherwise agreed in writing
(see [GOVERNANCE.md](GOVERNANCE.md)).

## Architecture Decisions

Significant design choices must be documented as Architecture Decision Records
(ADRs) in `docs/adr/`. Use the numbered format: `ADR-NNNN-short-title.md`.
See [docs/adr/README.md](docs/adr/README.md) for the index and template.

## Testing

- Host tests: `cargo test --workspace` or `make test`.
- Boot artifacts: `make boot`.
- QEMU smoke: `make run` (serial log must contain `Aether OS kernel started`).
- Do not merge changes that break existing tests.

## Code Review

- All PRs require at least one approving review.
- Address review feedback with additional commits (squash at merge time).
- Be respectful and constructive in discussions.

## Questions?

Open a [Discussion](https://github.com/aether-os/aether-os/discussions) or file an issue.
