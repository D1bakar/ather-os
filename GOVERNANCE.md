# Aether OS Governance

This document describes how the Aether OS open-source project is governed during
early development (M0–M1). It will evolve as the contributor base grows.

## Project goals

1. Build a **security-first** operating system with honest documentation and reproducible engineering practices.
2. Prefer **small, reviewable changes** with recorded architecture decisions.
3. Maintain **upstream-friendly** licensing (MIT) and transparent security disclosure.

## Roles

| Role | Responsibility |
|------|----------------|
| **Maintainers** | Merge approved PRs, cut releases, triage security reports, steward ADRs |
| **Contributors** | Propose changes via pull requests, participate in review |
| **Security researchers** | Report vulnerabilities per [SECURITY.md](SECURITY.md) |

Maintainers are listed in [.github/CODEOWNERS](.github/CODEOWNERS). The list expands by maintainer consensus and documented in this file when formalized.

## Decision making

### Routine changes

- Bug fixes, documentation, tests, and small features: **one maintainer approval** after CI passes.
- Contributors open PRs from forks; maintainers review for correctness, scope, and alignment with ADRs.

### Architecture changes

- Significant design choices require a new or updated **Architecture Decision Record** in `docs/adr/`.
- ADRs use status values: `Proposed`, `Accepted`, `Deprecated`, `Superseded`.
- Superseding an ADR must reference the replacing ADR number.

### Security-sensitive changes

- Changes affecting boot chain, syscall ABI stability, capability model, or update verification require **two maintainer approvals** when two or more maintainers are available.
- Security fixes may be developed in private coordination with reporters before public merge.

### Breaking changes

- Syscall ABI breaks require a new ADR, version bump policy, and migration notes in [CHANGELOG.md](CHANGELOG.md).
- Breaking changes are avoided during active milestone delivery unless necessary for correctness or security.

## Release process (early stage)

During M0–M1 there are no stable release artifacts. When bootable images exist:

1. Tag releases with semantic versioning (`v0.x.y`).
2. Publish signed release notes and checksums.
3. Record changes in [CHANGELOG.md](CHANGELOG.md).

## Code of conduct

All participants must follow [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md). Maintainers may restrict participation for violations.

## Intellectual property

Contributions are licensed under the [MIT License](LICENSE) unless otherwise agreed in writing. Contributors assert they have the right to submit their work.

## Escalation

Disagreements on technical direction should be resolved through:

1. Discussion on the relevant issue or PR.
2. A proposed ADR if the dispute is architectural.
3. Maintainer decision documented in the ADR or PR thread.

## Amendments

Changes to this governance document require a PR with at least one maintainer approval and a summary in the CHANGELOG.
