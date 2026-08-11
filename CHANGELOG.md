# Changelog

All notable changes to Aether OS are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **M1 boot path:** UEFI boot loader crate (`aether-boot`) for `x86_64-unknown-uefi`.
- Bare-metal kernel binary (`aether-kernel` / `kernel.elf`) with COM1 serial output.
- `BootInfo` handoff structure in `aether-types` (magic, version, memory-map pointer, serial info).
- ESP build scripts: `scripts/build-boot.ps1`, `scripts/build-boot.sh`.
- QEMU smoke scripts: `scripts/run-qemu.ps1`, `scripts/run-qemu.sh`.
- Integration test crate (`tests/qemu_boot.rs`) with ignored QEMU boot smoke test.
- CI jobs to compile boot loader and bare-metal kernel; optional QEMU job on Ubuntu.

### Changed

- Kernel crate now builds a `bare-metal` binary in addition to the M0 host stub library.
- `make run` launches the QEMU smoke test when scripts and dependencies are available.

### Notes

- **QEMU serial boot works** when `qemu-system-x86_64` and OVMF are installed (verified in CI; local run requires same).
- **Real PC hardware boot is untested.** Memory-map copy in `BootInfo` is a stub until M2.
- GDT, IDT, paging, and heap remain planned for M2/M3.

## [0.1.0] - 2026-08-11

### Added

- M0 engineering foundation (documentation, ADRs, workspace layout, CI).
- Shared crates: `aether-types`, `aether-abi`, `aether-logger`.
- Kernel stub crate (`aether-kernel`) with M1 bare-metal path documented.
- Architecture, security, governance, and contributing documentation.
- Hardware compatibility matrix template under `docs/hardware/`.
- Seven initial Architecture Decision Records in `docs/adr/`.
- Initial repository scaffolding: workspace, toolchain pinning, Makefile, build scripts.
- GitHub Actions CI (format, clippy, test, build).
- MIT license and security reporting policy.

[Unreleased]: https://github.com/D1bakar/ather-os/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/D1bakar/ather-os/releases/tag/v0.1.0
