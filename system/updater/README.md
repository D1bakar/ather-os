# Aether OS Updater

> **Status:** M12 skeleton — design types and host-testable verification stubs only.
> No runtime update application, disk I/O, or boot-chain integration yet.

This crate defines the **A/B partition model**, **signed update verification** interface,
and **rollback API** for atomic OS updates. It is intended to run in user-space init
once M6 exists; host builds support manifest validation in CI via `scripts/update-check.ps1`.

## Documentation

- [docs/updates/README.md](../../docs/updates/README.md) — update architecture overview
- [docs/updates/ab-partitions.md](../../docs/updates/ab-partitions.md) — slot layout
- [docs/updates/signed-verification.md](../../docs/updates/signed-verification.md) — signature policy
- [docs/updates/rollback-api.md](../../docs/updates/rollback-api.md) — rollback semantics

## Modules

| Module | Purpose |
|--------|---------|
| `partition` | A/B slot identifiers, boot control block, active/inactive slot selection |
| `verify` | Update manifest parsing and Ed25519 signature verification skeleton |
| `rollback` | Rollback request types and manager API (no firmware reboot yet) |
| `error` | Update-specific error codes |

## Host validation

```powershell
cargo test -p aether-updater
powershell -File scripts/update-check.ps1
```
