#!/usr/bin/env bash
# Build Aether OS user-space binaries (host + bare-metal cross-compile).
# Output: build/user/init.elf, build/user/shell.elf
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

BUILD_DIR="$ROOT/build/user"
export CARGO_TARGET_DIR="$ROOT/target"
LINKER_SCRIPT="$ROOT/user/linker.ld"

unset RUSTFLAGS

mkdir -p "$BUILD_DIR"

echo "==> Building user-space crates (host)"
cargo build -p aether-rt -p aether-init -p aether-shell --release

echo "==> Running user-space host tests"
cargo test -p aether-rt -p aether-init -p aether-shell

echo "==> Adding bare-metal target"
rustup target add x86_64-unknown-none

echo "==> Cross-compiling init + shell (x86_64-unknown-none)"
export RUSTC_BOOTSTRAP=1
export RUSTFLAGS="-C link-arg=-T${LINKER_SCRIPT} -C relocation-model=static"

cargo build -p aether-init --no-default-features --features bare-metal \
    --target x86_64-unknown-none --release \
    -Z build-std=core,compiler_builtins \
    -Z build-std-features=compiler-builtins-mem

cargo build -p aether-shell --no-default-features --features bare-metal \
    --target x86_64-unknown-none --release \
    -Z build-std=core,compiler_builtins \
    -Z build-std-features=compiler-builtins-mem

unset RUSTFLAGS

INIT_SRC="$ROOT/target/x86_64-unknown-none/release/init"
if [[ ! -f "$INIT_SRC" ]]; then
    INIT_SRC="$ROOT/target/x86_64-unknown-none/release/init.exe"
fi

SHELL_SRC="$ROOT/target/x86_64-unknown-none/release/shell"
if [[ ! -f "$SHELL_SRC" ]]; then
    SHELL_SRC="$ROOT/target/x86_64-unknown-none/release/shell.exe"
fi

cp "$INIT_SRC" "$BUILD_DIR/init.elf"
cp "$SHELL_SRC" "$BUILD_DIR/shell.elf"

echo "User binaries ready at $BUILD_DIR"
echo "  $BUILD_DIR/init.elf"
echo "  $BUILD_DIR/shell.elf"
echo ""
echo "Status: bare-metal ELFs embedded in kernel via build.rs when present."
