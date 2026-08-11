# Integration Tests

> **Status:** M1 QEMU smoke test shipped; M2 adds arch layout tests and updated serial expectations.

## Strategy

Aether uses a layered test pyramid:

| Layer | Location | Runs on | Purpose |
|-------|----------|---------|---------|
| **Unit** | `crates/*`, `kernel` (`#[cfg(test)]`) | Host CI | Pure logic, types, encoding helpers |
| **Integration** | `tests/` | Host CI | Cross-crate APIs, path contracts, arch layout |
| **Property** | `tests/property_*.rs` | Host CI | ABI roundtrips, GDT math, VFS path rules |
| **Fuzz stubs** | `tests/fuzz_*.rs` | Host CI | Pseudo-random syscall/ramfs sequences (no panic) |
| **Integration suite** | `tests/integration_suite.rs` | Host CI | Cross-subsystem smoke invariants |
| **QEMU smoke** | `tests/qemu_boot.rs` (`#[ignore]`) | Local / optional CI | End-to-end UEFI boot + serial output |

### Host tests (always run)

```bash
cargo test --workspace
```

This executes:

- Shared crate unit tests (`aether-types`, `aether-abi`, `aether-logger`)
- Kernel host-stub tests (GDT layout, TSS encoding)
- Integration tests in this directory:
  - `arch_gdt` — GDT descriptor math via public `gdt::layout` API
  - `arch_idt` — IDT 16-byte gate layout (host mirror of kernel encoding)
  - `property_abi` / `property_gdt` / `property_vfs` — property-style invariants
  - `fuzz_syscall` / `fuzz_ramfs` — pseudo-random fuzz stubs (deterministic PRNG)
  - `integration_suite` — cross-module smoke checks
  - `qemu_boot` — artifact path documentation and timeout sanity (non-ignored)

### Aggressive local test matrix

```bash
bash scripts/run-all-tests.sh           # Linux/macOS — all host tests
bash scripts/run-all-tests.sh --qemu    # + headless QEMU when installed

powershell -File scripts/run-all-tests.ps1
powershell -File scripts/run-all-tests.ps1 -Qemu
```

Headless QEMU only (builds boot artifacts, no user input):

```bash
bash scripts/qemu-test.sh
powershell -File scripts/qemu-test.ps1
```

### QEMU boot smoke (optional)

Requires `qemu-system-x86_64`, OVMF, and boot artifacts from `scripts/build-boot.sh`:

```bash
bash scripts/build-boot.sh          # Linux/macOS
powershell -File scripts/build-boot.ps1   # Windows

cargo test --test qemu_boot -- --ignored
# or
bash scripts/run-qemu.sh
```

**M2 expected serial output:**

1. `Aether OS kernel started`
2. `BootInfo OK` (when handoff is valid)
3. `Aether OS M2: GDT/IDT/interrupts initialized`
4. Optional: `[timer] tick N` every ~1 s at 100 Hz PIT rate

The ignored integration test asserts (1) and (3); timer ticks are logged when the QEMU run lasts long enough.

### Local CI gate

Run the full quality gate (fmt, clippy, tests, workspace + bare-metal builds):

```bash
bash scripts/ci-check.sh            # Linux/macOS
powershell -File scripts/ci-check.ps1   # Windows
```

This mirrors the GitHub Actions `quality` job in `.github/workflows/ci.yml`.

### What is not host-testable

- `LGDT` / `LIDT` / `STI` and IRQ delivery (QEMU smoke only)
- Page-fault handler paths requiring CR2
- Bare-metal-only modules (`idt::init`, PIC/PIT MMIO)

See [docs/hardware/README.md](../docs/hardware/README.md) for target platforms.
