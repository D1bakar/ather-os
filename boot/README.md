# Boot Loader

> **Status:** Shipped in **Milestone M1** (UEFI + QEMU serial boot).

The UEFI boot loader:

1. Locates and loads `aether/kernel.elf` from the EFI System Partition.
2. Parses ELF64 program headers and maps PT_LOAD segments.
3. Allocates and fills a [`BootInfo`](../crates/aether-types/src/boot_info.rs) structure.
4. Calls `ExitBootServices` and jumps to the kernel entry (`RDI` = BootInfo pointer).

See [ADR-0006](../docs/adr/ADR-0006-boot-architecture.md) for the boot strategy.

## Build

```powershell
cargo build -p aether-boot --target x86_64-unknown-uefi --release
```

Output: `target/x86_64-unknown-uefi/release/bootx64.efi` (copied to `build/esp/EFI/BOOT/BOOTX64.EFI` by `scripts/build-boot.ps1`).

## ESP layout

```
EFI/BOOT/BOOTX64.EFI    ← this crate
aether/kernel.elf       ← aether-kernel bare-metal binary
```

## M2 follow-ups

- Copy full UEFI memory map into `BootInfo`.
- Locate ACPI RSDP and GOP framebuffer when available.
