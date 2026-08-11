# System — Userspace Components

> **Status:** M10 adds GUI foundations; M6 init and service manifests remain planned.

This directory holds system-level user-space components for Aether OS.

| Component | Path | Milestone | Status |
|-----------|------|-----------|--------|
| GUI foundations | [`window/`](window/), [`gui-ipc/`](gui-ipc/), [`compositor/`](compositor/), [`desktop/`](desktop/), [`terminal/`](terminal/), [`gui-demo/`](gui-demo/) | M10 | **Prototype** — host demo |
| Package manager stub | [`pkgmgr/`](pkgmgr/) | M11 | **Skeleton** |
| Atomic updater | [`updater/`](updater/) | M12 | **Skeleton** |

Nothing in this directory is loaded by the kernel in M0–M2.

## Host GUI demo (M10)

```bash
cargo run -p aether-gui-demo --bin aether-compositor-demo
```

Writes `target/aether-desktop-demo.ppm` (800×600 desktop with taskbar and terminal window).

## Crates

| Crate | Role |
|-------|------|
| `aether-window` | `Window`, `SurfaceBuffer`, `embedded-graphics` draw target |
| `aether-gui-ipc` | Message queue + shared-memory IPC stub (compositor ↔ apps) |
| `aether-compositor` | Display back-buffer, window stacking, IPC dispatch |
| `aether-desktop` | Taskbar panel and launcher stubs |
| `aether-terminal` | Framebuffer terminal emulator stub |
| `aether-gui-demo` | Host-buildable demo binary |
