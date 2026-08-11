#!/usr/bin/env bash
# Build boot artifacts and populate the ESP directory tree.
# Output: build/esp/EFI/BOOT/BOOTX64.EFI and build/esp/aether/kernel.elf
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

BUILD_DIR="$ROOT/build"
ESP_DIR="$BUILD_DIR/esp"
BOOT_EFI_DIR="$ESP_DIR/EFI/BOOT"
KERNEL_DIR="$ESP_DIR/aether"
export CARGO_TARGET_DIR="$ROOT/target"

mkdir -p "$BOOT_EFI_DIR" "$KERNEL_DIR"

echo "==> Adding Rust targets"
rustup target add x86_64-unknown-uefi x86_64-unknown-none

echo "==> Building UEFI boot loader"
cargo build -p aether-boot --target x86_64-unknown-uefi --release

echo "==> Building bare-metal kernel"
export RUSTC_BOOTSTRAP=1
cargo build -p aether-kernel --no-default-features --features bare-metal \
    --target x86_64-unknown-none --release \
    -Z build-std=core,alloc,compiler_builtins \
    -Z build-std-features=compiler-builtins-mem

BOOT_SRC="$ROOT/target/x86_64-unknown-uefi/release/bootx64.efi"
if [[ ! -f "$BOOT_SRC" ]]; then
    BOOT_SRC="$ROOT/target/x86_64-unknown-uefi/release/bootx64"
fi

KERNEL_SRC="$ROOT/target/x86_64-unknown-none/release/kernel"
if [[ ! -f "$KERNEL_SRC" ]]; then
    KERNEL_SRC="$ROOT/target/x86_64-unknown-none/release/kernel.exe"
fi

cp "$BOOT_SRC" "$BOOT_EFI_DIR/BOOTX64.EFI"
cp "$KERNEL_SRC" "$KERNEL_DIR/kernel.elf"

echo "ESP ready at $ESP_DIR"
echo "  $BOOT_EFI_DIR/BOOTX64.EFI"
echo "  $KERNEL_DIR/kernel.elf"
