# Boot Loader

> **Status:** Placeholder — implementation begins in **Milestone M1**.

The UEFI boot loader will:

1. Locate and load the kernel ELF from the EFI System Partition.
2. Collect the UEFI memory map.
3. Exit boot services.
4. Jump to the kernel entry point with a `BootInfo` handoff structure.

See [ADR 001](docs/architecture/001-initial-decisions.md) for the boot strategy.

## M1 Deliverables

- `boot/Cargo.toml` with `x86_64-unknown-uefi` target
- UEFI application using the `uefi` crate
- FAT32 ESP layout (`EFI/BOOT/BOOTX64.EFI`, `aether/kernel.elf`)
- Handoff structure shared via `aether-types`
