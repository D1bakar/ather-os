# Build Guide

Reference for building Aether OS on the host, cross-compiling the UEFI loader and bare-metal
kernel, and running under QEMU.

**Current milestone:** M2.

See also: [INSTALL.md](INSTALL.md) (prerequisites) · [DEPLOYMENT.md](DEPLOYMENT.md) (artifacts)

## Toolchain

| Component | Version / notes |
|-----------|-----------------|
| Rust | **1.85.0** — [rust-toolchain.toml](../rust-toolchain.toml) |
| Components | `rustfmt`, `clippy`, `rust-src`, `llvm-tools-preview` |
| UEFI target | `x86_64-unknown-uefi` |
| Kernel target | `x86_64-unknown-none` (requires `-Z build-std`) |
| AArch64 kernel target (M13 scaffold) | `aarch64-unknown-none` — module stub only; **not bootable** |

```bash
rustup target add x86_64-unknown-uefi x86_64-unknown-none
# Optional — M13 prep; no boot path yet:
rustup target add aarch64-unknown-none
export RUSTC_BOOTSTRAP=1   # required for build-std
```

## Makefile targets

| Target | Action |
|--------|--------|
| `make setup` | Run `scripts/setup-dev.sh` / `.ps1` |
| `make build` | `cargo build --workspace` (host) |
| `make boot` | UEFI loader + bare-metal kernel → `build/esp/` |
| `make run` | Build boot artifacts (if needed) and launch QEMU |
| `make test` | fmt, clippy, `cargo test --workspace` |
| `make m2-check` | fmt check, clippy, tests, boot build |
| `make clean` | `cargo clean` + remove `build/` |

## Workspace crates

| Crate | Target | Build |
|-------|--------|-------|
| `aether-types` | Host | `cargo build -p aether-types` |
| `aether-abi` | Host | `cargo build -p aether-abi` |
| `aether-logger` | Host | `cargo build -p aether-logger` |
| `aether-kernel` | Host stub / bare-metal | `--features bare-metal --target x86_64-unknown-none` |
| `aether-boot` | UEFI | `--target x86_64-unknown-uefi --release` |

## Boot artifacts

`scripts/build-boot.sh` (or `.ps1`) produces:

```
build/esp/EFI/BOOT/BOOTX64.EFI
build/esp/aether/kernel.elf
```

Bare-metal kernel build (manual):

```bash
cargo build -p aether-kernel --no-default-features --features bare-metal \
  --target x86_64-unknown-none --release \
  -Z build-std=core,compiler_builtins \
  -Z build-std-features=compiler-builtins-mem
```

UEFI boot loader build (manual):

```bash
cargo build -p aether-boot --target x86_64-unknown-uefi --release
```

## QEMU + OVMF

Install `qemu-system-x86_64` and OVMF firmware. See [INSTALL.md](INSTALL.md) for platform packages.
Place firmware under `ovmf/` or use system paths.

```bash
bash scripts/run-qemu.sh
# Serial log: build/qemu-serial.log
```

| Environment variable | Default | Purpose |
|---------------------|---------|---------|
| `TIMEOUT` | `30` | QEMU run timeout (seconds) |
| `OVMF_CODE` | auto-detect | Override OVMF code firmware path |
| `OVMF_VARS` | auto-detect | Override OVMF vars firmware path |

Set `TIMEOUT=45` for longer runs to capture timer ticks (~1 s interval at 100 Hz PIT).

## Testing

| Test suite | Command | Notes |
|------------|---------|-------|
| Host unit tests | `cargo test --workspace` | Includes GDT/IDT layout tests |
| QEMU boot smoke | `cargo test -p aether-integration-tests -- --ignored` | Requires QEMU + OVMF |
| Full gate | `bash scripts/ci-check.sh` | Mirrors CI quality job |

Integration tests:

- `tests/arch_gdt.rs` — GDT descriptor encoding
- `tests/arch_idt.rs` — IDT gate layout
- `tests/qemu_boot.rs` — QEMU serial boot (ignored by default)

## CI parity

Local full gate:

```bash
bash scripts/ci-check.sh        # Unix
powershell -File scripts/ci-check.ps1   # Windows
```

Mirrors `.github/workflows/ci.yml` quality job: fmt, clippy, tests, workspace build,
UEFI + bare-metal release builds.

Optional QEMU job runs on Ubuntu with `continue-on-error: true`.

## Dev container

Open in VS Code / Cursor Dev Containers — `.devcontainer/devcontainer.json` installs Rust,
QEMU, OVMF, and runs `scripts/setup-dev.sh`.

## Cross-compilation notes

- The bare-metal kernel uses `#![no_std]` with `-Z build-std=core,compiler_builtins`
- `RUSTC_BOOTSTRAP=1` is required for nightly/bootstrap features in stable toolchain
- Kernel `unsafe` is permitted; shared crates use `#![forbid(unsafe_code)]`
- Boot loader uses `uefi` crate ecosystem for UEFI services

## Troubleshooting

| Issue | Fix |
|-------|-----|
| `build-std` errors | Set `RUSTC_BOOTSTRAP=1`; ensure `rust-src` component installed |
| OVMF not found | Install `ovmf` package or copy `OVMF_CODE.fd` / `OVMF_VARS.fd` to `ovmf/` |
| No timer ticks in log | Increase `TIMEOUT=45`; ticks log every ~1 s at 100 Hz |
| Clippy `-D warnings` | Run `cargo fmt --all` first; check `RUSTFLAGS` |
| `x86_64-unknown-none` linker errors | Use `cargo build` with `-Z build-std`; do not invoke `rustc` directly without build-std |
| Windows path issues | Use PowerShell scripts; avoid mixing WSL and native paths in one build |

## Related documents

- [INSTALL.md](INSTALL.md) — developer environment installation
- [DEPLOYMENT.md](DEPLOYMENT.md) — release artifacts and deployment
- [development/getting-started.md](development/getting-started.md) — first boot walkthrough
- [development/code-style.md](development/code-style.md) — Rust kernel conventions
