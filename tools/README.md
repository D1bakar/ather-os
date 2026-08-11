# Development Tools

Host-side utilities for building, inspecting, and testing Aether OS.

## M0

Build orchestration lives in:

- [scripts/build.ps1](../scripts/build.ps1) — Windows quality gate
- [scripts/build.sh](../scripts/build.sh) — Unix quality gate
- [Makefile](../Makefile) — primary build targets

Future tools (disk image builder, boot log parser, syscall trace decoder) will be
added as Rust binaries under `tools/` when M1 boot artifacts exist.
