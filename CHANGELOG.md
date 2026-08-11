# Changelog

All notable changes to Aether OS are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Industry-ready documentation:** professional README, [docs/ROADMAP.md](docs/ROADMAP.md) (M0–M10),
  [docs/BUILD.md](docs/BUILD.md), [docs/INSTALL.md](docs/INSTALL.md), [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md).
- Expanded hardware compatibility matrix: [docs/hardware/README.md](docs/hardware/README.md).
- Application packaging specification: [docs/packages/README.md](docs/packages/README.md).
- Atomic update architecture index and sub-documents: [docs/updates/](docs/updates/).
- LICENSE header guidance in [CONTRIBUTING.md](CONTRIBUTING.md).
- **M12 atomic update skeleton:** `aether-updater` crate with A/B partition types, signed manifest verification stub, and rollback API (`system/updater/`).
- Update architecture docs: `docs/updates/` (A/B partitions, signed verification, rollback API).
- [ADR-0009](docs/adr/ADR-0009-atomic-update-architecture.md) — atomic update architecture decision.
- Host update validation script: `scripts/update-check.ps1`.
- **M2 CPU bring-up:** GDT, IDT, legacy PIC remap, and PIT timer interrupt wired into kernel boot (`arch/x86_64/`).
- Host integration tests for GDT descriptor math and IDT gate layout (`tests/arch_gdt.rs`, `tests/arch_idt.rs`).
- Local CI gate scripts: `scripts/ci-check.ps1`, `scripts/ci-check.sh` (fmt, clippy, tests, bare-metal builds).
- Testing strategy documentation in `tests/README.md`.
- **M1 boot path:** UEFI boot loader crate (`aether-boot`) for `x86_64-unknown-uefi`.
- Bare-metal kernel binary (`aether-kernel` / `kernel.elf`) with COM1 serial output.
- `BootInfo` handoff structure in `aether-types` (magic, version, memory-map pointer, serial info).
- ESP build scripts: `scripts/build-boot.ps1`, `scripts/build-boot.sh`.
- QEMU smoke scripts: `scripts/run-qemu.ps1`, `scripts/run-qemu.sh`.
- Integration test crate (`tests/qemu_boot.rs`) with ignored QEMU boot smoke test.
- CI jobs to compile boot loader and bare-metal kernel; optional QEMU job on Ubuntu.

### Changed

- [ARCHITECTURE.md](ARCHITECTURE.md) — subsystem map through M10; shipped vs planned markers updated for M2.
- [SECURITY.md](SECURITY.md) and [docs/security/threat-model.md](docs/security/threat-model.md) — M2 maturity, interrupt handling, update/packaging scaffolds.
- [docs/architecture/README.md](docs/architecture/README.md) — M2 shipped status for CPU/arch subsystem.
- QEMU smoke test expects M2 serial banner (`Aether OS M2: GDT/IDT/interrupts initialized`); timer tick lines are optional.
- Kernel crate now builds a `bare-metal` binary in addition to the M0 host stub library.
- `make run` launches the QEMU smoke test when scripts and dependencies are available.

### Notes

- **QEMU serial boot works** when `qemu-system-x86_64` and OVMF are installed (verified in CI; local run requires same).
- **M2 host tests** cover GDT/IDT layout encoding; IRQ/timer delivery is validated via QEMU smoke when run.
- **Real PC hardware boot is untested.** Memory-map copy in `BootInfo` remains a stub.
- Paging and heap remain planned for M3+.

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
