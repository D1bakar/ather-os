# Security Policy

## Supported versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | Active development (M2 CPU / interrupts) |

**Important:** Aether OS is in early development. M2 delivers UEFI boot, bare-metal kernel
entry, GDT/IDT/PIC/PIT interrupts, and serial diagnostics in QEMU. There is **no user space**,
**no syscall dispatch**, **no paging isolation**, and **no runtime security enforcement**
beyond partial `BootInfo` validation. Statements below describe **design intent** and
**reporting process**, not production guarantees.

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

The full threat model — assets, adversaries, trust boundaries, subsystem-specific threats,
and security principles with honest status markers — is maintained in:

**[docs/security/threat-model.md](docs/security/threat-model.md)**

Summary of current maturity (M2):

| Area | M2 status |
|------|-----------|
| Boot handoff validation | **Partial** — magic/version only |
| Kernel isolation / paging | **Planned** M3 |
| Interrupt / exception handling | **Shipped** — log and halt on unexpected exceptions |
| Syscall boundary enforcement | **Planned** M5 |
| Capability model | **Planned** M5 |
| Signed updates | **Planned** M10 (host skeleton in `system/updater/`) |

## Boot architecture (security-relevant)

**Status:** M1 boot path **shipped**; signature verification **not implemented**.

Shipped properties:

- Fixed-layout `BootInfo` with magic and version validation at kernel entry.
- Boot loader exits UEFI boot services before kernel entry.

Planned properties:

- Full validation of memory-map pointers before allocator use (M3).
- Signature verification of `kernel.elf` and update payloads (M10).

See [ARCHITECTURE.md](ARCHITECTURE.md), [docs/security/threat-model.md](docs/security/threat-model.md), and [ADR-0006](docs/adr/ADR-0006-boot-architecture.md).

## Interrupt and exception handling (M2)

**Status:** **shipped** — CPU exceptions log vector to serial and halt; timer IRQ uses PIC EOI discipline.

Security-relevant properties:

- No silent recovery from unexpected exceptions in early milestones.
- Double-fault handler logs and halts (no stack recovery yet).
- Interrupts enabled only after IDT and PIC are configured.

See [ADR-0008](docs/adr/ADR-0008-interrupt-architecture.md).

## Capability model (design intent)

**Status:** not implemented at runtime.

Access to kernel-mediated objects (files, devices, ports, memory mappings) will
require unforgeable **capabilities** held in per-process tables. Delegation copies
or attenuates rights explicitly; there is no global "root" capability for ordinary
processes.

See [ADR-0004](docs/adr/ADR-0004-capability-security-model.md) and [docs/security/threat-model.md](docs/security/threat-model.md).

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

See [ARCHITECTURE.md](ARCHITECTURE.md) and [docs/security/threat-model.md](docs/security/threat-model.md).

## Update and packaging security (design intent)

**Status:** host-testable scaffolds only; no runtime apply or install.

| Component | Path | Security note |
|-----------|------|---------------|
| Update types + verify stub | `system/updater/` | Stub verifier accepts dev fixtures; Ed25519 planned M10 |
| Package manager scaffold | `system/pkgmgr/` | Signature stub; no kernel-mediated install |

Specifications: [docs/updates/README.md](docs/updates/README.md), [docs/packages/README.md](docs/packages/README.md).

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
- Boot chain flaws in shipped boot loader or kernel entry code
- Interrupt handler bugs causing memory corruption or silent privilege escalation
- CI or release pipeline weaknesses affecting artifact integrity

**Out of scope:**

- Theoretical attacks against unimplemented subsystems without a concrete code path on shipped code
- Issues requiring physical access that Secure Boot is explicitly designed to address later
- Vulnerabilities in QEMU, OVMF, or host Rust toolchain (report upstream)

Details: [docs/security/threat-model.md](docs/security/threat-model.md).

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

- [docs/security/threat-model.md](docs/security/threat-model.md) — full threat model
- [ARCHITECTURE.md](ARCHITECTURE.md) — system architecture and security model summary
- [docs/adr/](docs/adr/) — architecture decisions (security-relevant: ADR-0004, ADR-0005, ADR-0006, ADR-0008, ADR-0009)
- [docs/development/code-style.md](docs/development/code-style.md) — `unsafe` and ABI conventions
- [GOVERNANCE.md](GOVERNANCE.md) — security-sensitive change approval
