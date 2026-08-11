# Application Packaging Specification

**Status:** **Planned (M9)** — host-buildable scaffold in `system/pkgmgr/` (`aether-pkgmgr`);
manifest parser, signature stub, and install API exist for **host tests only**. No kernel or
init integration, no runtime package installation.

This document defines the intended format for distributable applications on Aether OS.
It aligns with the capability security model ([ADR-0004](../adr/ADR-0004-capability-security-model.md))
and the modular monolithic kernel ([ADR-0001](../adr/ADR-0001-modular-monolithic-kernel.md)).

## Implementation status

| Component | Path | Status |
|-----------|------|--------|
| Package manager crate (host scaffold) | [`system/pkgmgr/`](../../system/pkgmgr/) | **M9 skeleton** — host-testable only |
| Manifest parser + signature stub | `system/pkgmgr/src/manifest.rs`, `signature.rs` | **M9 skeleton** |
| Install API | `system/pkgmgr/src/install.rs` | **M9 skeleton** |
| Kernel / init integration | — | **Planned** |
| `aether-pkg` CLI | `tools/` (future) | **Planned** |

## Goals

1. **Reproducible installs** — same manifest produces the same installed layout
2. **Capability-scoped permissions** — packages declare required capabilities, not ambient root
3. **Signed artifacts** — maintainers and publishers sign packages; clients verify before install
4. **Dependency resolution** — explicit version constraints with conflict detection
5. **Sandbox-friendly layout** — install paths and metadata support least-privilege execution

## Non-goals (initial M9)

- Cross-distribution compatibility with Linux package formats (deb, rpm)
- Dynamic linking across unrelated publishers without explicit ABI contracts
- App store / commercial distribution infrastructure

## Package format overview

```
example-1.0.0.aetherpkg   # ZIP container (planned)
├── manifest.toml         # Required — package metadata and capabilities
├── manifest.sig          # Required — Ed25519 signature over manifest.toml
├── payload/              # Required — files to install
│   ├── bin/
│   │   └── hello
│   └── share/
│       └── example/
│           └── README
└── CHECKSUMS.sha256      # Required — payload file hashes
```

### File extension

| Extension | Meaning |
|-----------|-----------|
| `.aetherpkg` | Signed application package (M9) |
| `.aetherdevpkg` | Unsigned development package (local install only) |

## Manifest schema (`manifest.toml`)

Illustrative schema — subject to change before M9 implementation.

```toml
[package]
name = "example"
version = "1.0.0"
description = "Example user-space program"
license = "MIT"
authors = ["Aether OS Contributors"]

[package.aether]
min_os_version = "0.6.0"      # Minimum Aether OS version
arch = ["x86_64"]
abi = "aether-syscall-v1"     # Syscall ABI version from aether-abi

[install]
bin = ["payload/bin/hello"]     # Installed to /usr/bin/ (path TBD at M6)
data = ["payload/share/example"]

[capabilities]
# Required capabilities — install fails if user/process lacks grant path
required = [
  "cap:serial.write",           # Write to serial console
  "cap:fs.read:/usr/share/example",
]

[capabilities.optional]
# Optional capabilities — requested at install time
optional = [
  "cap:net.connect",
]

[dependencies]
# Other aether packages
runtime = [
  "aether-libc >= 0.1.0",
]

[dependencies.build]
# Host-only build dependencies (not installed on target)
build = [
  "aether-sdk >= 0.1.0",
]
```

## Capability declarations

Packages must declare all capabilities required at runtime. The package manager maps
declarations to capability grants at install time (subject to user/policy approval).

| Capability pattern | Example | Description |
|--------------------|---------|-------------|
| `cap:serial.write` | — | Write to diagnostic serial |
| `cap:fs.read:<path>` | `cap:fs.read:/etc/aether` | Read access to path prefix |
| `cap:fs.write:<path>` | `cap:fs.write:/var/log` | Write access to path prefix |
| `cap:net.connect` | — | Outbound network connections |
| `cap:proc.spawn` | — | Spawn child processes |

Exact capability taxonomy will be defined alongside M5 capability table implementation.

## Signing and verification

| Phase | Policy |
|-------|--------|
| **Publisher signing** | Package author signs `manifest.toml` + `CHECKSUMS.sha256` with Ed25519 key |
| **Repository signing** | Official repo indexes signed by project maintainers (planned) |
| **Install-time verification** | `aether-pkg install` verifies signature and checksums before extraction |
| **Trust roots** | Pinned public keys in `/etc/aether/trusted-keys/` (path TBD) |

Unsigned `.aetherdevpkg` packages may be installed with `--insecure` for local development only.

## Installation layout (planned)

| Path | Content |
|------|---------|
| `/usr/bin/` | Executable binaries |
| `/usr/lib/<package>/` | Package-private libraries |
| `/usr/share/<package>/` | Static data files |
| `/etc/aether/packages/<name>/` | Installed manifest copy and capability grant record |
| `/var/lib/aether/packages/<name>/` | Mutable package state |

Exact paths depend on VFS layout finalized at M6.

## Package manager CLI (`aether-pkg`) — planned

```bash
aether-pkg install example-1.0.0.aetherpkg
aether-pkg remove example
aether-pkg list
aether-pkg search hello
aether-pkg verify example-1.0.0.aetherpkg
aether-pkg build .                  # Build package from source tree
```

## Dependency resolution

1. Parse manifest dependencies with semver constraints
2. Resolve transitive dependencies from local index or remote repository
3. Detect conflicts (two packages requiring incompatible versions of the same dependency)
4. Topological sort for install order
5. Verify all packages before any filesystem mutation (atomic install transaction)

## Build pipeline (publisher workflow)

```mermaid
graph LR
    SRC[Source tree] --> BUILD[cargo build --target aether-user]
    BUILD --> STAGE[Stage payload/]
    STAGE --> MANIFEST[Generate manifest.toml]
    MANIFEST --> HASH[Compute CHECKSUMS.sha256]
    HASH --> SIGN[Sign manifest]
    SIGN --> PKG[.aetherpkg archive]
```

## Relationship to OS updates

Application packages (M9) are distinct from OS update bundles (M10).
OS updates replace kernel, boot loader, and system components atomically.
Application packages install into user space without reboot when possible.

See [updates/README.md](../updates/README.md).

## Security considerations

- Packages cannot self-grant capabilities — grants require policy approval
- Payload paths must not escape install root (path traversal checks at extract)
- Setuid/setgid binaries are **not supported** — use capability delegation instead
- Package scripts (pre/post install) are deferred; initial M9 is file-only payloads

## Implementation milestones

| Milestone | Deliverable |
|-----------|-------------|
| M5 | Capability types and delegation API |
| M6 | VFS paths for install layout |
| M9 | `aether-pkg` CLI, manifest parser, signing |
| M10 | Official signed package repository |

## Related documents

- [ADR-0004](../adr/ADR-0004-capability-security-model.md) — capability security model
- [ADR-0005](../adr/ADR-0005-syscall-abi-strategy.md) — syscall ABI
- [ROADMAP.md](../ROADMAP.md) — M9 milestone
- [updates/README.md](../updates/README.md) — OS update bundles
- [ARCHITECTURE.md](../../ARCHITECTURE.md) — filesystem strategy
