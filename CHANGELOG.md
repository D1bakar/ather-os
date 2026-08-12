# Changelog

All notable changes to Aether OS are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **M5 security & syscalls:** `SYSCALL`/`SYSRET` entry via IA32_STAR/LSTAR/FMASK (`kernel/src/arch/x86_64/syscall.rs`); `int 0x80` documented as fallback only.
- Syscall dispatch table validation, userspace pointer checks, and capability enforcement stubs (`kernel/src/syscall/dispatch.rs`, `validate.rs`).
- Stub handlers: `write` (COM1 serial), `exit`, `yield` — wired to M4 scheduler; `getpid` returns current process id.
- Ring-3 user code/data GDT segments and per-process capability table stub (`kernel/src/cap/`).
- Host integration tests: dispatch table, pointer validation, capability audit (`tests/sched_syscall.rs`, `tests/security_m5.rs`, `tests/arch_user_gdt.rs`).
- **M4 scheduler:** round-robin kernel-thread scheduler with idle + worker threads, voluntary yield, and PIT timer preemption (`kernel/src/sched/`).
- Context switch assembly saving callee-saved GPRs, `RSP`, `RIP`, and `CR3` (`kernel/src/arch/x86_64/switch.rs`).
- Host test for round-robin run-queue topology (`sched::scheduler` tests).
- **M3 memory (local):** physical frame allocator, paging, and kernel heap (`kernel/src/mm/`).
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

### Fixed

- CI bare-metal build: include `alloc` in `-Z build-std` to match `build-boot` scripts (fixes duplicate `core` lang item in `aether-collections`).
- CI / Build workflow: exclude `aether-boot` from host workspace builds (fixes duplicate `panic_impl` on Linux host).
- Integration tests: split `support/rng` from sync-only tests so `-D warnings` clippy gates pass for `security_m5` and `sched_syscall`.
- Clippy: remove redundant `#[must_use]` on `Result` returns in `aether-updater`; fix `aether-img-builder` lints.
- Kernel bare-metal: use `addr_of_mut!` for static task/registry access (Rust 2024 `static_mut_refs` under `-D warnings`).
- Kernel linker: emit `linker.ld` from `build.rs` for reliable bare-metal bin links on Linux CI.
- Fix FAT32 LFN directory entry slice bounds in `aether-img-builder`.
- Boot loader: satisfy clippy `question_mark` in ELF section parser.

### Changed

- Kernel boot sequence: `mm::init` → GDT/IDT/PIC/timer → `sched::init` → worker thread → `syscall::init` → `sched::start` (STI + first context switch).
- Timer IRQ handler sends EOI before calling `sched::tick_preempt()`.
- QEMU smoke test expects `Aether OS M4: scheduler initialized` and optionally worker thread output.
- README milestone table marks M5 as shipped; badge updated to M5.
- `aether-abi` workspace dependency defaults to `default-features = false` for bare-metal builds.
- Capability and audit globals use `SpinMutex` instead of `thread_local` for `#![no_std]` bare-metal.
- [ARCHITECTURE.md](ARCHITECTURE.md) — subsystem map through M10; shipped vs planned markers updated for M2.
- [SECURITY.md](SECURITY.md) and [docs/security/threat-model.md](docs/security/threat-model.md) — M2 maturity, interrupt handling, update/packaging scaffolds.
- [docs/architecture/README.md](docs/architecture/README.md) — M2 shipped status for CPU/arch subsystem.
- QEMU smoke test expects M2 serial banner (`Aether OS M2: GDT/IDT/interrupts initialized`); timer tick lines are optional.
- Kernel crate now builds a `bare-metal` binary in addition to the M0 host stub library.
- `make run` launches the QEMU smoke test when scripts and dependencies are available.

### Notes

- Timer preemption uses the legacy PIT path; local APIC timer migration remains a follow-up.
- Context switches preserve callee-saved registers only; full interrupt-frame save is documented in `switch.rs`.
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
