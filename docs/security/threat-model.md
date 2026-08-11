# Aether OS Threat Model

This document describes the **security assumptions, assets, adversaries, and mitigations**
for Aether OS. It is a living design document; status markers distinguish **shipped**
behavior from **planned** intent.

For vulnerability reporting and disclosure policy, see [SECURITY.md](../../SECURITY.md).

## Scope and maturity

| Phase | Runtime state | Threat model applicability |
|-------|---------------|----------------------------|
| **M0** | Shared crates and CI only | Design assumptions only |
| **M1** | UEFI boot + serial kernel stub | Boot handoff validation partial; no user space |
| **M2** | GDT, IDT, PIC/PIT interrupts | Exception handling shipped; no paging isolation |
| **M3** | Paging, heap (planned) | Kernel memory integrity becomes critical |
| **M5+** | Syscalls, capabilities (planned) | User/kernel boundary enforcement in scope |
| **M10** | Signed updates (planned) | Boot chain verification and rollback in scope |

**Current milestone:** M2. There is **no user space**, **no syscall dispatch**, **no paging**,
and **no capability enforcement** at runtime. Statements about those controls describe
**design intent**.

## Assets

| Asset | Description | M2 status | Future intent |
|-------|-------------|-----------|---------------|
| **Kernel memory and control flow** | Code, stacks, page tables, interrupt state | Running (minimal); no user isolation | Protect from user-space and driver corruption |
| **Interrupt state** | IDT, PIC mask, timer handler | **Shipped** — configured at boot | APIC migration; IST stacks for fault recovery |
| **Syscall boundary** | Register ABI and argument validation | ABI types only (`aether-abi`) | Validate pointers and capabilities per call |
| **Boot chain integrity** | Boot loader → kernel handoff, update images | Boot path shipped; no signature verification | Verified handoff and signed updates (M10) |
| **`BootInfo` handoff** | Fixed-layout structure from boot loader | Validated for magic/version only | Full memory-map and pointer validation before use (M3) |
| **User data on disk** | Files, credentials, application state | Not implemented | Access control via capabilities; encryption TBD |
| **Build artifacts and release keys** | CI outputs, future signed releases | CI builds only | Reproducible builds ([ADR-0007](../adr/ADR-0007-reproducible-builds-intent.md)) and signed releases |
| **Maintainer signing keys** | Future update signing material | Not established | Hardware-backed or offline key storage |
| **Update metadata** | A/B slot state, signed manifests | Host skeleton only (`system/updater/`) | Persisted boot control block on ESP (M10) |

## Trust boundaries

```mermaid
graph TB
    subgraph untrusted["Untrusted (future)"]
        USER[User-space processes]
        DRV[User-space drivers - optional future]
    end

    subgraph semi["Semi-trusted"]
        BL[UEFI boot loader - project code]
        FW[UEFI firmware - third party]
    end

    subgraph trusted["Trusted (design goal)"]
        KERN[Aether kernel]
    end

    USER -->|syscall + capabilities| KERN
    DRV -->|IPC / capabilities| KERN
    FW -->|loads| BL
    BL -->|BootInfo handoff| KERN
```

| Boundary | M2 state | Planned controls |
|----------|----------|------------------|
| User → kernel | Not present | Syscall validation, W^X user mappings, capability checks (M5) |
| Boot loader → kernel | **Partial** — magic/version check | Reject invalid `BootInfo`; validate memory-map pointers before allocator use (M3) |
| Firmware → boot loader | Implicit trust in OVMF/PC firmware | Future: measured boot / Secure Boot integration (M10) |
| Host toolchain → artifacts | Trust in Rust, QEMU, CI | Reproducible builds, pinned toolchain, signed releases |
| Update client → boot chain | Host skeleton only | Ed25519 verification, A/B rollback (M10) |

## Adversaries

### In-scope adversaries (design assumptions)

1. **Malicious user-space process (M5+)**  
   Attempts privilege escalation via syscalls, confused deputies, or kernel memory
   corruption. Mitigations: capability checks, pointer validation, Rust safety, W^X.

2. **Malicious or compromised driver (post-M6)**  
   Attempts to bypass capability checks or access arbitrary physical memory.
   Mitigations: least privilege, optional user-space driver isolation (future ADR).

3. **Local attacker with physical access**  
   Can modify disk images, attempt unsigned boot, attach debuggers, or replace firmware.
   Partially mitigated by future signed boot and update verification (M10); full physical
   access remains out of scope for pure software guarantees.

4. **Supply-chain attacker**  
   Compromises dependencies, CI, or release signing infrastructure.
   Mitigations: pinned toolchain, dependency review, reproducible builds intent,
   protected signing keys.

5. **Malformed hardware interrupts (M2+)**  
   Spurious or misconfigured IRQ delivery. Mitigations: PIC EOI discipline, exception
   handlers that halt rather than continue in ambiguous state.

### Out of scope (initial milestones)

| Category | Rationale |
|----------|-----------|
| **Remote network exploitation** | No network stack until M7 |
| **Side-channel attacks** (Spectre/Meltdown class) | Acknowledged; not mitigated in early milestones |
| **DoS via resource exhaustion** in unprivileged user code | Unless it crashes or hangs the kernel |
| **Third-party tool vulnerabilities** (QEMU, OVMF, host Rust) | Report upstream |
| **Theoretical attacks on unimplemented subsystems** | No concrete code path without a PoC on shipped code |

## Security principles

| Principle | Description | M2 status |
|-----------|-------------|-----------|
| **Fail closed** | Invalid syscalls, capabilities, or boot handoff rejected | **Partial** — `BootInfo` magic/version only |
| **Least privilege** | Processes hold explicit capabilities, not ambient root | **Planned** M5 |
| **Memory safety** | `#![forbid(unsafe_code)]` in shared crates; documented kernel `unsafe` | **Partial** — shared crates enforced |
| **Explicit ABI** | Small, versioned syscall surface in `aether-abi` | **Partial** — scaffold only |
| **W^X** | No writable+executable user mappings | **Planned** M3 |
| **Kernel isolation from user** | User mode cannot map or execute kernel pages | **Planned** M3 |
| **Exception transparency** | Unexpected CPU exceptions logged and halted | **Shipped** M2 |
| **Signed updates** | Atomic, verified OS updates | **Planned** M10 (host skeleton) |
| **Reproducible builds** | Deterministic artifacts where feasible | **Intent** — [ADR-0007](../adr/ADR-0007-reproducible-builds-intent.md) |

## Subsystem-specific threats

### Boot chain

**Shipped (M1):** UEFI loads `BOOTX64.EFI`, boot loader exits boot services, jumps to
kernel with `BootInfo` in `RDI`.

**Threats:**

- Tampered `kernel.elf` on ESP → **unmitigated** until signature verification (M10).
- Malformed `BootInfo` → **partially mitigated** by magic/version check; full pointer
  validation planned before memory allocator consumes the map (M3).
- Boot-services use-after-free → mitigated by exiting boot services before kernel entry
  ([ADR-0006](../adr/ADR-0006-boot-architecture.md)).

### CPU and interrupts (M2 — shipped)

**Shipped:** GDT, IDT, PIC remap, PIT timer, exception handlers.

**Threats:**

- Unhandled exceptions leaving CPU in ambiguous state → **mitigated** — dedicated handlers log and halt.
- Spurious or misconfigured IRQ storms → PIC EOI in timer handler; rate-limited serial logging.
- Double-fault without recovery → `#DF` handler logs and halts (IST stacks planned M4).
- Interrupt handler bugs corrupting kernel state → limited attack surface (no user input); Rust + minimal handlers.

See [ADR-0008](../adr/ADR-0008-interrupt-architecture.md).

### Memory management (M3, planned)

**Threats:**

- Use of UEFI memory after boot services exit → boot loader copies map to stable storage;
  kernel marks firmware-reserved regions non-allocatable.
- User mapping of kernel physical frames → separate page tables; supervisor-only PTE flags.
- Writable+executable user pages → W^X enforcement at map time.

### Syscall boundary (M5, planned)

**Threats:**

- Confused deputy via unchecked user pointers → validate against caller address space and
  capabilities ([ADR-0005](../adr/ADR-0005-syscall-abi-strategy.md)).
- Syscall number confusion → fail closed with defined error; no partial side effects.

### Capability model (M5, planned)

**Threats:**

- Forged capability tokens → kernel-issued opaque handles; no user-constructible rights.
- Over-delegation → explicit copy/attenuate operations; auditable delegation log (future).

See [ADR-0004](../adr/ADR-0004-capability-security-model.md).

### Signed updates (M10, planned)

**Host skeleton:** `system/updater/` types and stub verifier.

**Threats:**

- Downgrade to vulnerable kernel → minimum version policy in boot loader (planned).
- Partial slot write → checksum verification before slot activation (planned).
- Forged update manifest → Ed25519 verification against pinned keys (planned).

See [docs/updates/signed-verification.md](../updates/signed-verification.md), [ADR-0009](../adr/ADR-0009-atomic-update-architecture.md).

### Application packaging (M9, planned)

**Host skeleton:** `system/pkgmgr/` manifest and signature stub.

**Threats:**

- Malicious package payload → signature verification + capability-scoped install paths (planned).
- Path traversal on extract → install root confinement (planned).

See [docs/packages/README.md](../packages/README.md).

## Hardware exposure

| Target | Tier | Security note |
|--------|------|---------------|
| QEMU x86_64 + OVMF | Tier 1 dev | Primary test environment; not a production deployment |
| Real PC hardware | Tier 2 experimental | Untested; firmware and Secure Boot behavior vary |

See [docs/hardware/README.md](../hardware/README.md).

## Report scope

**In scope for private disclosure** ([SECURITY.md](../../SECURITY.md)):

- Memory safety violations in project Rust code
- Logic errors in syscall ABI that imply unsafe kernel behavior when implemented
- Boot chain flaws in shipped boot loader or kernel entry code
- Interrupt handler or exception path bugs on shipped M2 code
- CI or release pipeline weaknesses affecting artifact integrity

**Out of scope:**

- Attacks requiring physical access that future Secure Boot is explicitly designed to address
- Hypothetical bugs in subsystems with no code path (unless a concrete PoC on shipped code exists)

## Related documents

- [SECURITY.md](../../SECURITY.md) — reporting, disclosure, safe harbor
- [ARCHITECTURE.md](../../ARCHITECTURE.md) — system design and security model summary
- [ADR-0004](../adr/ADR-0004-capability-security-model.md) — capability security model
- [ADR-0005](../adr/ADR-0005-syscall-abi-strategy.md) — syscall ABI strategy
- [ADR-0006](../adr/ADR-0006-boot-architecture.md) — boot architecture
- [ADR-0008](../adr/ADR-0008-interrupt-architecture.md) — interrupt architecture
- [ADR-0009](../adr/ADR-0009-atomic-update-architecture.md) — atomic updates
- [GOVERNANCE.md](../../GOVERNANCE.md) — security-sensitive change approval
