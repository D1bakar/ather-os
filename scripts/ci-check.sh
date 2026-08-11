#!/usr/bin/env bash
# Full local CI gate — mirrors .github/workflows/ci.yml quality job.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

export CARGO_TERM_COLOR=always
export RUSTFLAGS="-Dwarnings"
export RUSTC_BOOTSTRAP=1

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

echo "==> cargo build --workspace"
cargo build --workspace

echo "==> cargo build UEFI boot loader (release)"
cargo build -p aether-boot --target x86_64-unknown-uefi --release

echo "==> cargo build bare-metal kernel (release)"
cargo build -p aether-kernel --no-default-features --features bare-metal \
    --target x86_64-unknown-none --release \
    -Z build-std=core,compiler_builtins \
    -Z build-std-features=compiler-builtins-mem

echo ""
echo "ci-check: all gates passed."
