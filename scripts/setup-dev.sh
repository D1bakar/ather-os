#!/usr/bin/env bash
# Prepare a local Aether OS development environment.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "==> Aether OS developer setup"
echo "    Repository: $ROOT"
echo ""

if ! command -v rustup >/dev/null 2>&1; then
    echo "rustup not found. Install Rust from https://rustup.rs/ and re-run this script." >&2
    exit 1
fi

echo "==> Ensuring pinned toolchain and components"
rustup show active-toolchain
rustup component add rustfmt clippy rust-src llvm-tools-preview

echo "==> Adding cross-compilation targets"
rustup target add x86_64-unknown-uefi x86_64-unknown-none

echo ""
echo "==> Optional: QEMU + OVMF (required for 'make run')"
if command -v qemu-system-x86_64 >/dev/null 2>&1; then
    echo "    qemu-system-x86_64: $(command -v qemu-system-x86_64)"
else
    echo "    qemu-system-x86_64: NOT FOUND"
    echo "    Debian/Ubuntu:  sudo apt-get install qemu-system-x86 ovmf"
    echo "    Fedora:         sudo dnf install qemu-system-x86 edk2-ovmf"
    echo "    macOS (Homebrew): brew install qemu"
fi

ovmf_found=0
for candidate in \
    "$ROOT/ovmf/OVMF_CODE.fd" \
    "/usr/share/OVMF/OVMF_CODE.fd" \
    "/usr/share/ovmf/x64/OVMF_CODE.fd" \
    "/usr/share/edk2/ovmf/OVMF_CODE.fd" \
    "/usr/share/edk2-ovmf/OVMF/OVMF_CODE.fd"
do
    if [[ -f "$candidate" ]]; then
        echo "    OVMF_CODE.fd: $candidate"
        ovmf_found=1
        break
    fi
done
if [[ "$ovmf_found" -eq 0 ]]; then
    echo "    OVMF_CODE.fd: NOT FOUND — copy OVMF_CODE.fd and OVMF_VARS.fd to ovmf/"
fi

echo ""
echo "==> Quick start"
echo "    make build     # host workspace"
echo "    make boot      # UEFI loader + kernel.elf"
echo "    make run       # QEMU smoke test"
echo "    make test      # fmt + clippy + tests"
echo ""
echo "Setup complete."
