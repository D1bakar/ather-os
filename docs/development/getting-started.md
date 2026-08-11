# Getting Started with Aether OS Development

This guide walks through cloning the repository, installing dependencies, building the
M1 boot path, and running the QEMU smoke test. For coding conventions, see
[code-style.md](code-style.md).

## What you will build

At **M2** (current milestone), Aether OS produces:

- A UEFI boot loader (`BOOTX64.EFI`) for a FAT32 ESP
- A bare-metal kernel (`kernel.elf`) with COM1 serial output
- GDT/IDT initialization, PIC remap, and ~100 Hz PIT timer ticks

There is no interactive shell, filesystem, or user space yet.

## Prerequisites

| Tool | Version / notes |
|------|-----------------|
| **Rust** | 1.85.0 — pinned in [rust-toolchain.toml](../../rust-toolchain.toml); includes `rust-src` for kernel `build-std` |
| **rustfmt, clippy** | Installed via `rustup component add rustfmt clippy` |
| **QEMU** | `qemu-system-x86_64` — required for `make run` and integration tests |
| **OVMF** | UEFI firmware (`OVMF_CODE.fd`, `OVMF_VARS.fd`) |
| **GNU Make** or **PowerShell** | Build orchestration |

### Cross-compilation targets

```bash
rustup target add x86_64-unknown-uefi x86_64-unknown-none
```

**M13 prep (scaffold only — not bootable):** the kernel tree includes
`kernel/src/arch/aarch64/` for future `aarch64-unknown-none` work. Install the target
when experimenting locally; no CI job or Makefile target builds it yet:

```bash
rustup target add aarch64-unknown-none
```

## Clone and verify

```bash
git clone https://github.com/aether-os/aether-os.git
cd aether-os
```

Run the host quality gate (no QEMU required):

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
```

On Windows with Make:

```powershell
make test
```

## Build the boot path

### Windows (PowerShell)

```powershell
.\scripts\build-boot.ps1
```

### Unix

```bash
bash scripts/build-boot.sh
```

**Outputs:**

| Artifact | Path |
|----------|------|
| UEFI boot loader | `build/esp/EFI/BOOT/BOOTX64.EFI` |
| Kernel ELF | `build/esp/aether/kernel.elf` |

The bare-metal kernel build uses `RUSTC_BOOTSTRAP=1` and `-Z build-std=core,compiler_builtins`.

## Run in QEMU

Install OVMF and place firmware files under `ovmf/` **or** use system paths:

| File | Common locations |
|------|------------------|
| `OVMF_CODE.fd` | `ovmf/`, `/usr/share/OVMF/`, `%ProgramFiles%\qemu\share\` |
| `OVMF_VARS.fd` | same |

### Windows

```powershell
.\scripts\run-qemu.ps1
```

### Unix / Make

```bash
make run
# or: bash scripts/run-qemu.sh
```

Serial output is written to `build/qemu-serial.log`. A successful boot contains:

```
Aether OS kernel started
BootInfo OK
```

## Integration test

The QEMU boot smoke test is **ignored by default** in CI quality gates (requires QEMU + OVMF):

```bash
cargo test -p aether-integration-tests -- --ignored
```

## Repository orientation

```
.
├── boot/           # UEFI boot loader (aether-boot) — shipped M1
├── kernel/         # Bare-metal kernel entry — shipped M1 (serial only)
├── crates/         # aether-types, aether-abi, aether-logger — shipped M0
├── scripts/        # build-boot, run-qemu
├── tests/          # QEMU integration tests
├── docs/
│   ├── adr/        # Architecture Decision Records
│   ├── architecture/  # Subsystem index
│   ├── development/   # This directory
│   └── security/      # Threat model
└── .github/workflows/ # CI
```

## Development workflow

1. Create a feature branch from `main`.
2. Make focused changes; follow [code-style.md](code-style.md).
3. Run `make test` (or equivalent `cargo` commands).
4. For architectural changes, add or update an ADR in [docs/adr/](../adr/).
5. Open a pull request using the repository template.

See [CONTRIBUTING.md](../../CONTRIBUTING.md) and [GOVERNANCE.md](../../GOVERNANCE.md).

## Milestone context

| Milestone | You can work on | Status |
|-----------|-----------------|--------|
| **M0** | Shared crates, docs, CI | Shipped |
| **M1** | Boot loader, kernel entry, serial, QEMU | **Current** |
| **M2** | GDT, IDT, paging, physical allocator | Planned |
| **M3** | APIC timer, scheduler, preemption | Planned |
| **M4** | Syscall dispatch | Planned |

Consult [ARCHITECTURE.md](../../ARCHITECTURE.md) and [docs/architecture/README.md](../architecture/README.md)
before implementing a subsystem marked **Planned**.

## Troubleshooting

| Symptom | Likely cause |
|---------|--------------|
| `OVMF_CODE.fd` not found | Install OVMF or copy firmware to `ovmf/` |
| Empty serial log | QEMU not starting; check script output for firmware path errors |
| `BootInfo invalid` in serial log | Boot loader / kernel ABI mismatch; rebuild both artifacts |
| Clippy failures on `aether-boot` | Build with `--target x86_64-unknown-uefi` (see Makefile) |

## Next steps

- Read [ARCHITECTURE.md](../../ARCHITECTURE.md) for design intent
- Review [docs/adr/](../adr/) before proposing architectural changes
- Read [docs/security/threat-model.md](../security/threat-model.md) for security assumptions
