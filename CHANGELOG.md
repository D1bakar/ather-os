# Changelog

All notable changes to Aether OS are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- M0 engineering foundation (documentation, ADRs, workspace layout, CI).
- Shared crates: `aether-types`, `aether-abi`, `aether-logger`.
- Kernel stub crate (`aether-kernel`) with M1 bare-metal path documented.
- Architecture, security, governance, and contributing documentation.
- Hardware compatibility matrix template under `docs/hardware/`.
- Seven initial Architecture Decision Records in `docs/adr/`.

### Notes

- **The OS does not boot.** Boot loader, kernel entry, and QEMU run target are planned for M1.
- Syscall ABI and capability security model are **design intent only** — not enforced in the kernel.

## [0.1.0] - 2026-08-11

### Added

- Initial repository scaffolding: workspace, toolchain pinning, Makefile, build scripts.
- GitHub Actions CI (format, clippy, test, build).
- MIT license and security reporting policy.

[Unreleased]: https://github.com/D1bakar/ather-os/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/D1bakar/ather-os/releases/tag/v0.1.0
