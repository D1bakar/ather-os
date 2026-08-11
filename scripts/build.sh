#!/usr/bin/env bash
# Build helper scripts for Aether OS
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

usage() {
    cat <<EOF
Aether OS build helpers

Usage: $(basename "$0") <command>

Commands:
  check       Run fmt, clippy, and tests
  build       Build workspace crates
  clean       Remove build artifacts
  help        Show this message
EOF
}

cmd_check() {
    cargo fmt --all -- --check
    cargo clippy --workspace --exclude aether-boot --all-targets -- -D warnings
    cargo clippy -p aether-boot --target x86_64-unknown-uefi -- -D warnings
    cargo test --workspace
    cargo build --workspace
    echo "All checks passed."
}

cmd_build() {
    cargo build --workspace
}

cmd_clean() {
    cargo clean
    rm -rf build/
}

case "${1:-help}" in
    check)  cmd_check ;;
    build)  cmd_build ;;
    clean)  cmd_clean ;;
    help)   usage ;;
    *)
        echo "Unknown command: $1" >&2
        usage >&2
        exit 1
        ;;
esac
