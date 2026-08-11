# Integration Tests

> **Status:** Not started — planned for M1 (QEMU boot smoke test).

Host unit tests currently run via `cargo test --workspace` on shared crates and
the kernel stub.

Integration tests will:

1. Build boot loader and kernel for `x86_64-unknown-none` / UEFI.
2. Assemble a FAT32 disk image with ESP layout.
3. Launch QEMU headlessly and assert serial output contains the boot banner.

See [docs/hardware/README.md](../docs/hardware/README.md) for target platforms.
