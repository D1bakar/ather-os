---
title: "Universal Platform: in-browser UEFI boot via qemu.wasm (Phase 2)"
labels: ["enhancement", "universal-platform", "blocked:uefi-in-browser"]
assignees: []
---

## Summary

Phase 1 of the Universal Platform ([ADR-0010](docs/adr/ADR-0010-browser-vm-architecture.md)) ships the manifest pipeline, honest landing page, and VM worker stub. **In-browser boot of real Aether artifacts is blocked** until we integrate a UEFI-capable emulator.

## Problem

Aether OS boots exclusively via UEFI today:

- `EFI/BOOT/BOOTX64.EFI` (Rust UEFI loader)
- `aether/kernel.elf` on a FAT32 ESP
- Verified with QEMU + OVMF (`scripts/run-qemu.ps1`)

**v86 / SeaBIOS cannot boot this chain:**

- No OVMF/UEFI firmware support ([copy/v86#263](https://github.com/copy/v86/issues/263))
- No x86_64 64-bit guest support
- Placing `OVMF.fd` as the v86 "bios" does not work (missing UEFI environment)

Adding a BIOS/multiboot-only web boot path would require kernel/boot rewrites — explicitly out of scope.

## Proposed solution (Phase 2)

Integrate **qemu.wasm** ([ktock/qemu-wasm](https://github.com/ktock/qemu-wasm) or upstream wasm64 TCG):

1. Preload artifacts from `web/public/manifest.json` (SHA-256 verified)
2. Configure QEMU args matching local smoke test:
   - `-machine q35`
   - OVMF pflash drives (`OVMF_CODE.fd`, writable `OVMF_VARS.fd`)
   - FAT ESP or `aether.img`
   - `-serial stdio` → Web Worker → `#serial-pane`
3. **No fake terminal UI** — display only real COM1 output from the guest
4. Optional: GOP framebuffer when M8 ships

## Acceptance criteria

- [ ] Browser loads same `BOOTX64.EFI` + `kernel.elf` hashes as manifest (no forked web kernel)
- [ ] Serial log contains `Aether OS kernel started` and `Aether init started`
- [ ] `manifest.boot.browser_runtime.status` updates to `ready`
- [ ] Document OVMF fetch/redistribution in `web/README.md`
- [ ] Mobile: basic viewport + touch notes per [web-threat-model.md](docs/security/web-threat-model.md)
- [ ] CI optional job builds web artifacts after boot build

## Interim workaround

Local QEMU remains the verified path:

```powershell
.\scripts\build-boot.ps1
.\scripts\run-qemu.ps1
```

## References

- ADR-0010: Browser VM architecture
- Web threat model: `docs/security/web-threat-model.md`
- Phase 1 scaffold: `web/`, `scripts/build-web-artifacts.ps1`
