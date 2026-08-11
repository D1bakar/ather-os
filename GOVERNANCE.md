# Aether OS Governance

This document describes how the Aether OS open-source project is governed. It applies
during early development (M0–M2) and will evolve as the contributor base and release
cadence grow.

## Project goals

1. Build a **security-first** operating system with honest documentation and reproducible engineering practices.
2. Prefer **small, reviewable changes** with recorded architecture decisions (ADRs).
3. Maintain **upstream-friendly** licensing (MIT) and transparent security disclosure.
4. Mark **shipped vs planned** behavior clearly in architecture and security documentation.

## Roles and responsibilities

| Role | Responsibilities | How to engage |
|------|------------------|---------------|
| **Maintainers** | Merge approved PRs; cut releases; triage security reports; accept or supersede ADRs; enforce code of conduct | Listed in [.github/CODEOWNERS](.github/CODEOWNERS); `@` mention on PRs |
| **Contributors** | Propose changes via pull requests; participate in review; follow [CONTRIBUTING.md](CONTRIBUTING.md) and [code style](docs/development/code-style.md) | Fork, branch, open PR |
| **Security researchers** | Report vulnerabilities privately per [SECURITY.md](SECURITY.md) | Email **security@aether-os.dev** |
| **Users / downstream** | Consume releases; file issues for bugs in **shipped** behavior | GitHub Issues (not for security reports) |

### Maintainers

Maintainers are listed in [.github/CODEOWNERS](.github/CODEOWNERS). The roster expands by
**maintainer consensus** and is reflected in CODEOWNERS and this document when formalized.

Maintainer duties include:

- Reviewing PRs within a reasonable timeframe
- Ensuring CI passes before merge
- Requesting ADRs for architectural changes
- Coordinating security fixes with reporters before public disclosure

### Becoming a maintainer

There is no fixed election schedule in early development. Maintainers may invite new
maintainers when a contributor has:

- A sustained history of quality PRs and reviews
- Demonstrated alignment with project goals and ADRs
- Agreement from existing maintainers (documented in a PR updating CODEOWNERS)

## Decision making

### Routine changes

| Change type | Approval | Requirements |
|-------------|----------|--------------|
| Bug fixes, docs, tests, small features | **One maintainer** | CI green; scope aligned with open milestones |
| Typo / formatting only | **One maintainer** | CI green |

Contributors open PRs from forks; maintainers review for correctness, scope, and ADR alignment.

### Architecture changes

Significant design choices require a new or updated **Architecture Decision Record** in
[docs/adr/](docs/adr/):

- New subsystems or module boundaries
- Changes to boot handoff, interrupt model, memory layout, or scheduler design
- Syscall ABI additions or breaking changes
- Security model changes

ADR lifecycle:

| Status | Meaning |
|--------|---------|
| **Proposed** | Under review in a PR |
| **Accepted** | Approved by maintainers; design intent is project direction |
| **Deprecated** | No longer recommended; retained for history |
| **Superseded** | Replaced by a newer ADR (must reference replacing number) |

Architecture documentation ([ARCHITECTURE.md](ARCHITECTURE.md), [docs/architecture/](docs/architecture/))
must be updated when an ADR is accepted if the change affects the documented system.

### Security-sensitive changes

Changes affecting the boot chain, syscall ABI stability, capability model, interrupt
architecture, or update verification require **two maintainer approvals** when two or more
maintainers are available.

Security fixes may be developed in private coordination with reporters before public merge.
See [SECURITY.md](SECURITY.md) and [docs/security/threat-model.md](docs/security/threat-model.md).

### Breaking changes

- Syscall ABI breaks require a new ADR, version bump policy, and migration notes in [CHANGELOG.md](CHANGELOG.md).
- `BootInfo` layout breaks require `BOOT_INFO_VERSION` increment and coordinated boot loader + kernel update.
- Breaking changes are avoided during active milestone delivery unless necessary for correctness or security.

## Release process

| Phase | Policy |
|-------|--------|
| **M0–M1 (current)** | No stable release artifacts; tags document milestones |
| **M2+** | Semantic versioning (`v0.x.y`); signed release notes and checksums when signing infrastructure exists |

When bootable images are published:

1. Tag releases with semantic versioning.
2. Publish release notes and checksums in [CHANGELOG.md](CHANGELOG.md).
3. Record security-relevant changes in [SECURITY.md](SECURITY.md) supported-versions table.

## Documentation standards

- [ARCHITECTURE.md](ARCHITECTURE.md) and subsystem docs must distinguish **shipped**, **partial**, and **planned** behavior.
- Security assumptions live in [docs/security/threat-model.md](docs/security/threat-model.md); [SECURITY.md](SECURITY.md) covers reporting and policy.
- Developer onboarding: [docs/development/getting-started.md](docs/development/getting-started.md).

## Code of conduct

All participants must follow [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md). Maintainers may restrict participation for violations.

## Intellectual property

Contributions are licensed under the [MIT License](LICENSE) unless otherwise agreed in writing. Contributors assert they have the right to submit their work.

## Escalation and dispute resolution

Disagreements on technical direction should be resolved through:

1. **Discussion** on the relevant issue or PR thread.
2. **ADR proposal** if the dispute is architectural — status remains `Proposed` until consensus or maintainer decision.
3. **Maintainer decision** documented in the ADR outcome or PR merge rationale.

For conduct issues, contact maintainers privately; see [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## Amendments to this document

Changes to governance require:

- A pull request with a clear summary of what changed and why
- At least **one maintainer approval**
- An entry in [CHANGELOG.md](CHANGELOG.md) under `Changed` or `Added`

## Related documents

- [CONTRIBUTING.md](CONTRIBUTING.md) — contribution workflow
- [SECURITY.md](SECURITY.md) — vulnerability reporting
- [docs/security/threat-model.md](docs/security/threat-model.md) — threat model
- [docs/adr/README.md](docs/adr/README.md) — ADR index
- [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)
