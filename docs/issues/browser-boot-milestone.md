---
title: "Universal Platform: in-browser UEFI boot via qemu.wasm (Phase 2)"
labels: ["enhancement", "universal-platform"]
assignees: []
---

## Summary

Phase 2 ships live in-browser UEFI boot at https://d1bakar.github.io/ather-os/

## Acceptance criteria

- [x] Browser loads same `BOOTX64.EFI` + `kernel.elf` hashes as manifest (no forked web kernel)
- [x] Serial log contains `Aether OS kernel started` and `Aether init started` (live boot)
- [x] `manifest.boot.browser_runtime.status` updates to `ready` when OVMF bundled
- [x] Document OVMF fetch/redistribution in `web/README.md`
- [x] Mobile: basic viewport notes per [web-threat-model.md](docs/security/web-threat-model.md)
- [x] CI builds web artifacts after boot build + OVMF

## Architecture

Option A (ADR-0010): qemu.wasm from ktock CDN + OVMF bundled by CI + FAT ESP in MEMFS.

## Limitations

- Serial only (no GUI/mouse)
- Mobile experimental
- ~300 MB first-load download (WASM from CDN)

## Local fallback

```powershell
.\scripts\build-boot.ps1
.\scripts\run-qemu.ps1
```
