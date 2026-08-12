# Aether Universal Platform (web)

Static site and in-browser UEFI boot for **real** Aether OS artifacts.

## Current status

**Live browser boot** is available when OVMF firmware is bundled (GitHub Pages CI). The demo runs the same `BOOTX64.EFI` + `kernel.elf` as local QEMU via [qemu.wasm](https://github.com/ktock/qemu-wasm) (fetched from CDN).

| Capability | Status |
|------------|--------|
| SHA-256 manifest + artifact download | Shipped |
| Live COM1 serial in browser | Shipped (desktop browsers) |
| GOP / mouse / GUI | Not shipped |
| Mobile | Experimental (memory limits) |

See [ADR-0010](../docs/adr/ADR-0010-browser-vm-architecture.md).

## Architecture

```
Browser page (main thread)
  coi-serviceworker.js  → COOP/COEP for WASM pthreads
  artifact-loader.js    → fetch + SHA-256 verify OVMF + ESP
  qemu-emulator.js      → MEMFS mount, import qemu-system-x86_64.js (CDN)
  xterm-pty             → bridge -serial stdio to #serial-pane
```

QEMU binary is **not** committed (size/GPL). Default CDN: `ktock.github.io/qemu-wasm-demo/images/alpine-x86_64/`.

OVMF is copied from the host `ovmf` package during CI (`apt install ovmf`) into `artifacts/firmware/`.

## Prerequisites

- Rust toolchain (same as root repo)
- **Local browser boot:** install OVMF (e.g. QEMU/OVMF for Windows, `ovmf` package on Linux)
- **Local native boot:** QEMU + OVMF — `../scripts/run-qemu.ps1`

## Quick start

From repository root:

```powershell
.\scripts\build-boot.ps1
.\scripts\build-web-artifacts.ps1   # status=ready when OVMF found
.\web\serve.ps1
```

Open http://localhost:8080 — click **BOOT AETHER**.

**Live demo:** https://d1bakar.github.io/ather-os/

## Directory layout

```
web/
├── public/
│   ├── index.html          Boot demo UI
│   ├── js/app.js           Manifest + boot button
│   ├── js/boot.js          Boot orchestration
│   ├── js/coi-serviceworker.js
│   ├── manifest.json       Generated
│   ├── artifacts/          ESP + OVMF (CI)
│   └── vm/                 Copied from ../vm/ at build time
└── vm/
    ├── qemu-emulator.js
    ├── artifact-loader.js
    └── worker.js
```

## OVMF redistribution

OVMF firmware is **not** in git. CI installs the distribution package and copies files into `web/public/artifacts/firmware/` with checksums in `manifest.json`. License: BSD-2-Clause-Patent (TianoCore/OVMF).

## Bundle size notes

| Asset | Approx. size | Source |
|-------|--------------|--------|
| Aether ESP | ~500 KiB | Repo / CI build |
| OVMF | ~8 MiB | CI `ovmf` package |
| qemu.wasm | ~25–40 MiB | CDN lazy load |
| aether.img (optional) | 64 MiB | Not required for browser boot (FAT ESP used) |

GitHub Pages soft limit is 100 MiB per deployment; we stay under by CDN-loading the emulator.

## Security

Artifact integrity uses SHA-256 in `manifest.json`. Full web threat model: [docs/security/web-threat-model.md](../docs/security/web-threat-model.md).
