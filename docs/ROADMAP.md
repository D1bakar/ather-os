# Aether OS Roadmap

Honest milestone tracking for the Aether OS kernel, boot chain, and platform services.
**Shipped** items are verified in CI or documented QEMU smoke tests unless noted otherwise.
**Planned** items describe design intent with no runtime implementation.

**Current milestone:** M6.1 (QEMU-verified ring-3 init boot).

## Milestone overview (M0–M10)

| Milestone | Theme | Status |
|-----------|-------|--------|
| M0 | Foundation | **Shipped** |
| M1 | Boot path | **Shipped** |
| M2 | CPU and interrupts | **Shipped** |
| M3 | Memory management | **Shipped** |
| M4 | Scheduler and preemption | **Shipped** |
| M5 | Syscalls and capabilities | **Shipped** |
| M6 | User space and VFS | **Shipped** |
| M6.1 | QEMU-verified ring-3 init | **Shipped** |
| M7 | Networking | Planned |
| M8 | Graphics and input | Planned |
| M9 | Application packaging | Planned |
| M10 | Atomic updates and production readiness | Planned |

---

## M0 — Foundation (shipped)

**Goal:** Establish engineering infrastructure before bare-metal code.

| Deliverable | Status |
|-------------|--------|
| Cargo workspace layout | Shipped |
| Shared crates: `aether-types`, `aether-abi`, `aether-logger` | Shipped |
| Architecture Decision Records (ADR-0001–0007) | Shipped |
| CI workflows, Makefile, build scripts | Shipped |
| Architecture, security, governance, contributing docs | Shipped |
| Host-buildable kernel stub for unit tests | Shipped |
| MIT license, CODE_OF_CONDUCT, SECURITY policy | Shipped |

**Version tag:** `v0.1.0`

---

## M1 — Boot path (shipped)

**Goal:** Boot a minimal kernel from UEFI under QEMU with serial diagnostics.

| Deliverable | Status |
|-------------|--------|
| UEFI boot loader (`aether-boot`) loading `kernel.elf` | Shipped |
| Bare-metal kernel entry (`_start`), panic handler | Shipped |
| COM1 serial console output | Shipped |
| `BootInfo` handoff with magic/version validation | Shipped |
| ESP build scripts (`build-boot.sh` / `.ps1`) | Shipped |
| QEMU + OVMF smoke test | Shipped (CI optional job) |

**Not in M1:**

- Full UEFI memory-map copy into stable storage (stubbed)
- Real PC hardware verification
- Paging or heap

---

## M2 — CPU and interrupts (shipped)

**Goal:** Bring up CPU privilege structures and validate IRQ delivery.

| Deliverable | Status |
|-------------|--------|
| Global Descriptor Table (GDT) — kernel code/data, TSS placeholder | Shipped |
| 256-entry IDT with exception handlers | Shipped |
| 8259 PIC remapping (IRQ 0–15 → vectors 32–47) | Shipped |
| PIT channel 0 at ~100 Hz; atomic tick counter | Shipped |
| Timer IRQ handler with periodic serial diagnostics | Shipped |
| Host tests: GDT descriptor math, IDT gate layout | Shipped |
| Local CI gate scripts (`ci-check.sh` / `.ps1`) | Shipped |
| ADR-0008 interrupt architecture | Shipped (Accepted) |

**Not in M2:**

- Local APIC / I/O APIC (follow-up)
- Scheduler or context switch (M4)

---

## M3 — Memory management (shipped)

**Goal:** Transition from firmware-owned memory to kernel-managed virtual memory.

| Deliverable | Status |
|-------------|--------|
| Physical frame allocator (bitmap) | Shipped |
| Four-level x86_64 page tables | Shipped |
| Kernel higher-half direct map | Shipped |
| Kernel heap allocator | Shipped |
| W^X enforcement for user mappings | Shipped (user segments) |

**Not in M3:**

- Copy UEFI memory map into stable kernel storage (stubbed)
- Full page-fault recovery policy

---

## M4 — Scheduler (shipped)

**Goal:** Multitasking with preemptive scheduling.

| Deliverable | Status |
|-------------|--------|
| Task Control Blocks, run queues | Shipped |
| Context switch (`arch/x86_64/switch.rs`) | Shipped |
| Cooperative yield and preemptive round-robin (PIT) | Shipped |
| Idle and worker kernel threads | Shipped |
| User code/data GDT segments (ring 3 scaffold) | Shipped |
| TSS + kernel stack for syscall entry | Shipped |

**Not in M4:**

- APIC timer (PIT path used)
- Full interrupt-frame save on context switch (documented limitation)

---

## M5 — Syscalls and capabilities (shipped)

**Goal:** User/kernel boundary with validated dispatch.

| Deliverable | Status |
|-------------|--------|
| Syscall entry (`SYSCALL`/`SYSRET` via IA32_STAR/LSTAR/FMASK) | Shipped |
| Dispatch table with fail-closed unknown syscall handling | Shipped |
| User pointer validation against caller address space | Shipped |
| Capability table scaffold and enforcement stubs | Shipped |
| Initial syscall set: exit, yield, write, getpid | Shipped |

**Dependencies:** M4 (user-mode tasks).

**ADR:** [ADR-0005](adr/ADR-0005-syscall-abi-strategy.md), [ADR-0004](adr/ADR-0004-capability-security-model.md).

---

## M6 — User space and VFS (shipped)

**Goal:** Minimal userspace environment with in-memory filesystem.

| Deliverable | Status |
|-------------|--------|
| VFS layer with pluggable backends | Shipped |
| ramfs (early root mount) | Shipped |
| User-space runtime (`aether-rt`) | Shipped |
| Init process (embedded ELF) | Shipped |
| Syscalls: open, read, close | Shipped |
| Per-process page tables and ELF64 loader | Shipped |
| First ring-3 entry via `IRETQ` | Shipped |

**Dependencies:** M5 (syscalls for file I/O and process spawn).

**Not in M6:**

- tmpfs, devfs
- Minimal shell in QEMU boot path
- Multi-process spawn

---

## M6.1 — QEMU-verified ring-3 init (shipped)

**Goal:** End-to-end proof that init runs in ring 3 under QEMU.

| Deliverable | Status |
|-------------|--------|
| `build-boot.sh` embeds user init ELF before kernel build | Shipped |
| CI/QEMU scripts invoke shell scripts via `bash` (no execute-bit dependency) | Shipped |
| Serial shows `Aether init started` from ring-3 `write` syscall | Shipped (CI optional job) |
| `tests/qemu_boot.rs` asserts M6 + init strings | Shipped |
| `run-qemu.ps1` / `run-qemu.sh` pass criteria include ring-3 init | Shipped |

---

## M7 — Networking (planned)

**Goal:** Basic network connectivity for development and testing.

| Deliverable | Status |
|-------------|--------|
| VirtIO net driver (QEMU first) | Planned |
| Ethernet frame TX/RX | Planned |
| IPv4, ARP, ICMP (ping) | Planned |
| TCP/UDP socket syscalls | Planned |
| Loopback and DHCP client (QEMU) | Planned |

**Dependencies:** M6 (user-space tools), M4 (interrupt-driven I/O).

**Not in initial M7 scope:** Wi-Fi, TLS, firewall.

---

## M8 — Graphics and input (planned)

**Goal:** Interactive development beyond serial console.

| Deliverable | Status |
|-------------|--------|
| Framebuffer or GOP-backed linear framebuffer | Planned |
| Basic terminal emulator on framebuffer | Planned |
| PS/2 or VirtIO keyboard input | Planned |
| Mouse input (optional) | Planned |

**Dependencies:** M6 (user-space terminal), M3 (framebuffer mapping).

---

## M9 — Application packaging (planned; host scaffold)

**Goal:** Distributable application format and package manager.

| Deliverable | Status |
|-------------|--------|
| Package manifest format (`.aetherpkg`) | Spec draft ([packages/README.md](packages/README.md)) |
| Host package manager crate (`aether-pkgmgr`) | **Skeleton** — `system/pkgmgr/`; host-testable only |
| Package signing and verification | **Skeleton** — stub verifier; real Ed25519 behind `verify` feature |
| Package manager CLI (`aether-pkg`) | Planned |
| Dependency resolution and install paths | **Skeleton** — install API; no kernel FS |
| Sandboxed install via capabilities | Planned |

**Specification:** [docs/packages/README.md](packages/README.md)

**Dependencies:** M5 (capabilities), M6 (filesystem).

---

## M10 — Atomic updates and production readiness (planned; host scaffold)

**Goal:** Safe, verifiable OS updates and real-hardware support.

| Deliverable | Status |
|-------------|--------|
| A/B partition update types | **Skeleton** — `system/updater/src/partition.rs` |
| Signed manifest verification stub | **Skeleton** — `system/updater/src/verify.rs` |
| Rollback API (in-memory) | **Skeleton** — `system/updater/src/rollback.rs` |
| Host manifest checker | **Skeleton** — `scripts/update-check.ps1` |
| Boot loader slot selection | Planned |
| Runtime apply daemon | Planned |
| Real PC hardware compatibility (Tier 2 verified) | Planned |
| Reproducible build verification | Planned (intent: [ADR-0007](adr/ADR-0007-reproducible-builds-intent.md)) |
| Release pipeline with checksums | Planned |

**Specification:** [docs/updates/README.md](updates/README.md) · [ADR-0009](adr/ADR-0009-atomic-update-architecture.md)

**Dependencies:** M1 (boot chain), M6 (init health signal), signing infrastructure.

---

## Cross-cutting concerns

| Area | Document |
|------|----------|
| Architecture | [ARCHITECTURE.md](../ARCHITECTURE.md) |
| Interrupts | [ADR-0008](adr/ADR-0008-interrupt-architecture.md) |
| Security | [SECURITY.md](../SECURITY.md), [threat-model.md](security/threat-model.md) |
| Build | [BUILD.md](BUILD.md) |
| Install | [INSTALL.md](INSTALL.md) |
| Deployment | [DEPLOYMENT.md](DEPLOYMENT.md) |
| Hardware | [hardware/README.md](hardware/README.md) |
| Packages | [packages/README.md](packages/README.md) |
| Updates | [updates/README.md](updates/README.md) |

## Versioning

| Phase | Policy |
|-------|--------|
| M0–M2 | `0.1.x` — early development; tags document milestones |
| M3–M6 | `0.2.x`–`0.5.x` — subsystem milestones |
| M7–M10 | `0.6.x`+ — platform services; `1.0.0` when update path and real-hardware tier are verified |

Releases follow [Semantic Versioning](https://semver.org/). Git tags `v*` trigger the draft
release workflow (`.github/workflows/release.yml`).

Changes are recorded in [CHANGELOG.md](../CHANGELOG.md).
