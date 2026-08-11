#!/usr/bin/env bash
# Production release build — workspace release + ESP boot artifacts.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

export CARGO_TERM_COLOR=always
export RUSTC_BOOTSTRAP=1
export RUSTFLAGS="${RUSTFLAGS:--Dwarnings}"
export SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH:-$(git log -1 --format=%ct 2>/dev/null || echo 0)}"

echo "==> Aether OS production build"
echo "    SOURCE_DATE_EPOCH=$SOURCE_DATE_EPOCH"

echo "==> rustup targets"
rustup target add x86_64-unknown-none x86_64-unknown-uefi

echo "==> cargo fmt --check"
cargo fmt --all -- --check

echo "==> cargo clippy (host workspace)"
cargo clippy --workspace --exclude aether-boot --all-targets -- -D warnings

echo "==> cargo clippy (UEFI boot loader)"
cargo clippy -p aether-boot --target x86_64-unknown-uefi -- -D warnings

echo "==> cargo test --workspace"
cargo test --workspace

echo "==> cargo build --workspace --release"
cargo build --workspace --release

echo "==> boot artifacts (ESP layout)"
bash "$ROOT/scripts/build-boot.sh"

echo ""
echo "build-release: complete — ESP at build/esp/"
