#!/usr/bin/env bash
# Build boot artifacts and populate the ESP directory tree.
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
    -Z build-std=core,compiler_builtins \
    -Z build-std-features=compiler-builtins-mem

cp "$ROOT/target/x86_64-unknown-uefi/release/bootx64.efi" "$BOOT_EFI_DIR/BOOTX64.EFI"
cp "$ROOT/target/x86_64-unknown-none/release/kernel" "$KERNEL_DIR/kernel.elf"

echo "ESP ready at $ESP_DIR"
