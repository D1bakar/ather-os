# Changelog

All notable changes to Aether OS are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **In-browser UEFI boot (Phase 2 / ADR-0010):** Live demo at https://d1bakar.github.io/ather-os/ — **BOOT AETHER** runs real `BOOTX64.EFI` + `kernel.elf` via qemu.wasm (CDN) with OVMF bundled by CI; COM1 serial bridged to the page; SHA-256 verified artifact download with progress bar; `coi-serviceworker` for COOP/COEP on GitHub Pages.
- **Web boot modules:** `web/vm/qemu-emulator.js`, `artifact-loader.js`; redesigned mobile-friendly try page; reference serial log captured in CI (`artifacts/reference-serial.log`).

### Added (prior)

- **Web localhost + GitHub Pages:** `npm run serve` now serves `web/public/` (not repo root); VM worker copied to `public/vm/` during artifact build; `assetUrl()` helper resolves paths for local serve and `/ather-os/` Pages subpath; one-command launchers `web/serve.ps1` and `web/serve.sh`; `.github/workflows/pages.yml` deploys `web/public/` on push to `main`.
- **Documentation redesign (long-form):** README rewritten from scratch — research-lab narrative, ASCII banner, milestone progress bar, boot timeline, multiple mermaid diagrams, collapsible `<details>` sections, honest shipped vs blocked matrix; live demo link https://d1bakar.github.io/ather-os/.
- **`web/public/about.html`:** Updated voice — live demo link, Phase 1/2 honesty, refined hero ASCII.

### Fixed

- **Ring-3 init boot (M6.1 completion):** User tasks keep the kernel CR3 while running the ring-3 trampoline; syscalls switch to the kernel page table for handler execution and restore the user CR3 on `SYSRET`; user stack top moved to `0x7ffc0000` (away from GOP framebuffer at `0x80000000`); spawn path logs progress on serial; audit ring-buffer unit test no longer races on the global log.
- **CI bare-metal build:** Restore `kernel/linker.ld` rustflags when `RUSTFLAGS=-Dwarnings` overrides `.cargo/config.toml`.
- **Boot loader PT_LOAD allocation:** Page-align ELF segment addresses before `AllocatePages` and skip re-allocation when a segment lies inside pages already mapped for an earlier PT_LOAD (fixes `failed to allocate pages for ELF PT_LOAD segment` on M6 kernels with embedded init ELF and unaligned `.data` at `0x107100`).
- **Web serve broken on Windows:** `serve` package pointed at `web/` instead of `web/public/`; worker used absolute `/vm/worker.js` (404 locally and on GitHub Pages subpath).
- **M6.1 QEMU-verified ring-3 boot:** `build-boot.sh` builds and embeds user init ELF before kernel compile; CI/QEMU scripts invoke shell helpers via `bash`; serial smoke test requires `Aether OS M6: userland started` and ring-3 `Aether init started`; `tests/qemu_boot.rs` asserts M6 init strings.
- **M6 user space:** VFS trait layer (`kernel/src/vfs/`), ramfs mounted at boot (`kernel/src/fs/ramfs.rs`, `mount.rs`), per-process fd table wired to syscalls.
- Per-process page tables with kernel higher-half sharing; ELF64 loader maps user segments at `0x400000`; first ring-3 entry via `IRETQ` (`kernel/src/arch/x86_64/user_entry.rs`).
- Embedded init ELF from `build/user/init.elf`; `scripts/build-user.ps1` cross-compiles for `x86_64-unknown-none`.
- Syscall handlers: `open`, `read`, `close` (plus existing `write`, `exit`, `yield`, `getpid` from M5).
- User runtime wrappers in `libs/aether-rt` (`open`, `read`, `close`, `yield_cpu`).
- Host tests for ramfs fd table, syscall open path, and integration open/read pipeline.
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

- CI QEMU job: `run-qemu.sh` no longer executes `build-boot.sh` directly (avoids permission denied when scripts lack `+x` in git).
- Kernel bare-metal link: remove duplicate `-Tkernel/linker.ld` from `build.rs` (`.cargo/config.toml` already supplies it); discard `.comment` in linker script to avoid section overlap with embedded init ELF.
- CI bare-metal build: include `alloc` in `-Z build-std` to match `build-boot` scripts (fixes duplicate `core` lang item in `aether-collections`).
- CI / Build workflow: exclude `aether-boot` from host workspace builds (fixes duplicate `panic_impl` on Linux host).
- Integration tests: split `support/rng` from sync-only tests so `-D warnings` clippy gates pass for `security_m5` and `sched_syscall`.
- Clippy: remove redundant `#[must_use]` on `Result` returns in `aether-updater`; fix `aether-img-builder` lints.
- Kernel bare-metal: use `addr_of_mut!` for static task/registry access (Rust 2024 `static_mut_refs` under `-D warnings`).
- Kernel linker: emit `linker.ld` from `build.rs` for reliable bare-metal bin links on Linux CI.
- Fix FAT32 LFN directory entry slice bounds in `aether-img-builder`.
- Boot loader: satisfy clippy `question_mark` in ELF section parser.

### Changed

- [docs/ROADMAP.md](docs/ROADMAP.md) — synced to M0–M6.1 shipped status (was stuck at M2).
- QEMU smoke scripts (`run-qemu.ps1`, `run-qemu.sh`) wait for and require ring-3 init serial output.
- Kernel boot sequence: `mm::init` → GDT/IDT/PIC/timer → `sched::init` → worker thread → `syscall::init` → mount ramfs + spawn init → `sched::start`.
- QEMU smoke test expects optional `Aether OS M6: userland started` and ring-3 `Aether init started` when user ELF is embedded.
- README milestone table marks M6 as shipped; badge updated to M6.
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
- **QEMU serial boot works** when `qemu-system-x86_64` and OVMF are installed; ring-3 init message verified in CI optional job and local smoke scripts.
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
