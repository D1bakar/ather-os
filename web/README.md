# Aether Universal Platform (web)

Static site and VM worker stubs for delivering **real** Aether OS boot artifacts to browsers.

## Current status

**In-browser boot is not available yet.** Aether requires UEFI (`BOOTX64.EFI` + OVMF). The v86
emulator only provides SeaBIOS and cannot boot our ESP layout. Phase 2 targets [qemu.wasm](https://github.com/ktock/qemu-wasm).

This directory provides:

- Honest landing page with manifest metadata and artifact checksums
- Release manifest pipeline (`scripts/build-web-artifacts.ps1`)
- Web Worker stub prepared for qemu.wasm serial bridging (no fake OS UI)

See [ADR-0010](../docs/adr/ADR-0010-browser-vm-architecture.md).

## Prerequisites

- Rust toolchain (same as root repo)
- PowerShell (Windows) or adapt `build-web-artifacts.ps1` for bash
- **Local boot today:** QEMU + OVMF — `../scripts/run-qemu.ps1`

## Quick start

From repository root:

```powershell
# Build kernel + boot loader into build/esp/
.\scripts\build-boot.ps1

# Copy artifacts + generate web/public/manifest.json
.\scripts\build-web-artifacts.ps1

# Serve landing page (static files in public/)
cd web
npm run serve
```

Or from repository root: `.\web\serve.ps1` (Windows) / `./web/serve.sh` (Unix).

Open http://localhost:8080 — the page loads `manifest.json` and lists artifact SHA-256 hashes.

**Live demo:** https://d1bakar.github.io/ather-os/

## Directory layout

```
web/
├── package.json
├── README.md
├── public/
│   ├── index.html       Landing page
│   ├── css/style.css
│   ├── js/app.js        Manifest loader + VM status
│   ├── manifest.json    Generated (do not hand-edit)
│   ├── artifacts/       Generated ESP copy
│   └── vm/              Copied from ../vm/ at build time
└── vm/
    ├── worker.js        Source — copied to public/vm/
    └── emulator-stub.js Placeholder until qemu.wasm is integrated
```

## OVMF redistribution

OVMF firmware is **not** bundled in this repo (license/size). For local QEMU, install via your
OS package manager. For future browser boot, OVMF will be fetched or preloaded separately with
license attribution documented here.

## Mobile

Touch keyboard and viewport handling are specified in
[docs/security/web-threat-model.md](../docs/security/web-threat-model.md). Not implemented in Phase 1.

## Security

Artifact integrity uses SHA-256 in `manifest.json`. Do not boot images that fail manifest
verification. Full web threat model: [docs/security/web-threat-model.md](../docs/security/web-threat-model.md).
