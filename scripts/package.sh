#!/usr/bin/env bash
# Package distributable release archive with ESP layout + README.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

VERSION="$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')"
DIST="$ROOT/dist"
STAGING="$DIST/aether-os-${VERSION}"
PKG_ZIP="$DIST/aether-os-${VERSION}.zip"
ESP_TAR="$DIST/aether-os-${VERSION}-esp.tar.gz"

BOOT_EFI="$ROOT/build/esp/EFI/BOOT/BOOTX64.EFI"
KERNEL_ELF="$ROOT/build/esp/aether/kernel.elf"

if [[ ! -f "$BOOT_EFI" || ! -f "$KERNEL_ELF" ]]; then
    echo "error: boot artifacts missing — run scripts/build-release.sh first" >&2
    exit 1
fi

rm -rf "$STAGING"
mkdir -p "$STAGING/EFI/BOOT" "$STAGING/aether" "$DIST"

cp "$BOOT_EFI" "$STAGING/EFI/BOOT/BOOTX64.EFI"
cp "$KERNEL_ELF" "$STAGING/aether/kernel.elf"

cat > "$STAGING/README.txt" <<EOF
Aether OS ${VERSION}
====================

Contents:
  EFI/BOOT/BOOTX64.EFI   UEFI boot loader
  aether/kernel.elf      Bare-metal kernel (M2)

Copy the EFI/ and aether/ directories onto a FAT32 ESP, or extract
aether-os-${VERSION}-esp.tar.gz for the same layout.

Boot under QEMU + OVMF — see docs/BUILD.md in the repository.
EOF

(
    cd "$DIST"
    rm -f "$(basename "$PKG_ZIP")"
    if command -v zip >/dev/null 2>&1; then
        zip -r "$(basename "$PKG_ZIP")" "aether-os-${VERSION}"
    else
        echo "error: zip not found" >&2
        exit 1
    fi
)

tar czf "$ESP_TAR" -C "$STAGING" .

echo "package: created $PKG_ZIP"
echo "package: created $ESP_TAR"
echo "  EFI/BOOT/BOOTX64.EFI"
echo "  aether/kernel.elf"
echo "  README.txt"
