# Contributing to Aether OS

Thank you for your interest in contributing! This document covers the workflow,
standards, and expectations for all contributors.

## Getting Started

1. Fork the repository and clone your fork.
2. Install Rust via [rustup](https://rustup.rs/) — the pinned toolchain in
   `rust-toolchain.toml` will be used automatically.
3. Run the quality gate before submitting changes:

   ```bash
   make test
   ```

## Development Workflow

1. Create a feature branch from `main`.
2. Make focused, incremental changes.
3. Write or update tests for new behavior.
4. Ensure all checks pass locally.
5. Open a pull request using the provided template.

## Commit Messages

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]
```

Types: `feat`, `fix`, `docs`, `chore`, `ci`, `refactor`, `test`, `build`.

Examples:

- `feat(kernel): add physical frame allocator`
- `docs: update ADR for scheduler design`
- `ci: add cross-target build matrix`

## Code Standards

- Run `cargo fmt` before committing.
- All Clippy warnings must be resolved (`cargo clippy -- -D warnings`).
- Public API items require doc comments (`#![deny(missing_docs)]` in library crates).
- Prefer `#![forbid(unsafe_code)]` in shared crates; kernel `unsafe` must document invariants.
- Keep PRs small and reviewable — one logical change per PR.

## Architecture Decisions

Significant design choices must be documented as Architecture Decision Records
(ADRs) in `docs/adr/`. Use the numbered format: `ADR-NNNN-short-title.md`.
See [docs/adr/README.md](docs/adr/README.md) for the index and template.

## Testing

- Host tests run with `cargo test --workspace`.
- Kernel/integration tests will be added in M1+ using QEMU and custom harnesses.
- Do not merge changes that break existing tests.

## Code Review

- All PRs require at least one approving review.
- Address review feedback with additional commits (squash at merge time).
- Be respectful and constructive in discussions.

## Questions?

Open a [Discussion](https://github.com/aether-os/aether-os/discussions) or file an issue.
