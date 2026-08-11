# Security Policy

## Supported versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | Active development (pre-boot) |

**Important:** Aether OS is in early development. M0 provides shared libraries,
documentation, and build infrastructure only. There is **no bootable kernel** and
**no runtime security enforcement** yet. Statements below describe **design intent**
and **reporting process**, not shipped guarantees.

## Reporting a vulnerability

**Do not report security vulnerabilities through public GitHub issues.**

Report privately by emailing: **security@aether-os.dev**

Include:

- Description of the vulnerability
- Steps to reproduce
- Potential impact assessment
- Suggested fix (if any)

We aim to acknowledge reports within **48 hours** and provide an initial assessment
within **7 days**. Response times may vary during early development when no production
releases exist.

## Threat model

### Assets

| Asset | M0 status | Future intent |
|-------|-----------|---------------|
| Kernel memory and control flow | Not running | Protect from user-space and driver corruption |
| Syscall boundary | ABI defined only | Validate all user pointers and capabilities per call |
| Boot chain integrity | Not implemented | Verified boot loader → kernel handoff and signed updates |
| User data on disk | Not implemented | Encryption and access control via capabilities |
| Build artifacts and release keys | CI only | Reproducible builds and signed releases |

### Adversaries (design assumptions)

1. **Malicious user-space process** — attempts privilege escalation via syscalls, confused deputies, or kernel memory corruption.
2. **Malicious or compromised driver** — attempts to bypass capability checks (mitigated by least privilege and future driver isolation options).
3. **Local attacker with physical access** — can modify disk images, attempt unsigned boot, or attach debuggers (out of scope for pure software mitigations; secure boot intent documented for future milestones).
4. **Supply-chain attacker** — compromises dependencies, CI, or release signing infrastructure.

### Out of scope (initial milestones)

- Network remote exploitation (no network stack in early milestones).
- Side-channel attacks (Spectre/Meltdown class) — acknowledged, not mitigated in M0–M1.
- Denial of service via resource exhaustion in unprivileged user code, unless it crashes or hangs the kernel.
- Vulnerabilities in third-party tools (QEMU, OVMF, host Rust toolchain) — report upstream.

### Security principles (design intent)

| Principle | Description | M0 status |
|-----------|-------------|-----------|
| **Fail closed** | Invalid syscalls, capabilities, or boot handoff data are rejected | Planned |
| **Least privilege** | Processes hold explicit capabilities, not ambient root | Planned |
| **Memory safety** | Rust in shared crates; documented `unsafe` in kernel only | Partial (shared crates) |
| **Explicit ABI** | Small syscall surface defined in `aether-abi` | ABI scaffold only |
| **Signed updates** | Atomic, verified OS updates | Planned |
| **Reproducible builds** | Deterministic artifact generation where feasible | Intent documented (ADR-0007) |

## Boot architecture (security-relevant)

**Status:** not implemented.

Planned properties:

- Fixed-layout `BootInfo` handoff validated by the kernel before use.
- Boot loader exits UEFI boot services before kernel entry to reduce firmware attack surface during runtime.
- Future: signature verification of `kernel.elf` and update payloads before execution.

See [ARCHITECTURE.md](ARCHITECTURE.md) and [ADR-0006](docs/adr/ADR-0006-boot-architecture.md).

## Capability model (design intent)

**Status:** not implemented.

Access to kernel-mediated objects (files, devices, ports, memory mappings) will
require unforgeable **capabilities** held in per-process tables. Delegation copies
or attenuates rights explicitly; there is no global "root" capability for ordinary
processes.

See [ADR-0004](docs/adr/ADR-0004-capability-security-model.md).

## Syscall boundary (design intent)

**Status:** ABI types and numbers in `aether-abi`; no kernel dispatch.

Planned controls:

- Validate every user pointer against the caller's address space and capability set.
- Reject unknown syscall numbers with a defined error.
- Avoid implicit string formatting or allocation on untrusted data in kernel paths.

See [ADR-0005](docs/adr/ADR-0005-syscall-abi-strategy.md).

## Memory model (security-relevant)

**Status:** address/page types in `aether-types`; no allocator or page tables.

Planned properties:

- Separate user and kernel page tables per process.
- No executable writable mappings for user space (W^X intent).
- Kernel mappings not accessible from user mode.

## Supported hardware and exposure

| Target | Tier | Security note |
|--------|------|---------------|
| QEMU x86_64 + OVMF | Tier 1 dev | Primary test environment; not a production deployment |
| Real PC hardware | Tier 2 experimental | Untested; firmware and Secure Boot behavior vary |

See [docs/hardware/README.md](docs/hardware/README.md).

## Scope of reports

**In scope:**

- Memory safety violations in project Rust code
- Logic errors in syscall ABI definitions that imply unsafe kernel behavior
- Boot chain design flaws once boot code exists
- CI or release pipeline weaknesses affecting artifact integrity

**Out of scope:**

- Theoretical attacks against unimplemented subsystems without a concrete code path
- Issues requiring physical access that secure boot is explicitly designed to address later

## Disclosure policy

We follow coordinated disclosure. We will work with reporters to understand and fix
issues before public disclosure. Credit will be given unless anonymity is requested.

## Safe harbor

We consider security research conducted in good faith to be authorized. We will not
pursue legal action against researchers who:

- Make a good faith effort to avoid privacy violations and data destruction
- Report vulnerabilities promptly
- Allow reasonable time for remediation before public disclosure

## Related documents

- [ARCHITECTURE.md](ARCHITECTURE.md)
- [docs/adr/](docs/adr/)
- [GOVERNANCE.md](GOVERNANCE.md)
