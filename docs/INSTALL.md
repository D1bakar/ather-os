# Installation Guide

This guide covers installing the **developer toolchain** required to build and test Aether OS.
Aether OS does not yet ship installable release images for real hardware — this document
describes how to set up a development environment.

**Current milestone:** M2. Boot testing requires QEMU + OVMF.

## System requirements

| Resource | Minimum | Recommended |
|----------|---------|-------------|
| OS | Linux, macOS, Windows (WSL2 or native PowerShell) | Ubuntu 22.04+ or Dev Container |
| RAM | 4 GiB host | 8 GiB+ |
| Disk | 2 GiB free (Rust toolchain + build artifacts) | 5 GiB+ |
| CPU | x86_64 host (any) | Multi-core for faster builds |

Aether OS targets **x86_64 bare metal** with **UEFI**. You do not need x86_64-specific host
hardware beyond what Rust and QEMU require.

## Step 1 — Install Rust

Install [rustup](https://rustup.rs/). The repository pins Rust **1.85.0** via
[rust-toolchain.toml](../rust-toolchain.toml); it is selected automatically when you run
`cargo` inside the repository.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Verify:

```bash
cd /path/to/aether-os
rustc --version   # should report 1.85.0
```

## Step 2 — Run developer setup

The setup script installs Rust components, cross-compilation targets, and prints QEMU/OVMF hints.

**Unix / macOS / WSL / Dev Container:**

```bash
make setup
# or: bash scripts/setup-dev.sh
```

**Windows (PowerShell):**

```powershell
make setup
# or: .\scripts\setup-dev.ps1
```

### What setup installs

| Component | Purpose |
|-----------|---------|
| `rustfmt` | Code formatting (`make fmt`) |
| `clippy` | Linting (`make clippy`) |
| `rust-src` | Kernel `build-std` for `x86_64-unknown-none` |
| `llvm-tools-preview` | Coverage and debugging utilities |
| Target `x86_64-unknown-uefi` | UEFI boot loader |
| Target `x86_64-unknown-none` | Bare-metal kernel |

## Step 3 — Install QEMU and OVMF (optional but recommended)

Required for `make run` and the QEMU boot smoke test.

### Linux (Debian / Ubuntu)

```bash
sudo apt-get update
sudo apt-get install -y qemu-system-x86 ovmf
```

OVMF firmware is typically at `/usr/share/OVMF/OVMF_CODE.fd`.

### Linux (Fedora)

```bash
sudo dnf install qemu-system-x86 edk2-ovmf
```

### macOS (Homebrew)

```bash
brew install qemu
```

Download OVMF separately or copy firmware files to `ovmf/` in the repository root.

### Windows

1. Install [QEMU for Windows](https://www.qemu.org/download/#windows) or use Chocolatey:
   ```powershell
   choco install qemu
   ```
2. Copy `OVMF_CODE.fd` and `OVMF_VARS.fd` to `ovmf/` in the repository root, or install
   via a UEFI firmware package and note the path.

The run scripts search common paths and fall back to `ovmf/` in the repo.

## Step 4 — Verify installation

```bash
make build    # host workspace
make boot     # UEFI loader + kernel.elf → build/esp/
make test     # fmt + clippy + tests
make run      # QEMU boot (requires QEMU + OVMF)
```

Expected serial log (`build/qemu-serial.log`):

```
Aether OS kernel started
BootInfo OK
Aether OS M2: GDT/IDT/interrupts initialized
[timer] tick 100
```

Full CI parity:

```bash
bash scripts/ci-check.sh        # Unix
powershell -File scripts/ci-check.ps1   # Windows
```

## Dev Container (recommended)

Open the repository in a [Dev Container](https://containers.dev/) (VS Code / Cursor).
[.devcontainer/devcontainer.json](../.devcontainer/devcontainer.json) installs Rust, QEMU, OVMF,
and runs `scripts/setup-dev.sh` on create.

## Optional tools

| Tool | Purpose |
|------|---------|
| GNU Make | Convenience targets (`make boot`, `make run`) — scripts work standalone |
| `rust-analyzer` | IDE support (listed in `.vscode/extensions.json`) |
| `gdb` + `qemu-system-x86_64 -s -S` | Kernel debugging (advanced; not documented in M2) |

## Platform notes

### WSL2

Use WSL2 with Ubuntu. Install QEMU inside WSL, not on the Windows host, for `make run`.
File paths in serial logs are under the Linux filesystem.

### Windows native

PowerShell scripts (`*.ps1`) mirror the bash scripts. GNU Make is optional; invoke scripts directly.

### CI environment

GitHub Actions CI (`.github/workflows/ci.yml`) uses Ubuntu with Rust 1.85.0. The optional QEMU
job installs `qemu-system-x86` and `ovmf` via `apt`.

## Troubleshooting

| Issue | Resolution |
|-------|------------|
| `rustup` not found | Install from https://rustup.rs/ and restart your shell |
| Wrong Rust version | Run `rustup show` inside the repo; toolchain file should override |
| `build-std` errors | Set `RUSTC_BOOTSTRAP=1`; ensure `rust-src` is installed |
| OVMF not found | Copy firmware to `ovmf/` or install the `ovmf` / `edk2-ovmf` package |
| No timer ticks in log | Increase `TIMEOUT=45` when running QEMU; ticks log every ~1 s |
| Clippy failures | Run `cargo fmt --all` first; check `RUSTFLAGS=-Dwarnings` |

See also [BUILD.md](BUILD.md) and [development/getting-started.md](development/getting-started.md).

## What is not installable yet

| Artifact | Status |
|----------|--------|
| Bootable USB image for real PC | **Not available** — build locally with `make boot` |
| Signed release ISO | Planned M10 |
| Package repository | Planned M9 (host scaffold: `system/pkgmgr/`) |
| Pre-built binary releases | Draft workflow only; no stable artifacts |
| Update bundles (`.aup`) | Host skeleton only (`system/updater/`); no runtime apply |

Deployment guidance for future releases: [DEPLOYMENT.md](DEPLOYMENT.md).

## Related documents

- [BUILD.md](BUILD.md) — build targets and cross-compilation
- [README.md](../README.md) — project overview and quick start
- [docs/hardware/README.md](hardware/README.md) — supported hardware targets
