# Aether OS Update Architecture

> **Status:** M12 skeleton — documentation and host-testable types only.
> Runtime update application, disk I/O, and boot-chain integration are planned.

Aether OS delivers OS updates as **cryptographically signed**, **atomically applied** images
using an **A/B partition** model with automatic rollback on boot failure.

## Design goals

1. **Atomicity** — an update either fully commits or leaves the previously bootable slot intact.
2. **Verified trust** — every payload is signed by a maintainer key pinned in the boot chain.
3. **Recoverability** — failed boots roll back to the last known-good slot without user intervention.
4. **Auditability** — manifest fields and slot states are versioned and host-validatable in CI.

## Documents

| Document | Topic |
|----------|-------|
| [ab-partitions.md](ab-partitions.md) | A/B slot layout, boot control block, staging flow |
| [signed-verification.md](signed-verification.md) | Manifest format, Ed25519 policy, key pinning |
| [rollback-api.md](rollback-api.md) | Rollback triggers, API semantics, quarantine |

## Implementation

| Component | Path | Status |
|-----------|------|--------|
| Updater crate (types + stubs) | [`system/updater/`](../../system/updater/) | **M12 skeleton** |
| Host manifest checker | [`scripts/update-check.ps1`](../../scripts/update-check.ps1) | **M12 skeleton** |
| Boot loader slot selection | `boot/` | **Planned** |
| Init-time apply daemon | `system/updater/` (runtime) | **Planned** |

## Related references

- [ARCHITECTURE.md § Update strategy](../../ARCHITECTURE.md#update-strategy)
- [ADR-0009: Atomic update architecture](../adr/ADR-0009-atomic-update-architecture.md)
- [Threat model § Signed updates](../security/threat-model.md)
